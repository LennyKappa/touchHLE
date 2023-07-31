/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! Internal mutex interface.

use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::libc::errno::{EBUSY, EDEADLK, EPERM};
use crate::libc::pthread::mutex::HostMutexId;
use crate::{Environment, ThreadID};

#[derive(Default)]
pub struct MutexState {
    // TODO?: Maybe this should be a Vec instead? It would be bad if there were many mutexes over
    // the lifetime of an application, but it would perform better.
    // Maybe it could also be a fixed size allocator? (although that seems a little overkill)
    mutexes: HashMap<HostMutexId, MutexHostObject>,
    // Hopefully there will never be more than 2^64 mutexes in an applications lifetime :P
    mutex_count: u64,
}
impl MutexState {
    fn get_mut(env: &mut Environment) -> &mut Self {
        &mut env.mutex_state
    }
    fn get(env: &Environment) -> &Self {
        &env.mutex_state
    }
}

struct MutexHostObject {
    type_: MutexType,
    waiting_count: u32,
    /// The `NonZeroU32` is the number of locks on this thread (if it's a
    /// recursive mutex).
    locked: Option<(ThreadID, NonZeroU32)>,
}

#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum MutexType {
    PTHREAD_MUTEX_NORMAL = 0,
    PTHREAD_MUTEX_ERRORCHECK = 1,
    PTHREAD_MUTEX_RECURSIVE = 2,
}

impl TryFrom<i32> for MutexType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MutexType::PTHREAD_MUTEX_NORMAL),
            1 => Ok(MutexType::PTHREAD_MUTEX_ERRORCHECK),
            2 => Ok(MutexType::PTHREAD_MUTEX_RECURSIVE),
            _ => Err("Value is not a valid mutex type!"),
        }
    }
}
pub const PTHREAD_MUTEX_DEFAULT: MutexType = MutexType::PTHREAD_MUTEX_NORMAL;

/// Initializes a mutex and returns a handle to it. Similar to pthread_mutex_init, but for host code.
pub fn host_mutex_init(env: &mut Environment, mutex_type: MutexType) -> HostMutexId {
    let state = MutexState::get_mut(env);
    let mutex_id = state.mutex_count;
    state.mutex_count = state.mutex_count.checked_add(1).unwrap();
    state.mutexes.insert(
        mutex_id,
        MutexHostObject {
            type_: mutex_type,
            waiting_count: 0,
            locked: None,
        },
    );
    log_dbg!(
        "Created mutex #{}, type {:?}",
        state.mutex_count,
        mutex_type
    );
    mutex_id
}

/// Locks a mutex and returns the lock count or an error (as errno). Similar to
/// pthread_mutex_lock, but for host code.
/// NOTE: This only takes effect _after_ the calling function returns to the host run loop
/// ([crate::Environment::run]). As such, this should only be called right before a function
/// returns (to the host run loop).
pub fn host_mutex_lock(env: &mut Environment, mutex_id: HostMutexId) -> Result<u32, i32> {
    let current_thread = env.current_thread;
    let host_object: &mut _ = MutexState::get_mut(env).mutexes.get_mut(&mutex_id).unwrap();

    let Some((locking_thread, lock_count)) = host_object.locked else {
        log_dbg!("Locked mutex #{} for thread {}.", mutex_id, current_thread);
        host_object.locked = Some((current_thread, NonZeroU32::new(1).unwrap()));
        return Ok(1);
    };

    if locking_thread == current_thread {
        match host_object.type_ {
            MutexType::PTHREAD_MUTEX_NORMAL => {
                // This case would be a deadlock, we may as well panic.
                panic!(
                    "Attempted to lock non-error-checking mutex #{} for thread {}, already locked by same thread!",
                    mutex_id, current_thread,
                );
            }
            MutexType::PTHREAD_MUTEX_ERRORCHECK => {
                log_dbg!("Attempted to lock error-checking mutex #{} for thread {}, already locked by same thread! Returning EDEADLK.", mutex_id, current_thread);
                return Err(EDEADLK);
            }
            MutexType::PTHREAD_MUTEX_RECURSIVE => {
                log_dbg!(
                    "Increasing lock level on recursive mutex #{}, currently locked by thread {}.",
                    mutex_id,
                    locking_thread,
                );
                host_object.locked = Some((locking_thread, lock_count.checked_add(1).unwrap()));
                return Ok(lock_count.get() + 1);
            }
        }
    }

    // Add to the waiting count, so that the mutex isn't destroyed. This is subtracted in
    // [host_mutex_relock_unblocked].
    host_object.waiting_count += 1;

    // Mutex is already locked, block thread until it isn't.
    env.block_on_mutex(mutex_id);
    // Lock count is always 1 after a thread-blocking lock.
    Ok(1)
}

/// Unlocks a mutex and returns the lock count or an error (as errno). Similar to
/// pthread_mutex_unlock, but for host code.
pub fn host_mutex_unlock(env: &mut Environment, mutex_id: HostMutexId) -> Result<u32, i32> {
    let current_thread = env.current_thread;
    let host_object: &mut _ = MutexState::get_mut(env).mutexes.get_mut(&mutex_id).unwrap();

    let Some((locking_thread, lock_count)) = host_object.locked else {
        match host_object.type_ {
            MutexType::PTHREAD_MUTEX_NORMAL => {
                // This case is undefined, we may as well panic.
                panic!(
                    "Attempted to unlock non-error-checking mutex #{} for thread {}, already unlocked!",
                    mutex_id, current_thread,
                );
            },
            MutexType::PTHREAD_MUTEX_ERRORCHECK | MutexType::PTHREAD_MUTEX_RECURSIVE => {
                log_dbg!(
                    "Attempted to unlock error-checking or recursive mutex #{} for thread {}, already unlocked! Returning EPERM.",
                    mutex_id, current_thread,
                );
                return Err(EPERM);
            },
        }
    };

    if locking_thread != current_thread {
        match host_object.type_ {
            MutexType::PTHREAD_MUTEX_NORMAL => {
                // This case is undefined, we may as well panic.
                panic!(
                    "Attempted to unlock non-error-checking mutex #{} for thread {}, locked by different thread {}!",
                    mutex_id, current_thread, locking_thread,
                );
            }
            MutexType::PTHREAD_MUTEX_ERRORCHECK | MutexType::PTHREAD_MUTEX_RECURSIVE => {
                log_dbg!(
                    "Attempted to unlock error-checking or recursive mutex #{} for thread {}, locked by different thread {}! Returning EPERM.",
                    mutex_id, current_thread, locking_thread,
                );
                return Err(EPERM);
            }
        }
    }

    if lock_count.get() == 1 {
        log_dbg!(
            "Unlocked mutex #{} for thread {}.",
            mutex_id,
            current_thread
        );
        host_object.locked = None;
        Ok(0)
    } else {
        assert!(host_object.type_ == MutexType::PTHREAD_MUTEX_RECURSIVE);
        log_dbg!(
            "Decreasing lock level on recursive mutex #{}, currently locked by thread {}.",
            mutex_id,
            locking_thread
        );
        host_object.locked = Some((
            locking_thread,
            NonZeroU32::new(lock_count.get() - 1).unwrap(),
        ));
        Ok(lock_count.get() - 1)
    }
}

/// Destroys a mutex and returns an error on failure (as errno). Similar to
/// pthread_mutex_destroy, but for host code. Note that the mutex is not destroyed on an Err return.
pub fn host_mutex_destroy(env: &mut Environment, mutex_id: HostMutexId) -> Result<(), i32> {
    let state = MutexState::get_mut(env);
    let host_object = state.mutexes.get_mut(&mutex_id).unwrap();
    if host_object.locked.is_some() {
        log_dbg!("Attempted to destroy currently locked mutex, returning EBUSY!");
        return Err(EBUSY);
    } else if host_object.waiting_count != 0 {
        log_dbg!("Attempted to destroy mutex with waiting locks, returning EBUSY!");
        return Err(EBUSY);
    }
    // TODO?: If we switch to a vec-based system, we should reuse destroyed ids if they are at the
    // top of the stack.
    state.mutexes.remove(&mutex_id);
    Ok(())
}

pub fn host_mutex_is_locked(env: &Environment, mutex_id: HostMutexId) -> bool {
    let state = MutexState::get(env);
    state
        .mutexes
        .get(&mutex_id)
        .map_or(false, |host_obj| host_obj.locked.is_some())
}

/// Relock mutex that was just unblocked. This should probably only be used by the thread scheduler.
pub fn host_mutex_relock_unblocked(env: &mut Environment, mutex_id: HostMutexId) {
    host_mutex_lock(env, mutex_id).unwrap();
    MutexState::get_mut(env)
        .mutexes
        .get_mut(&mutex_id)
        .unwrap()
        .waiting_count -= 1;
}
