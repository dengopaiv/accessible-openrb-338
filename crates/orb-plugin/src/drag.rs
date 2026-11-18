//! The pattern drag-out's native drag (task-203, `docs/plugin.md` section 12): the operating
//! system's drag of a clip file ([`crate::clip`]) out of the editor window, which egui and
//! baseview cannot start.
//!
//! The panel asks for a drag inside its egui frame ([`orb_ui::embed::Embed::drag_pattern`]); the
//! editor's embed writes the clip and leaves its path in [`Pending`], and the editor takes it
//! after the frame and calls [`start`] with the window. Each platform starts the drag once the
//! current event is over, as the resize corner's request (`editor::HostRoute`): baseview's
//! window state is borrowed for the whole frame, and a drag runs the event loop.
//!
//! - macOS ([`macos`]): `-[NSView beginDraggingSessionWithItems:event:source:]` with the file's
//!   URL, posted to the main queue.
//! - Windows ([`windows`]): OLE's `DoDragDrop` with a data object of the file (`CF_HDROP`), posted
//!   with a thread timer.
//! - Linux ([`x11`]): the XDND protocol from a connection and a thread of its own.
//!
//! The drag takes the mouse from the window: at its end the window is sent the left button's
//! release it did not get (macOS, Windows; X11's window keeps the pointer and gets it).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use raw_window_handle::{HasWindowHandle, RawWindowHandle};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;
#[cfg(all(unix, not(target_os = "macos")))]
mod x11;

/// A clip waiting for its drag: the editor's embed puts its file, the editor takes it after the
/// frame.
#[derive(Clone, Default)]
pub struct Pending(Arc<Mutex<Option<PathBuf>>>);

impl Pending {
    pub fn put(&self, path: PathBuf) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(path);
    }

    pub fn take(&self) -> Option<PathBuf> {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).take()
    }
}

/// Starts the drag of the file `path` from `window` once the current event is over. `false`
/// when it cannot (no handle, another windowing system, nothing to post to).
pub fn start(window: &impl HasWindowHandle, path: PathBuf) -> bool {
    let Ok(h) = window.window_handle() else {
        return false;
    };
    match h.as_raw() {
        #[cfg(target_os = "macos")]
        RawWindowHandle::AppKit(h) => macos::start(h.ns_view.as_ptr(), path),
        #[cfg(windows)]
        RawWindowHandle::Win32(h) => windows::start(h.hwnd.get() as *mut std::ffi::c_void, path),
        #[cfg(all(unix, not(target_os = "macos")))]
        RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_) => x11::start(path),
        _ => {
            let _ = path;
            false
        }
    }
}

/// The editor closes: drags posted and not begun are dropped (Windows kills their timers; see
/// `editor::cancel_posts`).
pub fn editor_closed() {
    #[cfg(windows)]
    windows::cancel();
}
