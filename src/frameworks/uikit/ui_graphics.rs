/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `UIGraphics.h`

use touchHLE_proc_macros::boxify;

use crate::dyld::{export_c_func, FunctionExports};
use crate::frameworks::core_graphics::cg_context::{
    CGContextRef, CGContextRelease, CGContextRetain,
};
use crate::objc::nil;
use crate::{Environment, export_c_func_async};

#[derive(Default)]
pub(super) struct State {
    pub(super) context_stack: Vec<CGContextRef>,
}

#[boxify]
async fn UIGraphicsPushContext(env: &mut Environment, context: CGContextRef) {
    CGContextRetain(env, context).await;
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .push(context);
}
#[boxify]
async fn UIGraphicsPopContext(env: &mut Environment) {
    let context = env.framework_state.uikit.ui_graphics.context_stack.pop();
    CGContextRelease(env, context.unwrap()).await;
}
pub(super) fn UIGraphicsGetCurrentContext(env: &mut Environment) -> CGContextRef {
    env.framework_state
        .uikit
        .ui_graphics
        .context_stack
        .last()
        .copied()
        .unwrap_or(nil)
}

pub const FUNCTIONS: FunctionExports = &[
    export_c_func_async!(UIGraphicsPushContext(_)),
    export_c_func_async!(UIGraphicsPopContext()),
    export_c_func!(UIGraphicsGetCurrentContext()),
];
