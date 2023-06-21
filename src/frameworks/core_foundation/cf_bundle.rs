/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CFBundle`.
//!
//! This is not even toll-free bridged to `NSBundle` in Apple's implementation,
//! but here it is the same type.

use touchHLE_proc_macros::boxify;

use super::cf_string::CFStringRef;
use super::cf_url::CFURLRef;
use crate::dyld::{export_c_func_async, FunctionExports};
use crate::objc::{msg, msg_class};
use crate::Environment;

pub type CFBundleRef = super::CFTypeRef;

#[boxify]
async fn CFBundleGetMainBundle(env: &mut Environment) -> CFBundleRef {
    msg_class![env; NSBundle mainBundle]
}

#[boxify]
async fn CFBundleCopyResourcesDirectoryURL(env: &mut Environment, bundle: CFBundleRef) -> CFURLRef {
    let url: CFURLRef = msg![env; bundle resourceURL];
    msg![env; url copy]
}

#[boxify]
async fn CFBundleCopyResourceURL(
    env: &mut Environment,
    bundle: CFBundleRef,
    resource_name: CFStringRef,
    resource_type: CFStringRef,
    sub_dir_name: CFStringRef,
) -> CFURLRef {
    let url: CFURLRef = msg![env; bundle URLForResource:resource_name
                                          withExtension:resource_type
                                           subdirectory:sub_dir_name];
    msg![env; url copy]
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(CFBundleGetMainBundle()),
    export_c_func_async!(CFBundleCopyResourcesDirectoryURL(_)),
    export_c_func_async!(CFBundleCopyResourceURL(_, _, _, _)),
];
