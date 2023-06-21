/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
//! The `NSCharacterSet` class cluster, including `NSMutableCharacterSet`.

use super::ns_string;
use crate::objc::{
    autorelease, id, msg, msg_class, objc_classes, retain, ClassExports, HostObject, NSZonePtr,
};
use std::collections::HashSet;

/// Belongs to _touchHLE_NSCharacterSet
struct CharacterSetHostObject {
    set: HashSet<u16>,
}
impl HostObject for CharacterSetHostObject {}

pub const CLASSES: ClassExports = objc_classes! {

(env, this, _cmd);

// NSCharacterSet is an abstract class. A subclass must provide:
// - (bool)characterIsMember:(unichar)character
// We can pick whichever subclass we want for the various alloc methods.
// For the time being, that will always be _touchHLE_NSCharacterSet.
@implementation NSCharacterSet: NSObject

+ (id)allocWithZone:(NSZonePtr)zone {
    // NSCharacterSet might be subclassed by something which needs
    // allocWithZone: to have the normal behaviour. Unimplemented: call
    // superclass alloc then.
    assert!(this == env.objc.get_known_class("NSCharacterSet", &mut env.mem));
    msg_class![env; _touchHLE_NSCharacterSet allocWithZone:zone]
}

// This doesn't have a corresponding init method for some reason.
+ (id)characterSetWithCharactersInString:(id)string { // NSString*
    let mut set = HashSet::new();
    ns_string::for_each_code_unit(env, string, |_idx, c| { set.insert(c); });

    let new: id = msg![env; this alloc];
    env.objc.borrow_mut::<CharacterSetHostObject>(new).set = set;

    autorelease(env, new).await;

    new
}

// NSCopying implementation
- (id)copyWithZone:(NSZonePtr)_zone {
    // TODO: override this once we have NSMutableCharacterSet!
    retain(env, this).await
}

@end

// Our private subclass that is the single implementation of NSCharacterSet for the
// time being.
@implementation _touchHLE_NSCharacterSet: NSCharacterSet

+ (id)allocWithZone:(NSZonePtr)_zone {
    let host_object = Box::new(CharacterSetHostObject {
        set: HashSet::new(),
    });
    env.objc.alloc_object(this, host_object, &mut env.mem)
}

// TODO: initWithCoder:

- (bool)characterIsMember:(u16)code_unit {
    env.objc.borrow::<CharacterSetHostObject>(this).set.contains(&code_unit)
}

@end

};
