/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `NSAutoreleasePool`.

use crate::objc::{id, msg, objc_classes, release, ClassExports, HostObject, NSZonePtr};
use crate::Environment;

#[derive(Default)]
pub struct State {
    pool_stack: Vec<id>,
}
impl State {
    fn get(env: &mut Environment) -> &mut Self {
        &mut env.framework_state.foundation.ns_autorelease_pool
    }
}

struct NSAutoreleasePoolHostObject {
    /// This is allowed to contain duplicates, which get released several times!
    objects: Vec<id>,
}
impl HostObject for NSAutoreleasePoolHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation NSAutoreleasePool: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(NSAutoreleasePoolHostObject {
        objects: Vec::new(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (())addObject:(id)obj {
    if let Some(current_pool) = State::get(env).pool_stack.last().copied() {
        msg![env; current_pool addObject:obj]
    } else {
        log_dbg!("Warning: no active NSAutoreleasePool, leaking {:?}", obj);
    }
}

- (id)init {
    assert!(env.current_thread == 0); // TODO: per-thread stacks
    State::get(env).pool_stack.push(this);
    log_dbg!("New pool: {:?}", this);
    this
}

- (())addObject:(id)obj {
    env.objc.borrow_mut::<NSAutoreleasePoolHostObject>(this).objects.push(obj);
}

- (id)retain {
    // TODO: throw proper exception?
    panic!("NSAutoreleasePool can't be retained!");
}
- (id)autorelease {
    // TODO: throw proper exception?
    panic!("NSAutoreleasePool can't be autoreleased!");
}

- (())drain {
    msg![env; this release]
}

- (())dealloc {
    log_dbg!("Draining pool: {:?}", this);
    let pop_res = State::get(env).pool_stack.pop();
    assert!(pop_res == Some(this));
    let host_obj: &mut NSAutoreleasePoolHostObject = env.objc.borrow_mut(this);
    let objects = std::mem::take(&mut host_obj.objects);
    env.objc.dealloc_object(this, &mut env.mem);
    for object in objects {
        release(env, object).await;
    }
}

@end

};
