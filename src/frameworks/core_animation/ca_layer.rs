/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! `CALayer`.

use crate::frameworks::core_graphics::{CGPoint, CGRect, CGSize};
use crate::objc::{id, msg, nil, objc_classes, release, retain, ClassExports, HostObject};

pub(super) struct CALayerHostObject {
    /// Possibly nil, usually a UIView. This is a weak reference.
    delegate: id,
    /// Sublayers in back-to-front order. These are strong references.
    pub(super) sublayers: Vec<id>,
    /// The superlayer. This is a weak reference.
    superlayer: id,
    pub(super) bounds: CGRect,
    pub(super) position: CGPoint,
    pub(super) anchor_point: CGPoint,
    pub(super) hidden: bool,
    pub(super) opaque: bool,
    pub(super) opacity: f32,
    pub(super) background_color: id,
    /// `CGImageRef*`
    pub(super) contents: id,
    /// For CAEAGLLayer only
    pub(super) drawable_properties: id,
    /// For CAEAGLLayer only (internal state for compositor)
    pub(super) presented_pixels: Option<(Vec<u8>, u32, u32)>,
    /// Internal state for compositor
    pub(super) gles_texture: Option<crate::gles::gles11_raw::types::GLuint>,
    /// Internal state for compositor
    pub(super) gles_texture_is_up_to_date: bool,
}
impl HostObject for CALayerHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

@implementation CALayer: NSObject

+ (id)alloc {
    let host_object = Box::new(CALayerHostObject {
        delegate: nil,
        sublayers: Vec::new(),
        superlayer: nil,
        bounds: CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize { width: 0.0, height: 0.0 }
        },
        position: CGPoint { x: 0.0, y: 0.0 },
        anchor_point: CGPoint { x: 0.5, y: 0.5 },
        hidden: false,
        opaque: false,
        opacity: 1.0,
        background_color: nil, // transparency
        contents: nil,
        drawable_properties: nil,
        presented_pixels: None,
        gles_texture: None,
        gles_texture_is_up_to_date: false,
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

+ (id)layer {
    let new_layer: id = msg![env; this alloc];
    msg![env; new_layer init]
}

- (())dealloc {
    let &mut CALayerHostObject {
        drawable_properties,
        contents,
        superlayer,
        ref mut sublayers,
        ..
    } = env.objc.borrow_mut(this);
    let sublayers = std::mem::take(sublayers);

    if drawable_properties != nil {
        release(env, drawable_properties);
    }

    if contents != nil {
        release(env, contents);
    }

    assert!(superlayer == nil);
    for sublayer in sublayers {
        env.objc.borrow_mut::<CALayerHostObject>(sublayer).superlayer = nil;
        release(env, sublayer);
    }
}

- (id)delegate {
    env.objc.borrow::<CALayerHostObject>(this).delegate
}
- (())setDelegate:(id)delegate {
    env.objc.borrow_mut::<CALayerHostObject>(this).delegate = delegate;
}

- (id)superlayer {
    env.objc.borrow::<CALayerHostObject>(this).superlayer
}
// TODO: sublayers accessors

- (())addSublayer:(id)layer {
    if env.objc.borrow::<CALayerHostObject>(layer).superlayer == this {
        () = msg![env; this bringSublayerToFront:layer];
    } else {
        retain(env, layer);
        () = msg![env; layer removeFromSuperlayer];
        env.objc.borrow_mut::<CALayerHostObject>(layer).superlayer = this;
        env.objc.borrow_mut::<CALayerHostObject>(this).sublayers.push(layer);
    }
}

- (())removeFromSuperlayer {
    let CALayerHostObject { ref mut superlayer, .. } = env.objc.borrow_mut(this);
    let superlayer = std::mem::take(superlayer);
    if superlayer == nil {
        return;
    }

    let CALayerHostObject { ref mut sublayers, .. } = env.objc.borrow_mut(superlayer);
    let idx = sublayers.iter().position(|&sublayer| sublayer == this).unwrap();
    let sublayer = sublayers.remove(idx);
    assert!(sublayer == this);
    release(env, this);
}

- (CGRect)bounds {
    env.objc.borrow::<CALayerHostObject>(this).bounds
}
- (())setBounds:(CGRect)bounds {
    env.objc.borrow_mut::<CALayerHostObject>(this).bounds = bounds;
}
- (CGPoint)position {
    env.objc.borrow::<CALayerHostObject>(this).position
}
- (())setPosition:(CGPoint)position {
    env.objc.borrow_mut::<CALayerHostObject>(this).position = position;
}
- (CGPoint)anchorPoint {
    env.objc.borrow::<CALayerHostObject>(this).anchor_point
}
- (())setAnchorPoint:(CGPoint)anchor_point {
    env.objc.borrow_mut::<CALayerHostObject>(this).anchor_point = anchor_point;
}

- (CGRect)frame {
    let &CALayerHostObject {
        bounds,
        position,
        anchor_point,
        ..
    } = env.objc.borrow(this);
    CGRect {
        origin: CGPoint {
            x: position.x - bounds.size.width * anchor_point.x,
            y: position.y - bounds.size.height * anchor_point.y,
        },
        size: bounds.size,
    }
}
- (())setFrame:(CGRect)frame {
    let CALayerHostObject {
        bounds,
        position,
        anchor_point,
        ..
    } = env.objc.borrow_mut(this);
    *position = CGPoint {
        x: frame.origin.x + frame.size.width * anchor_point.x,
        y: frame.origin.y + frame.size.height * anchor_point.y,
    };
    *bounds = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: frame.size,
    };
}

- (bool)isHidden {
    env.objc.borrow::<CALayerHostObject>(this).hidden
}
- (())setHidden:(bool)hidden {
    env.objc.borrow_mut::<CALayerHostObject>(this).hidden = hidden;
}

- (bool)isOpaque {
    env.objc.borrow::<CALayerHostObject>(this).opaque
}
- (())setOpaque:(bool)opaque {
    env.objc.borrow_mut::<CALayerHostObject>(this).opaque = opaque;
}

- (f32)opacity {
    env.objc.borrow::<CALayerHostObject>(this).opacity
}
- (())setOpacity:(f32)opacity {
    env.objc.borrow_mut::<CALayerHostObject>(this).opacity = opacity;
}

// See remarks in ui_view.rs about the type of this property
- (id)backgroundColor {
    env.objc.borrow::<CALayerHostObject>(this).background_color
}
- (())setBackgroundColor:(id)color {
    env.objc.borrow_mut::<CALayerHostObject>(this).background_color = color;
}

// CGImageRef*
- (id)contents {
    env.objc.borrow::<CALayerHostObject>(this).contents
}
- (())setContents:(id)new_contents {
    let host_obj = env.objc.borrow_mut::<CALayerHostObject>(this);
    host_obj.gles_texture_is_up_to_date = false;
    let old_contents = std::mem::replace(&mut host_obj.contents, new_contents);
    retain(env, new_contents);
    release(env, old_contents);
}

// TODO: more

@end

};
