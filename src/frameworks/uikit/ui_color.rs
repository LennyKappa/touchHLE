/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIColor`.

use crate::frameworks::core_graphics::CGFloat;
use crate::mem::MutPtr;
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, ClassExports, HostObject, NSZonePtr, SEL,
};
use crate::Environment;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    standard_colors: HashMap<SEL, id>,
}

async fn get_standard_color(
    env: &mut Environment,
    sel: SEL,
    r: CGFloat,
    g: CGFloat,
    b: CGFloat,
    a: CGFloat,
) -> id {
    if let Some(&existing) = env.framework_state.uikit.ui_color.standard_colors.get(&sel) {
        existing
    } else {
        let new: id = msg_class![env; UIColor alloc];
        let new: id = msg![env; new initWithRed:r green:g blue:b alpha:a];
        env.framework_state
            .uikit
            .ui_color
            .standard_colors
            .insert(sel, new);
        new
    }
}

struct UIColorHostObject {
    rgba: (CGFloat, CGFloat, CGFloat, CGFloat),
}
impl HostObject for UIColorHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation UIColor: NSObject

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(UIColorHostObject {
        rgba: (0.0, 0.0, 0.0, 0.0),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)colorWithRed:(CGFloat)r
             green:(CGFloat)g
              blue:(CGFloat)b
             alpha:(CGFloat)a {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithRed:r green:g blue:b alpha:a];
    autorelease(env, new).await
}

+ (id)colorWithWhite:(CGFloat)w alpha:(CGFloat)a {
    let new: id = msg![env; this alloc];
    let new: id = msg![env; new initWithWhite: w alpha: a];
    autorelease(env, new).await
}

+ (id)clearColor    { get_standard_color(env, _cmd, 0.0, 0.0, 0.0, 0.0).await }
+ (id)blackColor    { get_standard_color(env, _cmd, 0.0, 0.0, 0.0, 1.0).await }
+ (id)whiteColor    { get_standard_color(env, _cmd, 1.0, 1.0, 1.0, 1.0).await }
+ (id)darkGrayColor {
    get_standard_color(env, _cmd, 1.0/3.0, 1.0/3.0, 1.0/3.0, 1.0).await
}
+ (id)grayColor {
    get_standard_color(env, _cmd, 1.0/2.0, 1.0/2.0, 1.0/2.0, 1.0).await
}
+ (id)lightGrayColor {
    get_standard_color(env, _cmd, 2.0/3.0, 2.0/3.0, 2.0/3.0, 1.0).await
}
+ (id)blueColor     { get_standard_color(env, _cmd, 0.0, 0.0, 1.0, 1.0).await}
+ (id)brownColor    { get_standard_color(env, _cmd, 0.6, 0.4, 0.2, 1.0).await }
+ (id)cyanColor     { get_standard_color(env, _cmd, 0.0, 1.0, 1.0, 1.0).await }
+ (id)greenColor    { get_standard_color(env, _cmd, 0.0, 1.0, 0.0, 1.0).await }
+ (id)magentaColor  { get_standard_color(env, _cmd, 1.0, 0.0, 1.0, 1.0).await }
+ (id)orangeColor   { get_standard_color(env, _cmd, 1.0, 0.5, 0.0, 1.0).await }
+ (id)purpleColor   { get_standard_color(env, _cmd, 0.5, 0.0, 1.5, 1.0).await }
+ (id)redColor      { get_standard_color(env, _cmd, 1.0, 0.0, 0.0, 1.0).await }
+ (id)yellowColor   { get_standard_color(env, _cmd, 1.0, 1.0, 0.0, 1.0).await }

// TODO: more initializers, set methods, more accessors

- (id)initWithWhite:(CGFloat)w alpha:(CGFloat)a {
    let w = w.clamp(0.0, 1.0);
    let a = a.clamp(0.0, 1.0);

    env.objc.borrow_mut::<UIColorHostObject>(this).rgba = (w, w, w, a);

    this
}

- (id)initWithRed:(CGFloat)r
            green:(CGFloat)g
             blue:(CGFloat)b
            alpha:(CGFloat)a {
    env.objc.borrow_mut::<UIColorHostObject>(this).rgba = (r, g, b, a);
    this
}

- (bool)getRed:(MutPtr<CGFloat>)r
         green:(MutPtr<CGFloat>)g
          blue:(MutPtr<CGFloat>)b
         alpha:(MutPtr<CGFloat>)a {
    let (r_, g_, b_, a_) = env.objc.borrow::<UIColorHostObject>(this).rgba;
    env.mem.write(r, r_);
    env.mem.write(g, g_);
    env.mem.write(b, b_);
    env.mem.write(a, a_);
    true
}

@end

};
