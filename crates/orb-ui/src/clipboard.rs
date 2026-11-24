//! Where Edit > Paste Pattern and the pattern editors' Paste read the clipboard text from, and
//! what a Copy also writes (task-287).
//!
//! A copy always goes out as `egui::Context::copy_text`, which eframe and egui-baseview write
//! to the system clipboard. A direct read is only needed where egui gives the text no other
//! way: the plugin (egui-baseview reads the clipboard on a paste key only, and a plugin leaves
//! keys to the host) and the pattern editors' Paste buttons. No test may touch the real
//! clipboard, so an [`App`](crate::app::App) starts with [`Clipboard::Memory`], a clipboard of
//! its own that tests share between two apps; the standalone binary and the plugin's editor
//! switch to [`Clipboard::System`].

use std::sync::{Arc, Mutex};

/// The clipboard a direct read takes the text from.
#[derive(Clone, Debug)]
pub enum Clipboard {
    /// The operating system's (arboard). Not for tests.
    System,
    /// A text kept in memory: every copy stores it, a read returns it. Clones share it.
    Memory(Arc<Mutex<Option<String>>>),
}

impl Clipboard {
    /// An empty memory clipboard.
    pub fn memory() -> Self {
        Clipboard::Memory(Arc::default())
    }

    /// The clipboard's text; `None` when it holds no text or cannot be read.
    pub fn read(&self) -> Option<String> {
        match self {
            Clipboard::System => arboard::Clipboard::new().ok()?.get_text().ok(),
            Clipboard::Memory(m) => m.lock().ok()?.clone(),
        }
    }

    /// Notes a copy of `text`: a memory clipboard stores it; the system's is written by
    /// `copy_text`.
    pub fn copied(&self, text: &str) {
        if let Clipboard::Memory(m) = self
            && let Ok(mut m) = m.lock()
        {
            *m = Some(text.to_string());
        }
    }
}
