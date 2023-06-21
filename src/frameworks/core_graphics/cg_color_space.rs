/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CGColorSpace.h`

use touchHLE_proc_macros::boxify;

use crate::dyld::{export_c_func, ConstantExports, FunctionExports, HostConstant};
use crate::frameworks::core_foundation::cf_string::CFStringRef;
use crate::frameworks::core_foundation::{CFRelease, CFRetain, CFTypeRef};
use crate::frameworks::foundation::ns_string;
use crate::objc::{msg, objc_classes, ClassExports, HostObject};
use crate::{Environment, export_c_func_async};

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// CGColorSpace seems to be a CFType-based type, but in our implementation
// those are just Objective-C types, so we need a class for it, but its name is
// not visible anywhere.
@implementation _touchHLE_CGColorSpace: NSObject
@end

};

pub(super) struct CGColorSpaceHostObject {
    pub(super) name: &'static str,
}
impl HostObject for CGColorSpaceHostObject {}

pub type CGColorSpaceRef = CFTypeRef;

#[boxify]
pub async fn CGColorSpaceCreateWithName(env: &mut Environment, name: CFStringRef) -> CGColorSpaceRef {
    let generic_rgb = ns_string::get_static_str(env, kCGColorSpaceGenericRGB).await;
    // TODO: support more color spaces
    assert!(msg![env; name isEqualToString:generic_rgb]);

    let isa = env
        .objc
        .get_known_class("_touchHLE_CGColorSpace", &mut env.mem);
    env.objc.alloc_object(
        isa,
        Box::new(CGColorSpaceHostObject {
            name: kCGColorSpaceGenericRGB,
        }),
        &mut env.mem,
    )
}

pub fn CGColorSpaceCreateDeviceRGB(env: &mut Environment) -> CGColorSpaceRef {
    // TODO: figure out what characteristics kCGColorSpaceDeviceRGB actually has on an iPhone
    let isa = env
        .objc
        .get_known_class("_touchHLE_CGColorSpace", &mut env.mem);
    env.objc.alloc_object(
        isa,
        Box::new(CGColorSpaceHostObject {
            name: kCGColorSpaceGenericRGB,
        }),
        &mut env.mem,
    )
}

pub fn CGColorSpaceRelease(env: &mut Environment, cs: CGColorSpaceRef) {
    if !cs.is_null() {
        CFRelease(env, cs);
    }
}
#[boxify]
pub async fn CGColorSpaceRetain(env: &mut Environment, cs: CGColorSpaceRef) -> CGColorSpaceRef {
    if !cs.is_null() {
        CFRetain(env, cs).await
    } else {
        cs
    }
}

pub const kCGColorSpaceGenericRGB: &str = "kCGColorSpaceGenericRGB";

pub const CONSTANTS: ConstantExports = &[(
    "_kCGColorSpaceGenericRGB",
    HostConstant::NSString(kCGColorSpaceGenericRGB),
)];

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(CGColorSpaceCreateWithName(_)),
    export_c_func!(CGColorSpaceCreateDeviceRGB()),
    export_c_func_async!(CGColorSpaceRetain(_)),
    export_c_func!(CGColorSpaceRelease(_)),
];
