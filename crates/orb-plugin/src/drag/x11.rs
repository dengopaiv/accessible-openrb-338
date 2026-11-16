//! The drag on Linux (X11): the source side of the XDND protocol (version 5,
//! <https://www.freedesktop.org/wiki/Specifications/XDND/>) offering the clip as
//! `text/uri-list`, on an X connection and a thread of its own.
//!
//! baseview's window runs its own event loop on its own connection, which would swallow the
//! target's replies, so the drag has a connection of its own with an invisible input-only window
//! as the source window and the `XdndSelection` owner. The pointer is not grabbed: the button's
//! press keeps it with baseview's window (an implicit grab), which therefore also gets the
//! release. The drag follows the pointer by polling (`QueryPointer` every 10 ms): the target is
//! the window under it with `XdndAware` (searched from the root through the children), and the
//! drop happens when the first button is up. Escape does not cancel; a drag ends after two
//! minutes in any case.

use std::error::Error;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConnectionExt as _, CreateWindowAux, EventMask, KeyButMask,
    PropMode, SELECTION_NOTIFY_EVENT, SelectionNotifyEvent, SelectionRequestEvent, Window,
    WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_DEPTH_FROM_PARENT, COPY_FROM_PARENT, NONE};

use crate::editor::guard;

x11rb::atom_manager! {
    Atoms: AtomsCookie {
        XdndAware,
        XdndSelection,
        XdndEnter,
        XdndPosition,
        XdndStatus,
        XdndLeave,
        XdndDrop,
        XdndFinished,
        XdndActionCopy,
        TARGETS,
        UriList: b"text/uri-list",
        Stamp: b"_ORB_DRAG_TIME",
    }
}

/// The XDND version orb speaks.
const VERSION: u32 = 5;
/// How often the pointer is polled.
const POLL: Duration = Duration::from_millis(10);
/// The longest drag.
const LIMIT: Duration = Duration::from_secs(120);
/// How long the drop waits for the target's `XdndFinished`.
const FINISH: Duration = Duration::from_secs(5);

type Res<T> = Result<T, Box<dyn Error>>;

/// Starts the drag of `path` on a thread of its own. `false` when no thread could be made.
pub fn start(path: PathBuf) -> bool {
    std::thread::Builder::new()
        .name("orb pattern drag".into())
        .spawn(move || {
            if let Ok(Err(e)) = guard("drag", || run(&path).map_err(|e| e.to_string())) {
                nice_plug::nice_warn!("the pattern drag failed: {e}");
            }
        })
        .is_ok()
}

/// The `file://` URI list of `path`, percent-encoded.
pub fn uri_list(path: &Path) -> String {
    let mut s = String::from("file://");
    for &b in path.as_os_str().as_bytes() {
        if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
            s.push(b as char);
        } else {
            s.push_str(&format!("%{b:02X}"));
        }
    }
    s.push_str("\r\n");
    s
}

/// The drag's state.
struct Drag {
    conn: RustConnection,
    root: Window,
    /// The source window.
    win: Window,
    atoms: Atoms,
    /// The server time of the drag's start (the selection's and the drop's).
    time: u32,
    uri: String,
    /// The target under the pointer and its version.
    target: Option<(Window, u32)>,
    /// The target accepts the drop (its last `XdndStatus`).
    accepted: bool,
    /// An `XdndPosition` waits for its `XdndStatus`.
    waiting: bool,
    /// The last position sent.
    sent: Option<u32>,
    finished: bool,
}

fn run(path: &Path) -> Res<()> {
    let (conn, screen) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen].root;
    let atoms = Atoms::new(&conn)?.reply()?;
    let win = conn.generate_id()?;
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,
        win,
        root,
        -10,
        -10,
        1,
        1,
        0,
        WindowClass::INPUT_ONLY,
        COPY_FROM_PARENT,
        &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
    )?;
    // a server time: the notification of a property change carries one
    conn.change_property8(PropMode::REPLACE, win, atoms.Stamp, AtomEnum::STRING, b"")?;
    conn.flush()?;
    let time = loop {
        if let Event::PropertyNotify(e) = conn.wait_for_event()? {
            break e.time;
        }
    };
    conn.set_selection_owner(win, atoms.XdndSelection, time)?;
    let mut d = Drag {
        conn,
        root,
        win,
        atoms,
        time,
        uri: uri_list(path),
        target: None,
        accepted: false,
        waiting: false,
        sent: None,
        finished: false,
    };
    let r = d.track();
    let _ = d.conn.destroy_window(win);
    let _ = d.conn.flush();
    r
}

impl Drag {
    /// Follows the pointer until the button is up, then drops or leaves.
    fn track(&mut self) -> Res<()> {
        let start = Instant::now();
        loop {
            while let Some(e) = self.conn.poll_for_event()? {
                self.event(e)?;
            }
            let p = self.conn.query_pointer(self.root)?.reply()?;
            if u16::from(p.mask) & u16::from(KeyButMask::BUTTON1) == 0 || start.elapsed() > LIMIT {
                return self.release();
            }
            let target = self.target_at(p.root_x, p.root_y);
            if target.map(|t| t.0) != self.target.map(|t| t.0) {
                self.leave()?;
                self.target = target;
                if let Some((t, v)) = target {
                    let data = [self.win, v.min(VERSION) << 24, self.atoms.UriList, 0, 0];
                    self.send(t, self.atoms.XdndEnter, data)?;
                }
            }
            if let Some((t, _)) = self.target {
                let pos = (p.root_x as u16 as u32) << 16 | p.root_y as u16 as u32;
                if !self.waiting && self.sent != Some(pos) {
                    let data = [self.win, 0, pos, self.time, self.atoms.XdndActionCopy];
                    self.send(t, self.atoms.XdndPosition, data)?;
                    self.waiting = true;
                    self.sent = Some(pos);
                }
            }
            self.conn.flush()?;
            std::thread::sleep(POLL);
        }
    }

    /// The button is up: the drop when the target accepts, else the target is left.
    fn release(&mut self) -> Res<()> {
        let Some((t, _)) = self.target else {
            return Ok(());
        };
        if !self.accepted {
            return self.leave();
        }
        self.send(t, self.atoms.XdndDrop, [self.win, 0, self.time, 0, 0])?;
        self.conn.flush()?;
        // the target asks for the selection, then says it has finished
        let end = Instant::now() + FINISH;
        while !self.finished && Instant::now() < end {
            while let Some(e) = self.conn.poll_for_event()? {
                self.event(e)?;
            }
            self.conn.flush()?;
            std::thread::sleep(POLL);
        }
        Ok(())
    }

    /// Leaves the current target, if any.
    fn leave(&mut self) -> Res<()> {
        if let Some((t, _)) = self.target.take() {
            self.send(t, self.atoms.XdndLeave, [self.win, 0, 0, 0, 0])?;
        }
        self.accepted = false;
        self.waiting = false;
        self.sent = None;
        Ok(())
    }

    fn send(&self, to: Window, kind: u32, data: [u32; 5]) -> Res<()> {
        let e = ClientMessageEvent::new(32, to, kind, data);
        self.conn.send_event(false, to, EventMask::NO_EVENT, e)?;
        Ok(())
    }

    /// The `XdndAware` window under the root point `(x, y)` and its version: the first aware
    /// window from the root down through the children under the point.
    fn target_at(&self, x: i16, y: i16) -> Option<(Window, u32)> {
        let mut w = self.root;
        loop {
            let r = self
                .conn
                .translate_coordinates(self.root, w, x, y)
                .ok()?
                .reply()
                .ok()?;
            if r.child == NONE {
                return None;
            }
            w = r.child;
            let aware = self
                .conn
                .get_property(false, w, self.atoms.XdndAware, AtomEnum::ATOM, 0, 1)
                .ok()?
                .reply()
                .ok()?;
            if let Some(v) = aware.value32().and_then(|mut v| v.next()) {
                return Some((w, v));
            }
        }
    }

    fn event(&mut self, e: Event) -> Res<()> {
        match e {
            Event::ClientMessage(m) => {
                let d = m.data.as_data32();
                let from = self.target.map(|t| t.0);
                if m.type_ == self.atoms.XdndStatus && Some(d[0]) == from {
                    self.accepted = d[1] & 1 != 0;
                    self.waiting = false;
                } else if m.type_ == self.atoms.XdndFinished {
                    self.finished = true;
                }
            }
            Event::SelectionRequest(r) => self.answer(&r)?,
            _ => {}
        }
        Ok(())
    }

    /// The target asks for the selection: the URI list, or the list of targets.
    fn answer(&self, r: &SelectionRequestEvent) -> Res<()> {
        let mut property = if r.property == NONE {
            r.target
        } else {
            r.property
        };
        if r.selection != self.atoms.XdndSelection {
            property = NONE;
        } else if r.target == self.atoms.UriList {
            self.conn.change_property8(
                PropMode::REPLACE,
                r.requestor,
                property,
                self.atoms.UriList,
                self.uri.as_bytes(),
            )?;
        } else if r.target == self.atoms.TARGETS {
            self.conn.change_property32(
                PropMode::REPLACE,
                r.requestor,
                property,
                AtomEnum::ATOM,
                &[self.atoms.TARGETS, self.atoms.UriList],
            )?;
        } else {
            property = NONE;
        }
        let e = SelectionNotifyEvent {
            response_type: SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: r.time,
            requestor: r.requestor,
            selection: r.selection,
            target: r.target,
            property,
        };
        self.conn
            .send_event(false, r.requestor, EventMask::NO_EVENT, e)?;
        self.conn.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_uri_list_is_percent_encoded() {
        assert_eq!(
            uri_list(Path::new("/tmp/OpenRB-338/clips/1-2-0/303-1 A1.mid")),
            "file:///tmp/OpenRB-338/clips/1-2-0/303-1%20A1.mid\r\n"
        );
    }
}
