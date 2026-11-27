//! The 303 pattern editors (task-274, docs/ui_spec.md section 23): two floating windows over the
//! panel, "303-1 Pattern Editor" and "303-2 Pattern Editor" (task-290), each fixed to its 303,
//! that edit one 303 pattern on a step grid with a 13-key piano roll and the Up, Down, Accent
//! and Slide rows. It edits only what the song stores for a 303 pattern (the 98
//! bytes of `docs/RBS42.txt`'s "303 " chunk: the shuffle flag, the length and 16 steps of note,
//! slide, accent, up, down and rest).
//!
//! Every change is one whole-pattern write through [`App::write_pattern`], the path of the Edit
//! menu's pattern commands and of the 303's Clear (`crate::link::Link::set_pattern`: the link's
//! pattern mirror, msg 41 sub 44 to the engine, the panel's step views; the song is marked
//! modified and the one-level Undo holds the former bytes). A drag is one change: its writes
//! keep the Undo of its first.
//!
//! The pure functions over a pattern's bytes ([`with_note`], [`shifted`], [`transposed`] and
//! the like) are the edits; [`Act`] is what the window asks for, run by [`App::editor_act`].
//! The window's frame (its placing, the header's controls, the grid's geometry and painting) is
//! shared with the drum editors (`crate::editor_frame`, task-280).

use egui::{Color32, Context, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};

use crate::app::App;
use crate::editor_frame::{
    self as frame, ARROW, CELL_H, CELL_W, GAP, HEAD_H, KEY_W, LABEL_SIZE, Number, Placing, Press,
    SPLIT_H, cell_rect, dimmed, paint_columns,
};
pub use crate::editor_frame::{STEPS, step_pattern};
use crate::palette::{AMBER, BG, INK, INK_DIM, SLATE_DARK, SLATE_LINE};

/// The pitches of the piano roll: C to the C an octave above (note bytes 0..=12).
pub const PITCHES: u8 = 13;
/// The bytes of a 303 pattern: the shuffle flag, the length, 16 steps of 6 bytes.
pub const PATTERN_BYTES: usize = 98;

/// Where step `s`'s six bytes start (note, slide, accent, up, down, rest).
fn step_at(s: usize) -> usize {
    2 + 6 * s
}

/// The note fields of a step.
const NOTE: usize = 0;
const REST: usize = 5;

/// A flag row of the editor, top to bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flag {
    Up,
    Down,
    Accent,
    Slide,
}

impl Flag {
    pub const ALL: [Flag; 4] = [Flag::Up, Flag::Down, Flag::Accent, Flag::Slide];

    /// The step byte the flag is (docs/song_load.md section 4.6: 1 slide, 2 accent, 3 up, 4
    /// down).
    pub fn field(self) -> usize {
        match self {
            Flag::Up => 3,
            Flag::Down => 4,
            Flag::Accent => 2,
            Flag::Slide => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Flag::Up => "Up",
            Flag::Down => "Down",
            Flag::Accent => "Accent",
            Flag::Slide => "Slide",
        }
    }
}

/// The pattern's length as the editor uses it: 1..=16 (a stored 0 or more than 16 is clamped).
pub fn length(bytes: &[u8]) -> usize {
    bytes[1].clamp(1, STEPS as u8) as usize
}

/// The pitch step `s` plays, `None` for a pause. A pause's stored pitch is not shown.
pub fn note(bytes: &[u8], s: usize) -> Option<u8> {
    let b = &bytes[step_at(s)..];
    (b[REST] == 0).then_some(b[NOTE])
}

/// Whether flag `f` of step `s` is on.
pub fn flag(bytes: &[u8], s: usize, f: Flag) -> bool {
    bytes[step_at(s) + f.field()] != 0
}

/// `bytes` with step `s` a note at `pitch` (rest off, the flags kept).
pub fn with_note(bytes: &[u8], s: usize, pitch: u8) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[step_at(s) + NOTE] = pitch;
    b[step_at(s) + REST] = 0;
    b
}

/// `bytes` with step `s` a pause: rest on, the stored pitch and the flags kept.
pub fn with_pause(bytes: &[u8], s: usize) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[step_at(s) + REST] = 1;
    b
}

/// `bytes` with flag `f` of step `s` set to `on`.
pub fn with_flag(bytes: &[u8], s: usize, f: Flag, on: bool) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[step_at(s) + f.field()] = u8::from(on);
    b
}

/// `bytes` with its length `n` (1..=16).
pub fn with_length(bytes: &[u8], n: u8) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[1] = n.clamp(1, STEPS as u8);
    b
}

/// `bytes` with its shuffle flag `on`.
pub fn with_shuffle(bytes: &[u8], on: bool) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[0] = u8::from(on);
    b
}

/// `bytes` with the steps rotated by one step, right (later) or left, wrapping within the
/// pattern's length; the steps beyond the length stay.
pub fn shifted(bytes: &[u8], right: bool) -> Vec<u8> {
    frame::shifted(bytes, 6, right)
}

/// `bytes` with every step's pitch moved by one semitone, up or down, wrapping over the 13
/// rows (12 up is 0, 0 down is 12). Pauses move too (their stored pitch); the Up and Down
/// flags stay. A stored pitch above 12 (not one of the rows) is left alone.
pub fn transposed(bytes: &[u8], up: bool) -> Vec<u8> {
    let mut b = bytes.to_vec();
    for s in 0..STEPS {
        let n = &mut b[step_at(s) + NOTE];
        if *n < PITCHES {
            *n = if up {
                (*n + 1) % PITCHES
            } else {
                (*n + PITCHES - 1) % PITCHES
            };
        }
    }
    b
}

/// Whether `pitch` (0 = C) is a black key.
pub fn is_black(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

/// The key's name: C, C#, ... B, C.
pub fn pitch_name(pitch: u8) -> &'static str {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    NAMES[(pitch % 12) as usize]
}

/// The steps a drag from step `s0` at `p0` to step `s1` at `p1` crosses after `s0`, each at the
/// pitch on the straight line between the two (the line is followed when the mouse skips
/// steps between two frames). The same step gives `[(s1, p1)]`.
pub fn crossed(s0: usize, p0: u8, s1: usize, p1: u8) -> Vec<(usize, u8)> {
    if s0 == s1 {
        return vec![(s1, p1)];
    }
    let n = s0.abs_diff(s1);
    (1..=n)
        .map(|i| {
            let s = if s1 > s0 { s0 + i } else { s0 - i };
            let p = f64::from(p0) + (f64::from(p1) - f64::from(p0)) * i as f64 / n as f64;
            (s, p.round() as u8)
        })
        .collect()
}

/// What the editor's window asks for; [`App::editor_act`] runs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// A bank button or the pattern number: show this pattern (0..=31) of the editor's 303,
    /// selected on the panel too, in both modes.
    Select(u8),
    Copy,
    Paste,
    Clear,
    /// The Length value; `drag` while its value is dragged (a drag is one change).
    Length {
        n: u8,
        drag: bool,
    },
    Shuffle,
    /// Shift right (later) or left.
    Shift {
        right: bool,
    },
    Transpose {
        up: bool,
    },
    /// The mouse went down on the piano roll at step `step`, pitch `pitch`.
    NoteDown {
        step: usize,
        pitch: u8,
    },
    /// The held mouse is over step `step`, pitch `pitch` of the piano roll.
    NoteMove {
        step: usize,
        pitch: u8,
    },
    /// The mouse went down on flag `flag` of step `step`.
    FlagDown {
        flag: Flag,
        step: usize,
    },
    /// The held mouse is over step `step`'s column (the drag stays in its row).
    FlagMove {
        step: usize,
    },
    /// The mouse went up: the end of a drag.
    Release,
}

/// A drag on a grid.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Gesture {
    /// On the piano roll: the last cell painted; `pause` while the press on a lit cell has not
    /// moved (a click there makes the step a pause); `wrote` once the drag changed the pattern.
    Notes {
        last: (usize, u8),
        pause: bool,
        wrote: bool,
    },
    /// On a flag row: the row, the value the first cell set, the last step painted.
    Flags {
        flag: Flag,
        on: bool,
        last: usize,
        wrote: bool,
    },
}

/// An editor's state, kept by the app (one per 303, `App::editors`).
#[derive(Clone, Debug, Default)]
pub struct Editor {
    /// Whether the window is open.
    pub open: bool,
    /// The 303 it edits (0 or 1; fixed, task-290) and the pattern shown (0..=31).
    pub synth: usize,
    pub pattern: u8,
    /// The window's placing and place.
    win: Placing,
    gesture: Option<Gesture>,
    /// A drag of the Length value wrote already (it keeps the Undo of its first change).
    length_wrote: bool,
    /// Whether the mouse was down on the piano roll and on the flag rows at the last frame.
    notes_down: bool,
    flags_down: bool,
    /// Where the window drew its parts last (for tests and the pointer).
    hits: Hits,
}

/// Where the window drew its grids and buttons last.
#[derive(Clone, Debug, Default)]
struct Hits {
    notes: Option<Rect>,
    flags: Option<Rect>,
    buttons: Vec<(&'static str, Rect)>,
    /// The playing step the grid showed, and whether the steps beyond the length were dimmed.
    playing: Option<usize>,
    dimmed: bool,
}

/// The pairs of white keys that meet (E-F, B-high C), lower pitch first: a line on the grid
/// between their rows.
const WHITE_PAIRS: [(u8, u8); 2] = [(4, 5), (11, 12)];

/// The rows of a white and a black key on the grid, from the app's palette (the same in the
/// Classic and Modern skins).
const WHITE_ROW: Color32 = SLATE_DARK;
const BLACK_ROW: Color32 = BG;

/// The fill of the grid row of `pitch`: black-key rows darker than white-key rows.
pub fn row_fill(pitch: u8) -> Color32 {
    if is_black(pitch) {
        BLACK_ROW
    } else {
        WHITE_ROW
    }
}

impl Editor {
    /// The editor of 303 `synth` (0 or 1), closed.
    pub fn new(synth: usize) -> Self {
        Self {
            synth: synth.min(1),
            ..Self::default()
        }
    }
}

/// The window's title of 303 `k` (0 or 1).
pub fn title(k: usize) -> &'static str {
    ["303-1 Pattern Editor", "303-2 Pattern Editor"][k.min(1)]
}

/// The Edit menu's item that opens the editor of 303 `k` (the ellipsis: it opens a window).
pub fn menu_item(k: usize) -> &'static str {
    ["303-1 Pattern Editor...", "303-2 Pattern Editor..."][k.min(1)]
}

/// The salt of the ids of the window of 303 `k` (the window's id is
/// `egui::Id::unique(salt(k))`).
pub fn salt(k: usize) -> &'static str {
    ["orb_pattern_editor_303_1", "orb_pattern_editor_303_2"][k.min(1)]
}

/// What the window shows of the song in a frame.
struct Shown {
    bytes: Vec<u8>,
    synth: usize,
    pattern: u8,
    /// The step the 303 plays in this pattern, if it plays it.
    playing: Option<usize>,
    paste: bool,
}

impl App {
    /// The editor of 303 `k` (0 or 1).
    pub fn pattern_editor(&self, k: usize) -> &Editor {
        &self.editors[k.min(1)]
    }

    /// Edit > 303-1 or 303-2 Pattern Editor... and Cmd/Ctrl+E with that 303 focused: opens the
    /// window of 303 `k` on its selected pattern, or brings it to the front and switches it to
    /// that pattern; the editor takes the focus (`App::editor_focus`). Nothing without a song.
    pub fn open_pattern_editor(&mut self, k: usize) {
        if self.link.is_none() {
            return;
        }
        let k = k.min(1);
        let pattern = self.panel.rows[k].pattern;
        let e = &mut self.editors[k];
        e.pattern = pattern;
        e.win.open(e.open);
        e.open = true;
        e.gesture = None;
        self.editor_focus = Some(k);
    }

    /// Where the window of 303 `k` drew step `step`'s cell at `pitch` of the piano roll last.
    pub fn pattern_editor_note_cell(&self, k: usize, step: usize, pitch: u8) -> Option<Rect> {
        let r = self.pattern_editor(k).hits.notes?;
        Some(cell_rect(r, step, (PITCHES - 1 - pitch) as usize))
    }

    /// Where the window of 303 `k` drew step `step`'s cell of flag row `flag` last.
    pub fn pattern_editor_flag_cell(&self, k: usize, flag: Flag, step: usize) -> Option<Rect> {
        let r = self.pattern_editor(k).hits.flags?;
        let row = Flag::ALL.iter().position(|&f| f == flag)?;
        Some(cell_rect(r, step, row))
    }

    /// Where the window of 303 `k` was drawn last (`None` when closed).
    pub fn pattern_editor_window(&self, k: usize) -> Option<Rect> {
        let e = self.pattern_editor(k);
        e.win.window.filter(|_| e.open)
    }

    /// The playing step the grid of 303 `k` showed last, and whether it dimmed the steps beyond
    /// the pattern's length.
    pub fn pattern_editor_shown(&self, k: usize) -> (Option<usize>, bool) {
        let h = &self.pattern_editor(k).hits;
        (h.playing, h.dimmed)
    }

    /// Where the window of 303 `k` drew its button or value `name` last ("A", "number",
    /// "Copy", "Length", "Shuffle", "shift_left", "transpose_up", ...).
    pub fn pattern_editor_hit(&self, k: usize, name: &str) -> Option<Rect> {
        self.pattern_editor(k)
            .hits
            .buttons
            .iter()
            .find(|(n, _)| *n == name)
            .map(|&(_, r)| r)
    }

    /// The selection rule (docs/ui_spec.md section 23), the drum editors' (task-290), in both
    /// modes: the editor shows its 303's selected pattern (a selection on the panel moves it,
    /// song playback's too; so does the switch of mode).
    pub(crate) fn editor_follow(&mut self, k: usize) {
        self.editors[k].pattern = self.panel.rows[k].pattern;
    }

    /// The pattern of the editor of 303 `k` as the link holds it, after the panel's pending
    /// events were noted (an edit of the panel forgets the Undo first) and sent; `None` without
    /// a song.
    fn editor_bytes(&mut self, k: usize) -> Option<Vec<u8>> {
        self.link.as_ref()?;
        self.note_edits();
        let link = self.link.as_mut()?;
        link.forward(&mut self.panel);
        Some(link.pattern(k, self.editors[k].pattern).to_vec())
    }

    /// Writes `f` of the pattern of the editor of 303 `k` for the command `what`; with `keep` (a
    /// drag that wrote already) the Undo stays the one of the drag's first write. Whether it
    /// changed.
    fn editor_edit(
        &mut self,
        k: usize,
        what: &'static str,
        keep: bool,
        f: impl FnOnce(&[u8]) -> Vec<u8>,
    ) -> bool {
        let Some(before) = self.editor_bytes(k) else {
            return false;
        };
        let p = self.editors[k].pattern;
        let after = f(&before);
        let prev = if keep { self.undo.clone() } else { None };
        let changed = self.write_pattern(k, p, &after, what);
        if changed
            && let Some(u) = prev
            && u.row == k
            && u.pattern == p
        {
            self.undo = Some(u);
        }
        changed
    }

    /// Runs what the window of 303 `k` asked for.
    pub fn editor_act(&mut self, k: usize, a: Act) {
        const EDIT: &str = "Edit Pattern";
        if self.link.is_none() {
            return;
        }
        let k = k.min(1);
        let p = self.editors[k].pattern;
        match a {
            Act::Select(pattern) => self.editor_select(k, pattern),
            Act::Copy => {
                if self.editor_bytes(k).is_some() {
                    self.copy_pattern_at(k, p, false);
                }
            }
            Act::Paste => {
                if self.editor_bytes(k).is_some() {
                    self.paste_pattern_at(k, p);
                }
            }
            Act::Clear => {
                if self.editor_bytes(k).is_some() {
                    self.clear_pattern_at(k, p);
                }
            }
            Act::Length { n, drag } => {
                let keep = drag && self.editors[k].length_wrote;
                let changed = self.editor_edit(k, "Pattern Length", keep, |b| with_length(b, n));
                self.editors[k].length_wrote = drag && (keep || changed);
            }
            Act::Shuffle => {
                self.editor_edit(k, "Pattern Shuffle", false, |b| with_shuffle(b, b[0] == 0));
            }
            Act::Shift { right } => {
                self.editor_edit(k, "Shift Pattern", false, |b| shifted(b, right));
            }
            Act::Transpose { up } => {
                self.editor_edit(k, "Transpose Pattern", false, |b| transposed(b, up));
            }
            Act::NoteDown { step, pitch } => {
                let Some(bytes) = self.editor_bytes(k) else {
                    return;
                };
                let lit = note(&bytes, step) == Some(pitch);
                // a press on the lit cell waits: a click there makes a pause, a drag paints
                let wrote = !lit && self.editor_edit(k, EDIT, false, |b| with_note(b, step, pitch));
                self.editors[k].gesture = Some(Gesture::Notes {
                    last: (step, pitch),
                    pause: lit,
                    wrote,
                });
            }
            Act::NoteMove { step, pitch } => {
                let Some(Gesture::Notes { last, wrote, .. }) = self.editors[k].gesture else {
                    return;
                };
                if last == (step, pitch) {
                    return;
                }
                let cells = crossed(last.0, last.1, step, pitch);
                let changed = self.editor_edit(k, EDIT, wrote, |b| {
                    cells
                        .iter()
                        .fold(b.to_vec(), |b, &(s, n)| with_note(&b, s, n))
                });
                self.editors[k].gesture = Some(Gesture::Notes {
                    last: (step, pitch),
                    pause: false,
                    wrote: wrote || changed,
                });
            }
            Act::FlagDown { flag: f, step } => {
                let Some(bytes) = self.editor_bytes(k) else {
                    return;
                };
                let on = !flag(&bytes, step, f);
                let wrote = self.editor_edit(k, EDIT, false, |b| with_flag(b, step, f, on));
                self.editors[k].gesture = Some(Gesture::Flags {
                    flag: f,
                    on,
                    last: step,
                    wrote,
                });
            }
            Act::FlagMove { step } => {
                let Some(Gesture::Flags {
                    flag: f,
                    on,
                    last,
                    wrote,
                }) = self.editors[k].gesture
                else {
                    return;
                };
                if last == step {
                    return;
                }
                let changed = self.editor_edit(k, EDIT, wrote, |b| {
                    (last.min(step)..=last.max(step))
                        .fold(b.to_vec(), |b, s| with_flag(&b, s, f, on))
                });
                self.editors[k].gesture = Some(Gesture::Flags {
                    flag: f,
                    on,
                    last: step,
                    wrote: wrote || changed,
                });
            }
            Act::Release => {
                if let Some(Gesture::Notes {
                    last: (step, _),
                    pause: true,
                    ..
                }) = self.editors[k].gesture.take()
                {
                    self.editor_edit(k, EDIT, false, |b| with_pause(b, step));
                }
            }
        }
    }

    /// The editor's selector: the pattern is selected on the panel, as a click on its button
    /// selects it (in song mode too, where a click records with record on).
    fn editor_select(&mut self, k: usize, pattern: u8) {
        let pattern = pattern.min(31);
        let e = &mut self.editors[k];
        e.pattern = pattern;
        e.gesture = None;
        if self.panel.rows[k].pattern != pattern {
            self.select_pattern(k, pattern);
        }
    }

    /// Draws the 303 editors' windows that are open, and runs what they ask for.
    pub(crate) fn pattern_editors_ui(&mut self, ctx: &Context) {
        for k in 0..2 {
            self.pattern_editor_ui(ctx, k);
        }
    }

    /// Draws the window of 303 `k` when it is open, and runs what it asks for.
    fn pattern_editor_ui(&mut self, ctx: &Context, k: usize) {
        if !self.editors[k].open {
            return;
        }
        if self.link.is_none() {
            self.editors[k].open = false;
            return;
        }
        self.editor_follow(k);
        let Some(link) = &self.link else {
            return;
        };
        let p = self.editors[k].pattern;
        let shown = Shown {
            bytes: link.pattern(k, p).to_vec(),
            synth: k,
            pattern: p,
            playing: self.panel.playing_step303[k].filter(|_| self.panel.rows[k].pattern == p),
            // the system clipboard is not read every frame: Paste is always enabled, and a text
            // that is no pattern of this device is answered on the status line
            paste: true,
        };
        let saved = if k == 0 {
            self.settings.pattern_editor_303_1_pos
        } else {
            self.settings.pattern_editor_303_2_pos
        };
        let mut acts = Vec::new();
        let mut length_drag = false;
        let mut win = std::mem::take(&mut self.editors[k].win);
        let editor = &mut self.editors[k];
        let spec = frame::Spec {
            id: egui::Id::unique(salt(k)),
            title: title(k),
            saved,
            guess: vec2(480.0, 340.0),
        };
        let out = frame::show_window(ctx, spec, &mut win, |ui| {
            length_drag = window_ui(ui, editor, &shown, &mut acts);
        });
        self.editors[k].win = win;
        if out.pressed {
            self.editor_focus = Some(k);
        }
        if !length_drag {
            self.editors[k].length_wrote = false;
        }
        for a in acts {
            self.editor_act(k, a);
        }
        // the place is saved once the window is let go (the standalone app's only)
        if let Some(pos) = out.save {
            if k == 0 {
                self.settings.set_pattern_editor_303_1_pos(Some(pos));
            } else {
                self.settings.set_pattern_editor_303_2_pos(Some(pos));
            }
            if let Err(e) = self.settings.save() {
                self.fail(e);
            }
        }
        if !out.open {
            self.editors[k].open = false;
            self.editors[k].gesture = None;
            self.editor_unfocus(k);
        }
    }
}

/// The window's content; what it asks for goes to `acts`. Whether the Length value is being
/// dragged.
fn window_ui(ui: &mut Ui, e: &mut Editor, s: &Shown, acts: &mut Vec<Act>) -> bool {
    e.hits.buttons.clear();
    let mut length_drag = false;
    let bank = s.pattern / 8;
    // ---- the top row ----
    ui.horizontal(|ui| {
        frame::compact(ui);
        if let Some(b) = frame::bank_buttons(ui, s.pattern, &mut e.hits.buttons) {
            acts.push(Act::Select(b * 8 + s.pattern % 8));
        }
        // the number between its arrows, which step through the 303's A1..D8 and stop at the
        // two ends (`step_pattern`; each window is fixed to its 303, task-290)
        let prev = step_pattern(s.pattern, false);
        let next = step_pattern(s.pattern, true);
        let n = frame::number_field(
            ui,
            s.pattern,
            prev.is_some(),
            next.is_some(),
            &mut e.hits.buttons,
        );
        let to = match n {
            Some(Number::Prev) => prev,
            Some(Number::Next) => next,
            Some(Number::Set(n)) => Some(bank * 8 + n),
            None => None,
        };
        if let Some(pattern) = to {
            acts.push(Act::Select(pattern));
        }
        ui.separator();
        match frame::edit_buttons(ui, s.paste, &mut e.hits.buttons) {
            Some("Copy") => acts.push(Act::Copy),
            Some("Paste") => acts.push(Act::Paste),
            Some(_) => acts.push(Act::Clear),
            None => {}
        }
        ui.separator();
        let h = frame::length_shuffle(ui, &s.bytes, &mut e.hits.buttons);
        length_drag = h.length_drag;
        if let Some((n, drag)) = h.length {
            acts.push(Act::Length { n, drag });
        }
        if h.shuffle {
            acts.push(Act::Shuffle);
        }
    });
    ui.add_space(6.0);

    let len = length(&s.bytes);
    // the step columns take the width the top row gives, up to the Transpose column
    let content_w = ui.min_rect().width();
    let grid_w = (content_w - KEY_W - GAP - ARROW.x).max(CELL_W * STEPS as f32);
    let cw = grid_w / STEPS as f32;
    let block_w = KEY_W + grid_w + GAP + ARROW.x;
    // ---- the step numbers ----
    let (head, _) = ui.allocate_exact_size(vec2(block_w, HEAD_H), Sense::hover());
    frame::step_numbers(ui, head, head.left() + KEY_W, cw, len, s.playing);

    // ---- the piano roll ----
    let rows = PITCHES as usize;
    let (block, _) = ui.allocate_exact_size(vec2(block_w, CELL_H * rows as f32), Sense::hover());
    let keys = Rect::from_min_size(block.min, vec2(KEY_W, block.height()));
    let grid = Rect::from_min_size(
        pos2(block.left() + KEY_W, block.top()),
        vec2(grid_w, block.height()),
    );
    e.hits.notes = Some(grid);
    let painter = ui.painter().clone();
    for r in 0..rows {
        let pitch = PITCHES - 1 - r as u8;
        let black = is_black(pitch);
        let key = Rect::from_min_size(
            pos2(keys.left(), keys.top() + r as f32 * CELL_H),
            vec2(KEY_W, CELL_H),
        );
        // every key the column's full width, so all the names end at one right edge (the
        // user's review, round 1)
        let (fill, ink) = if black { (BG, INK_DIM) } else { (INK, BG) };
        painter.rect_filled(key.shrink(0.5), CornerRadius::same(2), fill);
        painter.text(
            pos2(key.right() - 5.0, key.center().y),
            egui::Align2::RIGHT_CENTER,
            pitch_name(pitch),
            egui::FontId::proportional(LABEL_SIZE),
            ink,
        );
        let row = Rect::from_min_size(
            pos2(grid.left(), grid.top() + r as f32 * CELL_H),
            vec2(grid_w, CELL_H),
        );
        painter.rect_filled(row, 0.0, row_fill(pitch));
    }
    paint_columns(&painter, grid, len, s.playing);
    // a line where two white rows meet (E-F, B-C)
    for (_, upper) in WHITE_PAIRS {
        let y = grid.top() + (PITCHES - upper) as f32 * CELL_H;
        painter.line_segment(
            [pos2(grid.left(), y), pos2(grid.right(), y)],
            Stroke::new(1.0, SLATE_LINE),
        );
    }
    e.hits.playing = s.playing;
    e.hits.dimmed = len < STEPS;
    for st in 0..STEPS {
        if let Some(n) = note(&s.bytes, st).filter(|&n| n < PITCHES) {
            let c = cell_rect(grid, st, (PITCHES - 1 - n) as usize).shrink2(vec2(2.0, 2.0));
            painter.rect_filled(c, CornerRadius::same(2), dimmed(AMBER, st >= len));
        }
    }
    let id = egui::Id::unique((salt(s.synth), "notes"));
    match frame::grid_press(ui, id, grid, rows, &mut e.notes_down) {
        Some(Press::Down(step, row)) => acts.push(Act::NoteDown {
            step,
            pitch: PITCHES - 1 - row as u8,
        }),
        Some(Press::Held {
            cell: Some((step, row)),
            ..
        }) => acts.push(Act::NoteMove {
            step,
            pitch: PITCHES - 1 - row as u8,
        }),
        Some(Press::Release) => acts.push(Act::Release),
        _ => {}
    }
    // Transpose up and down at the grid's right, the pair centered on the piano roll
    let mid_y = grid.center().y;
    let tx = grid.right() + GAP;
    let up = Rect::from_min_size(pos2(tx, mid_y - 2.0 - ARROW.y), ARROW);
    let dn = Rect::from_min_size(pos2(tx, mid_y + 2.0), ARROW);
    let b = &mut e.hits.buttons;
    let sa = salt(s.synth);
    if frame::arrow_button(ui, up, vec2(0.0, -1.0), b, sa, "transpose_up") {
        acts.push(Act::Transpose { up: true });
    }
    if frame::arrow_button(ui, dn, vec2(0.0, 1.0), b, sa, "transpose_down") {
        acts.push(Act::Transpose { up: false });
    }

    // ---- the divider between the piano roll and the flag rows ----
    let (split, _) = ui.allocate_exact_size(vec2(block_w, SPLIT_H), Sense::hover());
    frame::divider(&painter, split, grid.right());

    // ---- the flag rows ----
    let frows = Flag::ALL.len();
    let (block, _) = ui.allocate_exact_size(vec2(block_w, CELL_H * frows as f32), Sense::hover());
    let flags = Rect::from_min_size(
        pos2(block.left() + KEY_W, block.top()),
        vec2(grid_w, block.height()),
    );
    e.hits.flags = Some(flags);
    for (r, f) in Flag::ALL.into_iter().enumerate() {
        let y = block.top() + (r as f32 + 0.5) * CELL_H;
        painter.text(
            pos2(block.left() + KEY_W - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            f.label(),
            egui::FontId::proportional(LABEL_SIZE),
            INK_DIM,
        );
        let row = Rect::from_min_size(
            pos2(flags.left(), flags.top() + r as f32 * CELL_H),
            vec2(grid_w, CELL_H),
        );
        painter.rect_filled(row, 0.0, if r % 2 == 0 { WHITE_ROW } else { BLACK_ROW });
    }
    paint_columns(&painter, flags, len, s.playing);
    for (r, f) in Flag::ALL.into_iter().enumerate() {
        for st in 0..STEPS {
            if flag(&s.bytes, st, f) {
                let c = cell_rect(flags, st, r).shrink2(vec2(4.0, 4.0));
                painter.rect_filled(c, CornerRadius::same(2), dimmed(AMBER, st >= len));
            }
        }
    }
    let id = egui::Id::unique((salt(s.synth), "flags"));
    match frame::grid_press(ui, id, flags, frows, &mut e.flags_down) {
        Some(Press::Down(step, row)) => acts.push(Act::FlagDown {
            flag: Flag::ALL[row],
            step,
        }),
        // the drag stays in its row: only the column counts
        Some(Press::Held {
            column: Some(step), ..
        }) => acts.push(Act::FlagMove { step }),
        Some(Press::Release) => acts.push(Act::Release),
        _ => {}
    }

    // ---- Shift at the bottom, centered under the step grid ----
    ui.add_space(6.0);
    let (row, _) = ui.allocate_exact_size(vec2(block_w, ARROW.y), Sense::hover());
    let mid = grid.center().x;
    let left = Rect::from_min_size(pos2(mid - 2.0 - ARROW.x, row.top()), ARROW);
    let right = Rect::from_min_size(pos2(mid + 2.0, row.top()), ARROW);
    let b = &mut e.hits.buttons;
    if frame::arrow_button(ui, left, vec2(-1.0, 0.0), b, sa, "shift_left") {
        acts.push(Act::Shift { right: false });
    }
    if frame::arrow_button(ui, right, vec2(1.0, 0.0), b, sa, "shift_right") {
        acts.push(Act::Shift { right: true });
    }
    length_drag
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pattern: the cleared steps (note 0, rest on), length 16, no shuffle.
    fn empty() -> Vec<u8> {
        let mut b = vec![0u8; PATTERN_BYTES];
        b[1] = 16;
        for s in 0..STEPS {
            b[step_at(s) + REST] = 1;
        }
        b
    }

    #[test]
    fn a_note_clears_the_rest_and_a_pause_keeps_the_pitch() {
        let b = with_note(&empty(), 3, 7);
        assert_eq!(note(&b, 3), Some(7));
        let b = with_pause(&b, 3);
        assert_eq!(note(&b, 3), None);
        assert_eq!(b[step_at(3) + NOTE], 7, "the stored pitch stays");
        assert_eq!(note(&with_note(&b, 3, 12), 3), Some(12));
    }

    #[test]
    fn up_and_down_are_independent_fields() {
        let b = with_flag(&empty(), 0, Flag::Up, true);
        let b = with_flag(&b, 0, Flag::Down, true);
        assert!(flag(&b, 0, Flag::Up) && flag(&b, 0, Flag::Down));
        assert_eq!(&b[step_at(0)..step_at(0) + 6], &[0, 0, 0, 1, 1, 1]);
        let b = with_flag(&b, 0, Flag::Slide, true);
        let b = with_flag(&b, 0, Flag::Accent, true);
        assert_eq!(&b[step_at(0)..step_at(0) + 6], &[0, 1, 1, 1, 1, 1]);
        let b = with_flag(&b, 0, Flag::Up, false);
        assert_eq!(&b[step_at(0)..step_at(0) + 6], &[0, 1, 1, 0, 1, 1]);
    }

    #[test]
    fn shift_wraps_within_the_length() {
        let mut b = with_length(&empty(), 4);
        for s in 0..STEPS {
            b = with_note(&b, s, s as u8 % 13);
        }
        let r = shifted(&b, true);
        let notes: Vec<_> = (0..STEPS).map(|s| note(&r, s).unwrap()).collect();
        assert_eq!(&notes[..4], &[3, 0, 1, 2]);
        assert_eq!(
            &notes[4..],
            &(4..16).map(|n| n % 13).collect::<Vec<u8>>()[..]
        );
        assert_eq!(shifted(&r, false), b, "left undoes right");
        // the whole step moves, flags and rest with it
        let f = with_flag(&with_pause(&b, 0), 0, Flag::Accent, true);
        let r = shifted(&f, false);
        assert_eq!(note(&r, 3), None);
        assert!(flag(&r, 3, Flag::Accent));
    }

    #[test]
    fn transpose_wraps_over_13_rows_and_keeps_up_and_down() {
        let b = with_flag(&with_note(&empty(), 0, 12), 0, Flag::Up, true);
        let b = with_note(&b, 1, 0);
        let up = transposed(&b, true);
        assert_eq!(note(&up, 0), Some(0), "12 up is 0");
        assert_eq!(note(&up, 1), Some(1));
        assert!(flag(&up, 0, Flag::Up));
        let dn = transposed(&b, false);
        assert_eq!(note(&dn, 0), Some(11));
        assert_eq!(note(&dn, 1), Some(12), "0 down is 12");
        // a pause's pitch moves too, unseen; a pitch above 12 stays
        assert_eq!(up[step_at(2) + NOTE], 1);
        let mut odd = empty();
        odd[step_at(5) + NOTE] = 20;
        assert_eq!(transposed(&odd, true)[step_at(5) + NOTE], 20);
        assert_eq!(transposed(&transposed(&b, true), false), b);
    }

    #[test]
    fn length_and_shuffle_are_the_header() {
        let b = with_shuffle(&with_length(&empty(), 7), true);
        assert_eq!(&b[..2], &[1, 7]);
        assert_eq!(with_length(&b, 0)[1], 1);
        assert_eq!(with_length(&b, 30)[1], 16);
        assert_eq!(length(&with_length(&b, 0)), 1);
    }

    #[test]
    fn a_drag_crosses_every_step_on_the_line() {
        assert_eq!(crossed(2, 5, 2, 7), vec![(2, 7)]);
        assert_eq!(crossed(0, 0, 4, 4), vec![(1, 1), (2, 2), (3, 3), (4, 4)]);
        assert_eq!(crossed(5, 12, 3, 12), vec![(4, 12), (3, 12)]);
    }

    #[test]
    fn the_number_arrows_run_a1_to_d8_of_the_303_and_stop_at_the_ends() {
        assert_eq!(step_pattern(7, true), Some(8), "A8 to B1");
        assert_eq!(step_pattern(8, false), Some(7));
        assert_eq!(step_pattern(0, false), None, "A1 is the first");
        assert_eq!(step_pattern(31, true), None, "D8 is the last");
    }

    #[test]
    fn each_303_has_its_own_window() {
        assert_eq!(title(0), "303-1 Pattern Editor");
        assert_eq!(title(1), "303-2 Pattern Editor");
        assert_eq!(menu_item(0), "303-1 Pattern Editor...");
        assert_eq!(menu_item(1), "303-2 Pattern Editor...");
        assert_ne!(salt(0), salt(1));
        assert_eq!(Editor::new(1).synth, 1);
        assert!(!Editor::new(0).open);
    }

    #[test]
    fn the_black_keys() {
        let black: Vec<u8> = (0..PITCHES).filter(|&p| is_black(p)).collect();
        assert_eq!(black, vec![1, 3, 6, 8, 10]);
        // the app's palette, not the skin's: black-key rows darker than white-key rows
        assert_eq!(row_fill(0), SLATE_DARK);
        assert_eq!(row_fill(1), BG);
        assert_eq!(row_fill(12), SLATE_DARK);
        assert_eq!(pitch_name(12), "C");
        assert_eq!(pitch_name(1), "C#");
    }
}
