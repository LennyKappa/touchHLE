/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFRunLoop`.
//!
//! This is not even toll-free bridged to `NSRunLoop` in Apple's implementation,
//! but here it is the same type.

use touchHLE_proc_macros::boxify;

use crate::dyld::{export_c_func_async, ConstantExports, FunctionExports, HostConstant};
use crate::objc::msg_class;
use crate::Environment;

pub type CFRunLoopRef = super::CFTypeRef;
pub type CFRunLoopMode = super::cf_string::CFStringRef;

#[boxify]
async fn CFRunLoopGetCurrent(env: &mut Environment) -> CFRunLoopRef {
    msg_class![env; NSRunLoop currentRunLoop]
}

#[boxify]
pub async fn CFRunLoopGetMain(env: &mut Environment) -> CFRunLoopRef {
    msg_class![env; NSRunLoop mainRunLoop]
}

pub const kCFRunLoopCommonModes: &str = "kCFRunLoopCommonModes";
pub const kCFRunLoopDefaultMode: &str = "kCFRunLoopDefaultMode";

pub const CONSTANTS: ConstantExports = &[
    (
        "_kCFRunLoopCommonModes",
        HostConstant::NSString(kCFRunLoopCommonModes),
    ),
    (
        "_kCFRunLoopDefaultMode",
        HostConstant::NSString(kCFRunLoopDefaultMode),
    ),
];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(CFRunLoopGetCurrent()),
    export_c_func_async!(CFRunLoopGetMain()),
];
