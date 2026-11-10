//! Which key events are the numpad's (task-225, the user's decision of 2026-11-10: the fork
//! builds for Windows only, so it may ask Windows).
//!
//! egui 0.37 names a numpad key like the main-keyboard key it shares a name with (`keys.rs`).
//! winit's message hook ([`hook`], installed in `main.rs` through
//! `NativeOptions::event_loop_builder`) sees every `WM_KEYDOWN` / `WM_KEYUP` with its virtual
//! key before winit turns it into an event: `VK_NUMPAD0..9`, `VK_ADD`, `VK_SUBTRACT`,
//! `VK_DIVIDE`, `VK_DECIMAL`, `VK_MULTIPLY`, and `VK_RETURN` with the extended-key bit (bit 24
//! of `lParam`) for the numpad's Enter. It queues what egui will call the key; `App::keyboard`
//! claims the entry of each egui key event ([`claim`]: same key, same press or release), and
//! drops what is left at the end of its frame ([`end_frame`]). The hook runs on the same thread
//! as the window procedure, in the same message pump, so the entries of a frame's events are
//! in the queue before the frame runs. Numpad `*` has no egui key at all: [`take_multiply`].
//!
//! The queue is only the transport of the hook's knowledge; the keys' meaning is
//! `keys::map`'s `numpad` flag. Without the hook (another platform) the queue stays empty and
//! no key is a numpad key.

use std::collections::VecDeque;
use std::sync::Mutex;

use eframe::egui::Key;

/// A numpad key event the hook saw: the key egui names it (`None`: numpad `*`) and whether it
/// went down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    key: Option<Key>,
    pressed: bool,
}

/// More entries than a frame can hold are dropped (no frame runs: a minimized window).
const MAX: usize = 64;

static QUEUE: Mutex<VecDeque<Entry>> = Mutex::new(VecDeque::new());

fn push(key: Option<Key>, pressed: bool) {
    if let Ok(mut q) = QUEUE.lock() {
        if q.len() >= MAX {
            q.pop_front();
        }
        q.push_back(Entry { key, pressed });
    }
}

/// The key egui names the numpad key with the virtual key `vk` (`extended`: the extended-key
/// bit of `lParam`). `Some(None)`: numpad `*`. `None`: not a numpad key.
pub fn key_of_vk(vk: u32, extended: bool) -> Option<Option<Key>> {
    Some(Some(match vk {
        0x60 => Key::Num0,
        0x61 => Key::Num1,
        0x62 => Key::Num2,
        0x63 => Key::Num3,
        0x64 => Key::Num4,
        0x65 => Key::Num5,
        0x66 => Key::Num6,
        0x67 => Key::Num7,
        0x68 => Key::Num8,
        0x69 => Key::Num9,
        0x6A => return Some(None),
        0x6B => Key::Plus,
        0x6D => Key::Minus,
        0x6E => Key::Period,
        0x6F => Key::Slash,
        0x0D if extended => Key::Enter,
        _ => return None,
    }))
}

/// Notes a keyboard message: `message` is `WM_KEYDOWN`, `WM_KEYUP` (or the `WM_SYS` pair),
/// `wparam` its virtual key, `lparam` its flags. Other messages are ignored.
pub fn note_message(message: u32, wparam: usize, lparam: isize) {
    let pressed = match message {
        0x0100 | 0x0104 => true,
        0x0101 | 0x0105 => false,
        _ => return,
    };
    let extended = lparam & (1 << 24) != 0;
    if let Some(key) = u32::try_from(wparam)
        .ok()
        .and_then(|vk| key_of_vk(vk, extended))
    {
        // auto-repeat of `*` (bit 30: the key was down): the app ignores it, as for any key
        if key.is_none() && pressed && lparam & (1 << 30) != 0 {
            return;
        }
        push(key, pressed);
    }
}

/// The prefix of the Win32 `MSG` the hook reads (same layout, `#[repr(C)]`).
#[cfg(windows)]
#[repr(C)]
struct Msg {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
}

/// The message hook for winit's event loop (`EventLoopBuilderExtWindows::with_msg_hook`): the
/// pointer is a `MSG`. It never consumes a message.
#[cfg(windows)]
pub fn hook(msg: *const std::ffi::c_void) -> bool {
    if !msg.is_null() {
        // SAFETY: winit passes a pointer to the `MSG` it has just taken from the queue; the
        // fields read are its first four, as in `windows-sys`' `MSG`.
        let m = unsafe { &*(msg as *const Msg) };
        note_message(m.message, m.wparam, m.lparam);
    }
    false
}

/// Takes the numpad `*` presses and releases (`true`: down) the hook saw since the last call,
/// in order.
pub fn take_multiply() -> Vec<bool> {
    let Ok(mut q) = QUEUE.lock() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    q.retain(|e| {
        if e.key.is_none() {
            out.push(e.pressed);
            false
        } else {
            true
        }
    });
    out
}

/// Whether the egui key event `key` (pressed or released) is the numpad's: the oldest entry of
/// the same key and direction is taken, with the older entries before it (their events did not
/// reach egui).
///
/// `key` is the key the app uses (the physical one, else the logical one) and `has_physical`
/// whether the event had a physical key.
pub fn claim(key: Key, has_physical: bool, pressed: bool) -> bool {
    let Ok(mut q) = QUEUE.lock() else {
        return false;
    };
    // egui-winit has no physical key for the numpad's `.`: only an event without a physical
    // key can be it, and it carries the logical key, a period or (on a layout with a decimal
    // comma, as Estonian) a comma. The main keyboard's comma has the physical key Comma.
    let same = |e: &Entry| {
        e.pressed == pressed
            && if e.key == Some(Key::Period) {
                !has_physical && matches!(key, Key::Period | Key::Comma)
            } else {
                e.key == Some(key)
            }
    };
    match q.iter().position(same) {
        Some(i) => {
            q.drain(..=i);
            true
        }
        None => false,
    }
}

/// The end of a frame: what no event claimed is dropped.
pub fn end_frame() {
    if let Ok(mut q) = QUEUE.lock() {
        q.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The queue is global: one test only touches it.
    #[test]
    fn the_hook_tags_numpad_events_and_pairs_press_and_release() {
        end_frame();
        // numpad - down and up, numpad Enter (extended), the main Enter (not), numpad *
        note_message(0x0100, 0x6D, 0);
        note_message(0x0101, 0x6D, 0);
        note_message(0x0100, 0x0D, 1 << 24);
        note_message(0x0100, 0x0D, 0);
        note_message(0x0100, 0x6A, 0);
        note_message(0x0101, 0x6A, 0);
        note_message(0x0200, 0x6D, 0); // WM_MOUSEMOVE: ignored
        assert_eq!(take_multiply(), vec![true, false]);
        assert!(claim(Key::Minus, true, true));
        assert!(!claim(Key::Minus, true, true), "taken once");
        assert!(claim(Key::Minus, true, false));
        assert!(claim(Key::Enter, true, true), "the numpad's Enter");
        assert!(
            !claim(Key::Enter, true, true),
            "the main Enter is not queued"
        );
        assert!(!claim(Key::Num5, true, true));
        // a tap the frame did not claim does not tag the next one
        note_message(0x0100, 0x65, 0);
        end_frame();
        assert!(!claim(Key::Num5, true, true));
        // an event that never reached egui is skipped over
        note_message(0x0100, 0x66, 0);
        note_message(0x0100, 0x67, 0);
        assert!(claim(Key::Num7, true, true));
        assert!(!claim(Key::Num6, true, true));
        // numpad `.`: a period, or a comma on a decimal-comma layout
        note_message(0x0100, 0x6E, 0);
        assert!(claim(Key::Comma, false, true));
        note_message(0x0100, 0x6E, 0);
        assert!(claim(Key::Period, false, true));
        note_message(0x0100, 0x6E, 0);
        assert!(!claim(Key::Comma, false, false), "the other direction");
        // the main keyboard's comma has a physical key and does not take the numpad's `.`
        assert!(!claim(Key::Comma, true, true));
        assert!(!claim(Key::Period, true, true));
        end_frame();
        // a frame the keyboard does not run in drops the entries too (App::update calls
        // end_frame whatever the gate says), so a stale `*` does not fire later
        note_message(0x0100, 0x6A, 0);
        end_frame();
        assert!(take_multiply().is_empty());
        // auto-repeat of `*` is not a new press
        note_message(0x0100, 0x6A, 1 << 30);
        assert!(take_multiply().is_empty());
    }

    #[test]
    fn the_virtual_keys() {
        for (vk, key) in [
            (0x60, Key::Num0),
            (0x69, Key::Num9),
            (0x6B, Key::Plus),
            (0x6D, Key::Minus),
            (0x6E, Key::Period),
            (0x6F, Key::Slash),
        ] {
            assert_eq!(key_of_vk(vk, false), Some(Some(key)));
        }
        assert_eq!(key_of_vk(0x6A, false), Some(None));
        assert_eq!(key_of_vk(0x0D, true), Some(Some(Key::Enter)));
        assert_eq!(key_of_vk(0x0D, false), None);
        assert_eq!(key_of_vk(0xBD, false), None, "VK_OEM_MINUS, the main key");
    }
}
