/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! POSIX `sys/stat.h`

use super::{off_t, FileDescriptor};
use crate::dyld::{export_c_func, FunctionExports};
use crate::fs::{FileType, GuestPath, Metadata};
use crate::libc::time::timespec;
use crate::mem::{ConstPtr, MutPtr, SafeRead};
use crate::Environment;

#[allow(non_camel_case_types)]
pub type dev_t = u32;

#[allow(non_camel_case_types)]
pub type mode_t = u16;

#[allow(non_camel_case_types)]
pub type nlink_t = u16;

#[allow(non_camel_case_types)]
pub type ino_t = u64;

#[allow(non_camel_case_types)]
pub type uid_t = u32;

#[allow(non_camel_case_types)]
pub type gid_t = u32;

#[allow(non_camel_case_types)]
pub type blkcnt_t = i64;

#[allow(non_camel_case_types)]
pub type blksize_t = i32;

#[allow(non_camel_case_types)]
#[repr(C, packed)]
#[derive(Default)]
pub struct stat {
    st_dev: dev_t,
    st_mode: mode_t,
    st_nlink: nlink_t,
    st_ino: ino_t,
    st_uid: uid_t,
    st_gid: gid_t,
    st_rdev: dev_t,
    st_atimespec: timespec,
    st_mtimespec: timespec,
    st_ctimespec: timespec,
    st_birthtimespec: timespec,
    st_size: off_t,
    st_blocks: blkcnt_t,
    st_blksize: blksize_t,
    st_flags: u32,
    st_gen: u32,
    st_lspare: i32,
    st_qspare: [u64; 2],
}

unsafe impl SafeRead for stat {}

fn mkdir(env: &mut Environment, path: ConstPtr<u8>, mode: mode_t) -> i32 {
    // TODO: respect the mode
    match env
        .fs
        .create_dir(GuestPath::new(&env.mem.cstr_at_utf8(path).unwrap()))
    {
        Ok(()) => {
            log_dbg!("mkdir({:?}, {:#x}) => 0", path, mode);
            0
        }
        Err(()) => {
            // TODO: set errno
            log!(
                "Warning: mkdir({:?}, {:#x}) failed, returning -1",
                path,
                mode,
            );
            -1
        }
    }
}

fn fstat(env: &mut Environment, fd: FileDescriptor, buf: MutPtr<stat>) -> i32 {
    // TODO: error handling for unknown fd?
    let file = env.libc_state.posix_io.file_for_fd(fd).unwrap();

    log!("Warning: fstat() call, this function is mostly unimplemented");

    let metadata = file.file.metadata();
    let file_stat = metadata_to_stat(&metadata);
    env.mem.write(buf, file_stat);

    0 // success
}

fn stat(env: &mut Environment, path: ConstPtr<u8>, buf: MutPtr<stat>) -> i32 {
    let pn = env.mem.cstr_at_utf8(path).expect("Non UTF-8 stat() path!");
    let path = GuestPath::new(pn);
    match env.fs.get_metadata(path) {
        None => -1,
        Some(metadata) => {
            let file_stat = metadata_to_stat(&metadata);
            env.mem.write(buf, file_stat);
            0 // success
        }
    }
}

// TODO: This is still very incomplete, due in part to lack of metadata
// collected and part to fields that don't make sense with the current
// filesystem model.
fn metadata_to_stat(metadata: &Metadata) -> stat {
    let mut file_stat = stat::default();

    file_stat.st_mode = match metadata.filetype {
        FileType::RegularFile => 0o0100000,
        FileType::Directory => 0o0040000,
    } | match metadata.permissions {
        // TODO: The current permission model doesn't have any model of groups
        // or other users, so the other mode bits are made up.
        (false, false, false) => 0o000,
        (false, false, true) => 0o111,
        (false, true, false) => 0o222,
        (false, true, true) => 0o333,
        (true, false, false) => 0o444,
        (true, false, true) => 0o555,
        (true, true, false) => 0o644,
        (true, true, true) => 0o755,
    };
    file_stat.st_size = metadata.size as i64;

    file_stat
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func!(mkdir(_, _)),
    export_c_func!(fstat(_, _)),
    export_c_func!(stat(_, _)),
];
