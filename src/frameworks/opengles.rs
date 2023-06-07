/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! OpenGL ES and EAGL.
//!
//! The OpenGL ES implementation is arranged in layers:
//!
//! - `gles_generic` provides an abstraction over OpenGL ES implementations.
//! - `gles_guest` wraps `guest_generic` to expose OpenGL ES to the guest app.
//! - Various child modules provide implementations:
//!   - `gles1_native` passes through native OpenGL ES 1.1.
//!   - `gles1_on_gl2` provides an implementation of OpenGL ES 1.1 using OpenGL
//!     2.1 compatibility profile.
//!   - There might be more in future.
//!
//! Useful resources for OpenGL ES 1.1:
//! - [Reference pages](https://registry.khronos.org/OpenGL-Refpages/es1.1/xhtml/)
//! - [Specification](https://registry.khronos.org/OpenGL/specs/es/1.1/es_full_spec_1.1.pdf)
//! - Extensions:
//!   - [OES_framebuffer_object](https://registry.khronos.org/OpenGL/extensions/OES/OES_framebuffer_object.txt)
//!   - [IMG_texture_compression_pvrtc](https://registry.khronos.org/OpenGL/extensions/IMG/IMG_texture_compression_pvrtc.txt)
//!   - [OES_compressed_paletted_texture](https://registry.khronos.org/OpenGL/extensions/OES/OES_compressed_paletted_texture.txt) (also incorporated into the main spec)
//!
//! Useful resources for OpenGL 2.1:
//! - [Reference pages](https://registry.khronos.org/OpenGL-Refpages/gl2.1/)
//! - [Specification](https://registry.khronos.org/OpenGL/specs/gl/glspec21.pdf)
//! - Extensions:
//!   - [EXT_framebuffer_object](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_framebuffer_object.txt)
//!
//! Useful resources for both:
//! - Extensions:
//!   - [EXT_texture_filter_anisotropic](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_filter_anisotropic.txt)
//!   - [EXT_texture_lod_bias](https://registry.khronos.org/OpenGL/extensions/EXT/EXT_texture_lod_bias.txt)

pub mod eagl;
mod gles1_native;
mod gles1_on_gl2;
mod gles_generic;
mod gles_guest;
mod util;

use gles1_native::GLES1Native;
use gles1_on_gl2::GLES1OnGL2;
use gles_generic::GLES;
pub use gles_guest::FUNCTIONS;

/// Labels for [GLES] implementations and an abstraction for constructing them.
#[derive(Copy, Clone)]
pub enum GLESImplementation {
    /// [GLES1Native].
    GLES1Native,
    /// [GLES1OnGL2].
    GLES1OnGL2,
}
impl GLESImplementation {
    /// List of OpenGL ES 1.1 implementations in order of preference.
    pub const GLES1_IMPLEMENTATIONS: &[Self] = &[Self::GLES1Native, Self::GLES1OnGL2];
    /// Convert from short name used for command-line arguments. Returns [Err]
    /// if name is not recognized..
    pub fn from_short_name(name: &str) -> Result<Self, ()> {
        match name {
            "gles1_on_gl2" => Ok(Self::GLES1OnGL2),
            "gles1_native" => Ok(Self::GLES1Native),
            _ => Err(()),
        }
    }
    /// See [GLES::description].
    pub fn description(self) -> &'static str {
        match self {
            Self::GLES1Native => GLES1Native::description(),
            Self::GLES1OnGL2 => GLES1OnGL2::description(),
        }
    }
    /// See [GLES::new].
    pub fn construct(self, window: &mut crate::window::Window) -> Result<Box<dyn GLES>, String> {
        fn boxer<T: GLES + 'static>(ctx: T) -> Box<dyn GLES> {
            Box::new(ctx)
        }
        match self {
            Self::GLES1Native => GLES1Native::new(window).map(boxer),
            Self::GLES1OnGL2 => GLES1OnGL2::new(window).map(boxer),
        }
    }
}

#[derive(Default)]
pub struct State {
    /// Current EAGLContext for each thread
    current_ctxs: std::collections::HashMap<crate::ThreadID, Option<crate::objc::id>>,
    /// Which thread's EAGLContext is currently active
    current_ctx_thread: Option<crate::ThreadID>,
}
impl State {
    fn current_ctx_for_thread(&mut self, thread: crate::ThreadID) -> &mut Option<crate::objc::id> {
        self.current_ctxs.entry(thread).or_insert(None);
        self.current_ctxs.get_mut(&thread).unwrap()
    }
}

fn sync_context<'a>(
    state: &mut State,
    objc: &'a mut crate::objc::ObjC,
    window: &mut crate::window::Window,
    current_thread: crate::ThreadID,
) -> &'a mut dyn GLES {
    let current_ctx = state.current_ctx_for_thread(current_thread);
    let host_obj = objc.borrow_mut::<eagl::EAGLContextHostObject>(current_ctx.unwrap());
    let gles_ctx = host_obj.gles_ctx.as_deref_mut().unwrap();

    if window.is_app_gl_ctx_no_longer_current() || state.current_ctx_thread != Some(current_thread)
    {
        log_dbg!(
            "Restoring guest app OpenGL context for thread {}.",
            current_thread
        );
        gles_ctx.make_current(window);
    }

    gles_ctx
}
