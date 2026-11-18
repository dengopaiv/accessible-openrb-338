//! The drag on macOS: `-[NSView beginDraggingSessionWithItems:event:source:]` from baseview's
//! view, with one `NSDraggingItem` whose pasteboard writer is the clip's file URL (an `NSURL`),
//! as a file dragged from the Finder; hosts take the file from the pasteboard.
//!
//! - The session is begun from a block on the main queue (after the egui frame), with a
//!   left-mouse-down event made at the pointer (`+[NSEvent mouseEventWithType:...]`), as JUCE does
//!   for its external drags: the event that the frame saw is gone by then. Nothing is begun when
//!   the button is already up.
//! - The source is an instance of a class registered at run time under a name unique to the loaded
//!   image (`ClassBuilder`, as the vendored raw-window-metal; two images of the plugin may be
//!   loaded in one process, `docs/plugin.md` section 10), answering copy for any destination.
//! - The session swallows the mouse-up: when it ends, the source sends the view a made-up
//!   `mouseUp:`, so that egui sees the button's release.
//!
//! Objective-C exceptions are not caught (an AppKit exception aborts as elsewhere in the
//! process); Rust panics are (`editor::guard`).

use std::cell::RefCell;
use std::ffi::{CString, c_void};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use objc2::encode::{Encode, Encoding};
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, AnyProtocol, ClassBuilder, NSObject, Sel};
use objc2::{ClassType, class, msg_send, sel};

use crate::editor::{guard, is_main_thread};

/// `NSPoint` (`CGPoint`; `CGFloat` is `f64` on 64-bit macOS).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Point {
    x: f64,
    y: f64,
}

// SAFETY: the layout and encoding of `CGPoint`.
unsafe impl Encode for Point {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[f64::ENCODING, f64::ENCODING]);
}

/// `NSSize` (`CGSize`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Size {
    w: f64,
    h: f64,
}

// SAFETY: the layout and encoding of `CGSize`.
unsafe impl Encode for Size {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[f64::ENCODING, f64::ENCODING]);
}

/// `NSRect` (`CGRect`).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Rect {
    origin: Point,
    size: Size,
}

// SAFETY: the layout and encoding of `CGRect`.
unsafe impl Encode for Rect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[Point::ENCODING, Size::ENCODING]);
}

/// `NSEventType`: left mouse down and up.
const LEFT_MOUSE_DOWN: usize = 1;
const LEFT_MOUSE_UP: usize = 2;
/// `NSDragOperationCopy`.
const DRAG_COPY: usize = 1;
/// The side of the dragged icon, points.
const ICON: f64 = 32.0;

/// A drag posted to the main queue.
struct Job {
    view: Retained<AnyObject>,
    path: PathBuf,
}

/// Posts the drag of `path` from baseview's view `view` to the main queue. `false` when this is
/// not the main thread (the window's) or there is no view.
pub fn start(view: *mut c_void, path: PathBuf) -> bool {
    unsafe extern "C" {
        static _dispatch_main_q: c_void;
        fn dispatch_async_f(
            queue: *const c_void,
            context: *mut c_void,
            work: extern "C" fn(*mut c_void),
        );
    }
    extern "C" fn run(context: *mut c_void) {
        // SAFETY: the context is the `Box<Job>` posted below, run once.
        let job = unsafe { Box::from_raw(context.cast::<Job>()) };
        if let Ok(Err(e)) = guard("drag", || begin(&job.view, &job.path)) {
            nice_plug::nice_warn!("the pattern drag did not start: {e}");
        }
    }
    if !is_main_thread() {
        return false;
    }
    // SAFETY: baseview's live `NSView`; retained until the job has run.
    let Some(view) = (unsafe { Retained::retain(view.cast::<AnyObject>()) }) else {
        return false;
    };
    let job = Box::into_raw(Box::new(Job { view, path }));
    // SAFETY: `_dispatch_main_q` is the main queue (`dispatch_get_main_queue()`); the context
    // is the job, taken back by `run`.
    unsafe { dispatch_async_f(&raw const _dispatch_main_q, job.cast(), run) };
    true
}

/// An `NSString` of `s`.
fn ns_string(s: &str) -> Result<Retained<AnyObject>, String> {
    let c = CString::new(s).map_err(|_| "a NUL in the path")?;
    // SAFETY: a NUL-terminated UTF-8 string, copied by the call.
    let s: Option<Retained<AnyObject>> =
        unsafe { msg_send![class!(NSString), stringWithUTF8String: c.as_ptr()] };
    s.ok_or_else(|| "no NSString".into())
}

/// A made-up left-button event of type `kind` at `at` (window coordinates) in `window`.
///
/// # Safety
/// `window` is an `NSWindow`.
unsafe fn mouse_event(window: &AnyObject, kind: usize, at: Point) -> Option<Retained<AnyObject>> {
    unsafe {
        let info: Retained<AnyObject> = msg_send![class!(NSProcessInfo), processInfo];
        let time: f64 = msg_send![&*info, systemUptime];
        let number: isize = msg_send![window, windowNumber];
        let nil: Option<&AnyObject> = None;
        msg_send![
            class!(NSEvent),
            mouseEventWithType: kind,
            location: at,
            modifierFlags: 0usize,
            timestamp: time,
            windowNumber: number,
            context: nil,
            eventNumber: 0isize,
            clickCount: 1isize,
            pressure: 1.0f32
        ]
    }
}

/// Begins the dragging session of the file `path` from `view` (on the main thread).
fn begin(view: &AnyObject, path: &Path) -> Result<(), String> {
    let path = path.to_str().ok_or("a path that is not UTF-8")?;
    // SAFETY: AppKit's documented methods on the main thread, with arguments of their types;
    // `view` is an `NSView`.
    unsafe {
        let buttons: usize = msg_send![class!(NSEvent), pressedMouseButtons];
        if buttons & 1 == 0 {
            return Err("the button is up already".into());
        }
        let window: Option<Retained<AnyObject>> = msg_send![view, window];
        let window = window.ok_or("the editor has no window")?;
        let ns_path = ns_string(path)?;
        let url: Option<Retained<AnyObject>> = msg_send![class!(NSURL), fileURLWithPath: &*ns_path];
        let url = url.ok_or("no file URL")?;
        let item: objc2::rc::Allocated<AnyObject> = msg_send![class!(NSDraggingItem), alloc];
        let item: Option<Retained<AnyObject>> = msg_send![item, initWithPasteboardWriter: &*url];
        let item = item.ok_or("no dragging item")?;
        // the file's icon under the pointer
        let at: Point = msg_send![&*window, mouseLocationOutsideOfEventStream];
        let nil: Option<&AnyObject> = None;
        let p: Point = msg_send![view, convertPoint: at, fromView: nil];
        let workspace: Retained<AnyObject> = msg_send![class!(NSWorkspace), sharedWorkspace];
        let icon: Option<Retained<AnyObject>> = msg_send![&*workspace, iconForFile: &*ns_path];
        let frame = Rect {
            origin: Point {
                x: p.x - ICON / 2.0,
                y: p.y - ICON / 2.0,
            },
            size: Size { w: ICON, h: ICON },
        };
        let _: () = msg_send![&*item, setDraggingFrame: frame, contents: icon.as_deref()];
        let items: Retained<AnyObject> = msg_send![class!(NSArray), arrayWithObject: &*item];
        let event = mouse_event(&window, LEFT_MOUSE_DOWN, at).ok_or("no mouse event")?;
        let source = new_source(view)?;
        let session: Option<Retained<AnyObject>> = msg_send![
            view,
            beginDraggingSessionWithItems: &*items,
            event: &*event,
            source: &*source
        ];
        if session.is_none() {
            forget_view(&source);
            return Err("no dragging session".into());
        }
        Ok(())
    }
}

/// The name of the source's instance variable: the view, retained (`*mut AnyObject`).
const VIEW: &std::ffi::CStr = c"orbView";

thread_local! {
    /// The source of the last drag, kept alive past its session (one at a time).
    static SOURCE: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

/// A new dragging source for a drag from `view`, kept in [`SOURCE`].
fn new_source(view: &AnyObject) -> Result<Retained<AnyObject>, String> {
    // SAFETY: `new` of an `NSObject` subclass without instance initialization of its own.
    let source: Retained<AnyObject> = unsafe { msg_send![source_class(), new] };
    let ivar = source_class()
        .instance_variable(VIEW)
        .ok_or("the source class has no view variable")?;
    // SAFETY: the ivar is a `*mut c_void` of the class; the view is retained into it.
    unsafe {
        *ivar.load_ptr::<*mut c_void>(&source) = Retained::into_raw(
            Retained::retain(view as *const AnyObject as *mut AnyObject).ok_or("no view")?,
        )
        .cast();
    }
    let old = SOURCE.with(|s| s.borrow_mut().replace(source.clone()));
    if let Some(old) = old {
        forget_view(&old);
    }
    Ok(source)
}

/// Takes the view out of a source's instance variable (the caller then owns its retain).
fn take_view(source: &AnyObject) -> Option<Retained<AnyObject>> {
    let ivar = source_class().instance_variable(VIEW)?;
    // SAFETY: an instance of the source class; the ivar holds null or a retained view.
    unsafe {
        let slot = ivar.load_ptr::<*mut c_void>(source);
        let p = std::mem::replace(&mut *slot, std::ptr::null_mut());
        Retained::from_raw(p.cast::<AnyObject>())
    }
}

/// Releases a source's view.
fn forget_view(source: &AnyObject) {
    drop(take_view(source));
}

/// The dragging source class, registered once per loaded image under a name of its own.
fn source_class() -> &'static AnyClass {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();
    CLASS.get_or_init(|| {
        let image = &CLASS as *const _ as usize;
        let mut n = 0u32;
        loop {
            let name = CString::new(format!("OrbClipDragSource_{image:x}_{n}"))
                .expect("no NUL in the class name");
            if let Some(mut b) = ClassBuilder::new(&name, NSObject::class()) {
                b.add_ivar::<*mut c_void>(VIEW);
                if let Some(p) = AnyProtocol::get(c"NSDraggingSource") {
                    b.add_protocol(p);
                }
                let mask: Mask = operation_mask;
                let ended: Ended = ended;
                // SAFETY: the signatures are `NSDraggingSource`'s.
                unsafe {
                    b.add_method(
                        sel!(draggingSession:sourceOperationMaskForDraggingContext:),
                        mask,
                    );
                    b.add_method(sel!(draggingSession:endedAtPoint:operation:), ended);
                }
                return b.register();
            }
            n += 1;
        }
    })
}

/// The source's two methods.
type Mask = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, isize) -> usize;
type Ended = unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject, Point, usize);

/// `-draggingSession:sourceOperationMaskForDraggingContext:`: copy, inside the app and outside.
unsafe extern "C-unwind" fn operation_mask(
    _this: *mut AnyObject,
    _cmd: Sel,
    _session: *mut AnyObject,
    _context: isize,
) -> usize {
    DRAG_COPY
}

/// `-draggingSession:endedAtPoint:operation:`: the view gets the button's release that the
/// session took.
unsafe extern "C-unwind" fn ended(
    this: *mut AnyObject,
    _cmd: Sel,
    _session: *mut AnyObject,
    _at: Point,
    _operation: usize,
) {
    let _ = guard("drag end", || {
        // SAFETY: the receiver, an instance of the source class.
        let Some(this) = (unsafe { this.as_ref() }) else {
            return;
        };
        let Some(view) = take_view(this) else {
            return;
        };
        // SAFETY: AppKit's methods on the main thread; `view` is an `NSView`.
        unsafe {
            let window: Option<Retained<AnyObject>> = msg_send![&*view, window];
            let Some(window) = window else {
                return;
            };
            let at: Point = msg_send![&*window, mouseLocationOutsideOfEventStream];
            if let Some(e) = mouse_event(&window, LEFT_MOUSE_UP, at) {
                let _: () = msg_send![&*view, mouseUp: &*e];
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What runs without a window: the source class (one per image, the two methods, the
    /// protocol), the view kept and given back, the strings and the file URL. (Debug builds
    /// check the messages' encodings.) The session itself is the user's check in a host.
    #[test]
    fn the_source_class_and_the_file_url() {
        let class = source_class();
        assert!(std::ptr::eq(class, source_class()), "registered once");
        assert!(
            class
                .name()
                .to_str()
                .unwrap()
                .starts_with("OrbClipDragSource_")
        );
        let view: Retained<AnyObject> = unsafe { msg_send![NSObject::class(), new] };
        let source = new_source(&view).unwrap();
        // SAFETY: the class's own methods, with their argument types.
        unsafe {
            for sel in [
                sel!(draggingSession:sourceOperationMaskForDraggingContext:),
                sel!(draggingSession:endedAtPoint:operation:),
            ] {
                let yes: bool = msg_send![&*source, respondsToSelector: sel];
                assert!(yes, "{sel:?}");
            }
            let nil: Option<&AnyObject> = None;
            let op: usize = msg_send![
                &*source,
                draggingSession: nil,
                sourceOperationMaskForDraggingContext: 1isize
            ];
            assert_eq!(op, DRAG_COPY);
        }
        if let Some(p) = AnyProtocol::get(c"NSDraggingSource") {
            assert!(class.conforms_to(p));
        }
        let back = take_view(&source).expect("the view");
        assert!(std::ptr::eq(&*back, &*view));
        assert!(take_view(&source).is_none(), "given back once");
        let path = ns_string("/tmp/OpenRB-338/clips/1-2-0/303-1 A1.mid").unwrap();
        // SAFETY: Foundation's methods with their argument types.
        unsafe {
            let url: Retained<AnyObject> = msg_send![class!(NSURL), fileURLWithPath: &*path];
            let s: Retained<AnyObject> = msg_send![&*url, absoluteString];
            let c: *const std::ffi::c_char = msg_send![&*s, UTF8String];
            assert_eq!(
                std::ffi::CStr::from_ptr(c).to_str().unwrap(),
                "file:///tmp/OpenRB-338/clips/1-2-0/303-1%20A1.mid"
            );
        }
        assert!(ns_string("a\0b").is_err());
    }
}
