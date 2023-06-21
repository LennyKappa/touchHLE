/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFArray` and `CFMutableArray`.
//!
//! These are toll-free bridged to `NSArray` and `NSMutableArray` in Apple's
//! implementation. Here they are the same types.

use touchHLE_proc_macros::boxify;

use super::cf_allocator::{kCFAllocatorDefault, CFAllocatorRef};
use super::CFIndex;
use crate::dyld::FunctionExports;
use crate::frameworks::foundation::NSUInteger;
use crate::mem::ConstVoidPtr;
use crate::objc::{id, msg, msg_class};
use crate::{Environment, export_c_func_async};

#[allow(dead_code)]
pub type CFArrayRef = super::CFTypeRef;
pub type CFMutableArrayRef = super::CFTypeRef;

#[boxify]
async fn CFArrayCreateMutable(
    env: &mut Environment,
    allocator: CFAllocatorRef,
    capacity: CFIndex,
    callbacks: ConstVoidPtr, // TODO, should be `const CFArrayCallBacks*`
) -> CFMutableArrayRef {
    assert!(allocator == kCFAllocatorDefault); // unimplemented
    assert!(capacity == 0); // TODO: fixed capacity support
    assert!(callbacks.is_null()); // TODO: support retaining etc

    msg_class![env; _touchHLE_NSMutableArray_non_retaining new]
}

#[boxify]
async fn CFArrayGetCount(env: &mut Environment, array: CFArrayRef) -> CFIndex {
    let count: NSUInteger = msg![env; array count];
    count.try_into().unwrap()
}

#[boxify]
async fn CFArrayGetValueAtIndex(env: &mut Environment, array: CFArrayRef, idx: CFIndex) -> ConstVoidPtr {
    let idx: NSUInteger = idx.try_into().unwrap();
    let value: id = msg![env; array objectAtIndex:idx];
    value.cast().cast_const()
}

#[boxify]
async fn CFArrayAppendValue(env: &mut Environment, array: CFMutableArrayRef, value: ConstVoidPtr) {
    let value: id = value.cast().cast_mut();
    msg![env; array addObject:value]
}

#[boxify]
async fn CFArrayRemoveValueAtIndex(env: &mut Environment, array: CFMutableArrayRef, idx: CFIndex) {
    let idx: NSUInteger = idx.try_into().unwrap();
    msg![env; array removeObjectAtIndex:idx]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(CFArrayCreateMutable(_, _, _)),
    export_c_func_async!(CFArrayGetCount(_)),
    export_c_func_async!(CFArrayGetValueAtIndex(_, _)),
    export_c_func_async!(CFArrayAppendValue(_, _)),
    export_c_func_async!(CFArrayRemoveValueAtIndex(_, _)),
];
