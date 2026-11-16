// OpenRB-338 patch (task-201): the class of the observer layer is registered at run time under a
// name unique to the loaded image, with `ClassBuilder`, instead of `define_class!` with the
// fixed name "RawWindowMetalLayer". Two images that link this crate (the VST3 and the CLAP
// bundle of one plugin, or two plugins) in one process each register their own class; with
// the fixed name the second registration panicked ("could not create new class
// \"RawWindowMetalLayer\", perhaps a class with that name already exists?"), inside the host's
// call to open the editor, which aborted the host. The behavior of the layer is the original's,
// except that the key-value observer ignores an unexpected change instead of panicking. This
// file is a rewrite (define_class! -> ClassBuilder), not a minimal diff of the original.
// See OpenRB-338's docs/plugin.md, "The vendored raw-window-metal".

use core::ffi::c_void;
use std::boxed::Box;
use std::ffi::CString;
use std::format;
use std::sync::OnceLock;

use objc2::rc::{Retained, Weak};
use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, MessageReceiver, Sel};
use objc2::{msg_send, sel, ClassType};
use objc2_foundation::{
    ns_string, NSDictionary, NSKeyValueChangeKey, NSKeyValueChangeNewKey,
    NSKeyValueObservingOptions, NSNumber, NSObjectNSKeyValueObserverRegistration, NSString,
    NSValue,
};
use objc2_quartz_core::{CALayer, CAMetalLayer};

/// A `CAMetalLayer` layer that will automatically update its bounds and scale factor to match
/// its super layer.
///
/// We do this by subclassing, to allow the user to just store the layer as
/// `Retained<CAMetalLayer>`, and still have our observers work.
///
/// See the documentation on Key-Value Observing for details on how this works in general:
/// <https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/KeyValueObserving/KeyValueObserving.html>
///
/// The instances are `CAMetalLayer`s of the subclass [`class`]; its instance variable `root`
/// holds a `Box<Weak<CALayer>>` (the super layer; boxed, as a weak reference must not move).
pub(crate) struct ObserverLayer;

/// The name of the instance variable holding the root layer.
const ROOT: &core::ffi::CStr = c"root";

/// The subclass of `CAMetalLayer`, registered once per loaded image.
fn class() -> &'static AnyClass {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();
    CLASS.get_or_init(|| {
        // The address of this static differs between images loaded at the same time; the
        // counter skips a name left by an image unloaded from the same address.
        let image = &CLASS as *const _ as usize;
        let mut n = 0u32;
        loop {
            let name = CString::new(format!("RawWindowMetalLayer_{image:x}_{n}"))
                .expect("no NUL in the class name");
            if let Some(builder) = ClassBuilder::new(&name, CAMetalLayer::class()) {
                return build(builder);
            }
            n += 1;
        }
    })
}

fn build(mut builder: ClassBuilder) -> &'static AnyClass {
    builder.add_ivar::<*mut c_void>(ROOT);
    let observe: unsafe extern "C-unwind" fn(
        *mut CAMetalLayer,
        Sel,
        *mut NSString,
        *mut AnyObject,
        *mut NSDictionary<NSKeyValueChangeKey, AnyObject>,
        *mut c_void,
    ) = observe_value;
    let dealloc: unsafe extern "C-unwind" fn(*mut CAMetalLayer, Sel) = dealloc;
    // SAFETY: The signatures are the methods' (`NSKeyValueObserving`, `NSObject`).
    unsafe {
        builder.add_method(
            sel!(observeValueForKeyPath:ofObject:change:context:),
            observe,
        );
        builder.add_method(sel!(dealloc), dealloc);
    }
    builder.register()
}

/// The `root` instance variable of `layer`, an instance of [`class`].
fn root_slot(layer: &CAMetalLayer) -> *mut *mut c_void {
    let obj: &AnyObject = layer;
    let ivar = class()
        .instance_variable(ROOT)
        .expect("the observer layer class has its root variable");
    // SAFETY: `layer` is an instance of `class`, whose ivar `root` is a `*mut c_void`.
    unsafe { ivar.load_ptr::<*mut c_void>(obj) }
}

/// The root layer an observer layer follows, while it lives.
fn root_layer(layer: &CAMetalLayer) -> Option<Retained<CALayer>> {
    // SAFETY: The slot holds null or a `Box<Weak<CALayer>>` set in `ObserverLayer::new`.
    let p = unsafe { *root_slot(layer) } as *const Weak<CALayer>;
    if p.is_null() {
        None
    } else {
        unsafe { &*p }.load()
    }
}

/// `-[dealloc]`: what the original's `Drop` did, then the superclass's `dealloc`.
unsafe extern "C-unwind" fn dealloc(this: *mut CAMetalLayer, _cmd: Sel) {
    // SAFETY: `this` is a live instance of `class` until the superclass's `dealloc`.
    let layer = unsafe { &*this };
    let slot = root_slot(layer);
    // SAFETY: The slot holds null or a `Box<Weak<CALayer>>` set in `ObserverLayer::new`.
    let p = unsafe { *slot } as *mut Weak<CALayer>;
    if !p.is_null() {
        // SAFETY: Taken once; the slot is cleared.
        let root = unsafe { Box::from_raw(p) };
        unsafe { *slot = core::ptr::null_mut() };
        // It is possible for the root layer to be de-allocated before the custom layer.
        //
        // In that case, the observer is already de-registered, and so we don't need to do
        // anything.
        //
        // We use a weak variable here to avoid issues if the layer was removed from the super
        // layer, and then later de-allocated, without de-registering these observers.
        if let Some(root_layer) = root.load() {
            // SAFETY: The observer is registered for these key paths in `new`.
            unsafe {
                root_layer.removeObserver_forKeyPath(layer, ns_string!("contentsScale"));
                root_layer.removeObserver_forKeyPath(layer, ns_string!("bounds"));
            }
        }
        drop(root);
    }
    // SAFETY: `dealloc` of the superclass, once.
    unsafe { MessageReceiver::send_super_message(this, CAMetalLayer::class(), sel!(dealloc), ()) }
}

impl ObserverLayer {
    /// The context pointer, to differentiate between key-value observing registered by this class,
    /// and the superclass.
    fn context() -> *mut c_void {
        class() as *const AnyClass as *mut c_void
    }

    /// Create a new custom layer that tracks parameters from the given super layer.
    pub fn new(root_layer: &CALayer) -> Retained<CAMetalLayer> {
        // SAFETY: `class` is a subclass of `CAMetalLayer`; initializing it is safe.
        let this: Retained<CAMetalLayer> = unsafe { msg_send![class(), new] };
        let root = Box::into_raw(Box::new(Weak::new(root_layer)));
        // SAFETY: The slot of a new instance; freed in `dealloc`.
        unsafe { *root_slot(&this) = root as *mut c_void };

        // Add the layer as a sublayer of the root layer.
        root_layer.addSublayer(&this);

        // Do not use auto-resizing mask.
        //
        // This is done to work around a bug in macOS 14 and above, where views using auto layout
        // may end up setting fractional values as the bounds, and that in turn doesn't propagate
        // properly through the auto-resizing mask and with contents gravity.
        //
        // Instead, we keep the bounds of the layer in sync with the root layer using an observer,
        // see below.
        //
        // this.setAutoresizingMask(kCALayerHeightSizable | kCALayerWidthSizable);

        // AppKit / UIKit automatically sets the correct scale factor and bounds for layers attached
        // to a view. Our layer, however, is not directly attached to a view, and so we need to
        // observe changes to the root layer's parameters, and apply them to our layer.
        //
        // Note the use of `NSKeyValueObservingOptionInitial` to also set the initial values here.
        //
        // SAFETY: Observer deregistered in `dealloc` before the observer object is deallocated.
        unsafe {
            root_layer.addObserver_forKeyPath_options_context(
                &this,
                ns_string!("contentsScale"),
                NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Initial,
                ObserverLayer::context(),
            );
            root_layer.addObserver_forKeyPath_options_context(
                &this,
                ns_string!("bounds"),
                NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Initial,
                ObserverLayer::context(),
            );
        }

        // The default content gravity (`kCAGravityResize`) is a fine choice for most applications,
        // as it masks / alleviates issues with resizing and behaves better when moving the window
        // between monitors, so we won't modify that.

        this
    }

    /// The root layer of an observer layer (tests).
    #[allow(dead_code)]
    pub(crate) fn root(layer: &CAMetalLayer) -> Option<Retained<CALayer>> {
        root_layer(layer)
    }
}

/// `-[observeValueForKeyPath:ofObject:change:context:]`. Unlike the original it does not
/// panic on an unexpected change (a panic here would unwind into AppKit): it ignores it.
unsafe extern "C-unwind" fn observe_value(
    this: *mut CAMetalLayer,
    _cmd: Sel,
    key_path: *mut NSString,
    object: *mut AnyObject,
    change: *mut NSDictionary<NSKeyValueChangeKey, AnyObject>,
    context: *mut c_void,
) {
    // SAFETY: The receiver of a method is a live instance.
    let this = unsafe { &*this };
    // An unrecognized context must belong to the super class.
    if context != ObserverLayer::context() {
        // SAFETY: The signature is correct, and it's safe to forward to the superclass' method
        // when we're overriding the method.
        let _: () = unsafe {
            msg_send![
                super(this, CAMetalLayer::class()),
                observeValueForKeyPath: key_path,
                ofObject: object,
                change: change,
                context: context,
            ]
        };
        return;
    }

    // SAFETY: Valid or null objects, as KVO passes them.
    let key_path: Option<&NSString> = unsafe { key_path.as_ref() };
    let Some(change) = (unsafe { change.as_ref() }) else {
        return;
    };
    // SAFETY: The static is declared with the correct type in `objc2`.
    let key = unsafe { NSKeyValueChangeNewKey };
    let Some(new) = change.objectForKey(key) else {
        return;
    };

    // NOTE: Setting these values usually causes a quarter second animation to occur, which is
    // undesirable.
    //
    // However, since we're setting them inside an observer, there already is a transaction
    // ongoing, and as such we don't need to wrap this in a `CATransaction` ourselves.

    if key_path == Some(ns_string!("contentsScale")) {
        // `contentsScale` is a CGFloat, and so the observed value is always a NSNumber.
        if let Ok(new) = new.downcast::<NSNumber>() {
            // Set the scale factor of the layer to match the root layer when it changes (e.g. if
            // moved to a different monitor, or monitor settings changed).
            this.setContentsScale(new.as_cgfloat());
        }
    } else if key_path == Some(ns_string!("bounds")) {
        // `bounds` is a CGRect, and so the observed value is always a NSValue.
        if let Some(bounds) = new.downcast::<NSValue>().ok().and_then(|v| v.get_rect()) {
            // Set `bounds` and `position` so that the new layer is inside the superlayer.
            //
            // This differs from just setting the `bounds`, as it also takes into account any
            // translation that the superlayer may have that we'd want to preserve.
            this.setFrame(bounds);
        }
    }
}

#[cfg(test)]
mod tests {
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};

    use super::*;

    #[test]
    fn release_order_does_not_matter() {
        let root_layer = CALayer::new();
        let layer = ObserverLayer::new(&root_layer);
        drop(root_layer);
        drop(layer);

        let root_layer = CALayer::new();
        let layer = ObserverLayer::new(&root_layer);
        assert!(ObserverLayer::root(&layer).is_some());
        drop(layer);
        drop(root_layer);
    }

    #[test]
    fn scale_factor_propagates() {
        let root_layer = CALayer::new();
        let layer = ObserverLayer::new(&root_layer);

        root_layer.setContentsScale(3.0);
        assert_eq!(layer.contentsScale(), 3.0);
    }

    #[test]
    fn bounds_propagates() {
        let root_layer = CALayer::new();
        let layer = ObserverLayer::new(&root_layer);

        root_layer.setBounds(CGRect::new(
            CGPoint::new(10.0, 20.0),
            CGSize::new(30.0, 40.0),
        ));
        assert_eq!(layer.position(), CGPoint::new(25.0, 40.0));
        assert_eq!(
            layer.bounds(),
            CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(30.0, 40.0),)
        );
    }

    #[test]
    fn superlayer_can_remove_all_sublayers() {
        let root_layer = CALayer::new();
        let layer = ObserverLayer::new(&root_layer);
        layer.removeFromSuperlayer();
        drop(layer);
        root_layer.setContentsScale(3.0);
    }

    #[test]
    fn the_class_name_is_not_the_fixed_one() {
        let name = class().name().to_string_lossy().into_owned();
        assert!(name.starts_with("RawWindowMetalLayer_"), "{name}");
    }
}
