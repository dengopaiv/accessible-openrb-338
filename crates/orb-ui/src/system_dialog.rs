//! The system's file and folder dialogs of screen reader mode (task-194, docs/accessibility.md
//! 3.6): with the mode on, Open, Save As, Export and Mods > Choose file use the operating
//! system's dialog (which screen readers read) instead of the in-app browser, and "Set Songs
//! Folder..." / "Set Mods Folder..." of the Options menu choose the folders the browser's "Use as
//! ... folder" buttons set.
//!
//! The dialog itself is behind [`Picker`], so tests give the app a fake one; the default is
//! [`system_picker`] over `rfd`. The dialog is modal: the thread that calls it (the UI thread,
//! inside a frame) waits for the answer, so the frame loop stands still meanwhile. The audio
//! runs on its own thread (orb-player's stream), so a song keeps playing.

use std::path::{Path, PathBuf};

use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle,
};

use eframe::egui::{
    self, Align, Color32, Context, Frame, Id, Key, Layout, Margin, Modal, Modifiers, RichText,
    Stroke,
};

use crate::app::App;
use crate::browser::{ExportFormat, Mode};
use crate::palette::{INK, INK_DIM, SLATE_DARK, SLATE_LINE};

/// What a dialog is asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// A file: an `.rbs` song, a `.rbm` mod, or a file to write.
    File(Mode),
    /// A folder, for the songs folder (`Songs`) or the mods folder (`Mods`) setting.
    Folder(Folder),
}

/// The two folder settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folder {
    Songs,
    Mods,
}

impl Folder {
    pub fn title(self) -> &'static str {
        match self {
            Folder::Songs => "Set Songs Folder",
            Folder::Mods => "Set Mods Folder",
        }
    }
}

/// The app's window as the owner of a dialog: the raw handles eframe gives each frame
/// (`eframe::Frame`). With an owner the app is disabled behind the dialog and the focus comes
/// back to it when the dialog closes.
#[derive(Clone, Copy)]
pub struct Parent {
    window: RawWindowHandle,
    display: RawDisplayHandle,
}

impl Parent {
    /// The handles of `frame`'s window (None when it has none, or not yet).
    pub fn of(frame: &eframe::Frame) -> Option<Self> {
        Some(Self {
            window: frame.window_handle().ok()?.as_raw(),
            display: frame.display_handle().ok()?.as_raw(),
        })
    }
}

impl HasWindowHandle for Parent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // SAFETY: the handle is the app's own window, refreshed every frame, which outlives the
        // dialog: the dialog is modal over the UI thread that owns the window.
        Ok(unsafe { WindowHandle::borrow_raw(self.window) })
    }
}

impl HasDisplayHandle for Parent {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // SAFETY: as for the window handle.
        Ok(unsafe { DisplayHandle::borrow_raw(self.display) })
    }
}

/// Shows a dialog for the request, starting in the folder, owned by the window `parent` (None:
/// no owner), and returns the chosen path (None: cancelled).
pub type Picker = Box<dyn FnMut(Request, &Path, Option<&Parent>) -> Option<PathBuf>>;

/// The dialog of an app that has none to show (`App::new`): cancels every request, so only
/// the real program (`App::use_system_services`) opens a dialog.
pub fn no_picker() -> Picker {
    Box::new(|_, _, _| None)
}

/// The operating system's dialog.
pub fn system_picker() -> Picker {
    Box::new(|request, start, parent| {
        let mut dialog = rfd::FileDialog::new().set_directory(start);
        if let Some(p) = parent {
            dialog = dialog.set_parent(p);
        }
        match request {
            Request::File(Mode::OpenSong) => dialog
                .set_title("Open song")
                .add_filter("ReBirth song (*.rbs)", &["rbs"])
                .pick_file(),
            Request::File(Mode::SaveSong) => dialog
                .set_title("Save song")
                .add_filter("ReBirth song (*.rbs)", &["rbs"])
                .save_file(),
            Request::File(Mode::Export) => {
                let mut dialog = dialog.set_title("Export audio");
                for f in ExportFormat::ALL {
                    let ext = f.extension();
                    dialog = dialog
                        .add_filter(format!("{} audio (*.{ext})", ext.to_uppercase()), &[ext]);
                }
                dialog.save_file()
            }
            Request::File(Mode::ChooseMod) => dialog
                .set_title("Choose mod")
                .add_filter("ReBirth mod (*.rbm)", &["rbm"])
                .pick_file(),
            Request::Folder(f) => dialog.set_title(f.title()).pick_folder(),
        }
    })
}

/// The path a writing dialog returned, with the extension the browser would give a typed name:
/// a song gets `.rbs` (unless it has it), an export keeps `.wav`, `.flac` or `.ogg` and gets
/// `.wav` when it has none; any other extension of an export is an error.
pub fn with_extension(mode: Mode, path: PathBuf) -> Result<PathBuf, String> {
    let ext = path.extension().and_then(|e| e.to_str());
    match mode {
        Mode::SaveSong => {
            if ext.is_some_and(|e| e.eq_ignore_ascii_case("rbs")) {
                Ok(path)
            } else {
                let mut s = path.into_os_string();
                s.push(".rbs");
                Ok(PathBuf::from(s))
            }
        }
        Mode::Export => match (ext, ExportFormat::from_extension(ext)) {
            (_, Some(_)) => Ok(path),
            (None, _) => Ok(path.with_extension("wav")),
            (Some(_), None) => Err(format!(
                "{}: the extension must be .wav, .flac or .ogg.",
                path.display()
            )),
        },
        Mode::OpenSong | Mode::ChooseMod => Ok(path),
    }
}

/// A file the system dialog did not confirm: the dialog was asked about the name it returned,
/// and `with_extension` gave it another that is a file already (`a.txt` became `a.txt.rbs`).
/// The app asks before it writes (decision task-203).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replace {
    pub mode: Mode,
    pub path: PathBuf,
    /// Whether the No button has been given the focus (once, on the first frame of the box,
    /// so that Tab is not fought).
    pub focused: bool,
}

impl App {
    /// Whether the question "Replace ...?" is open.
    pub fn replacing(&self) -> bool {
        self.replace.is_some()
    }

    /// The answer of the question: Yes writes (the same path as an answer of the dialog), No
    /// writes nothing and ends a Save As that the unsaved-changes prompt's Save started.
    pub fn answer_replace(&mut self, yes: bool) {
        let Some(r) = self.replace.take() else {
            return;
        };
        if yes {
            self.finish_pick(r.mode, r.path);
        } else {
            self.after_save = None;
        }
    }

    /// Draws the question and takes its keys: Y answers Yes; N, Esc and Enter answer No (the
    /// safe answer is the default, as in the browser's overwrite question). The No button has
    /// the focus when the box opens.
    pub fn replace_ui(&mut self, ctx: &Context) {
        let Some(r) = self.replace.clone() else {
            return;
        };
        let key = |k: Key| ctx.input_mut(|i| i.consume_key(Modifiers::NONE, k));
        let mut answer = None;
        let mut focus_given = r.focused;
        if key(Key::Y) {
            answer = Some(true);
        } else if key(Key::N) || key(Key::Escape) || key(Key::Enter) {
            answer = Some(false);
        }
        let name = r
            .path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let frame = Frame::new()
            .fill(SLATE_DARK)
            .stroke(Stroke::new(1.0, SLATE_LINE))
            .inner_margin(Margin::same(12))
            .corner_radius(3.0);
        Modal::new(Id::unique("orb_replace"))
            .frame(frame)
            .backdrop_color(Color32::from_black_alpha(170))
            .show(ctx, |ui| {
                ui.set_width(320.0);
                // the Dialog node egui makes of a Modal gets the question as its name, so that
                // the screen reader reads it when the focus enters
                let dialog = ui
                    .stack()
                    .iter()
                    .find(|f| f.kind() == Some(egui::UiKind::Modal))
                    .map(|f| f.unique_id);
                if let Some(id) = dialog {
                    let question = format!("Replace {name}?");
                    ui.ctx().accesskit_node_builder(id, |b| {
                        b.set_label(question);
                    });
                }
                ui.label(
                    RichText::new(format!("Replace {name}?"))
                        .color(INK)
                        .strong()
                        .size(15.0),
                );
                ui.label(
                    RichText::new(format!(
                        "{} is a file already, and the file dialog did not ask about it: \
                         it was given the extension of the name.",
                        r.path.display()
                    ))
                    .color(INK_DIM),
                );
                ui.add_space(8.0);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let no = ui.button("No");
                    // once, on the first frame: whatever held the focus before (a menu item, a
                    // panel control) gives it up, so that the screen reader reads the box
                    if !focus_given {
                        no.request_focus();
                        focus_given = true;
                    }
                    if no.clicked() {
                        answer = Some(false);
                    }
                    if ui.button("Yes").clicked() {
                        answer = Some(true);
                    }
                });
            });
        if let Some(replace) = &mut self.replace {
            replace.focused = focus_given;
        }
        if let Some(yes) = answer {
            self.answer_replace(yes);
        }
    }
}
