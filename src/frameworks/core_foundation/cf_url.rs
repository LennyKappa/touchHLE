/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFURL`.
//!
//! This is toll-free bridged to `NSURL` in Apple's implementation. Here it is
//! the same type.

use touchHLE_proc_macros::boxify;

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::CFIndex;
use crate::dyld::FunctionExports;
use crate::frameworks::foundation::ns_string::NSUTF8StringEncoding;
use crate::frameworks::foundation::NSUInteger;
use crate::mem::{ConstPtr, MutPtr};
use crate::objc::{id, msg, msg_class};
use crate::{Environment, export_c_func_async};

pub type CFURLRef = super::CFTypeRef;

#[boxify]
pub async fn CFURLGetFileSystemRepresentation(
    env: &mut Environment,
    url: CFURLRef,
    resolve_against_base: bool,
    buffer: MutPtr<u8>,
    buffer_size: CFIndex,
) -> bool {
    assert!(!resolve_against_base); // unimplemented
    let buffer_size: NSUInteger = buffer_size.try_into().unwrap();

    msg![env; url getFileSystemRepresentation:buffer
                                    maxLength:buffer_size]
}

#[boxify]
pub async fn CFURLCreateFromFileSystemRepresentation(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    buffer: ConstPtr<u8>,
    buffer_size: CFIndex,
    is_directory: bool,
) -> CFURLRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented

    let buffer_size: NSUInteger = buffer_size.try_into().unwrap();

    let string: id = msg_class![env; NSString alloc];
    let string: id = msg![env; string initWithBytes:buffer
                                             length:buffer_size
                                           encoding:NSUTF8StringEncoding];

    let url: id = msg_class![env; NSURL alloc];
    msg![env; url initFileURLWithPath:string isDirectory:is_directory]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(CFURLGetFileSystemRepresentation(_, _, _, _)),
    export_c_func_async!(CFURLCreateFromFileSystemRepresentation(_, _, _, _)),
];
