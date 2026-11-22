//! The 303 pattern editor (task-274, docs/ui_spec.md section 23): a floating window over the
//! panel that edits one TB-303 pattern on a step grid with a 13-key piano roll and the Up,
//! Down, Accent and Slide rows. It edits only what the song stores for a 303 pattern (the 98
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
pub use crate::editor_frame::STEPS;
use crate::editor_frame::{
    self as frame, ARROW, CELL_H, CELL_W, GAP, HEAD_H, KEY_W, LABEL_SIZE, Number, Placing, Press,
    SPLIT_H, cell_rect, dimmed, paint_columns,
};
use crate::link::emptied_pattern;
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

/// The pattern after (`next`) or before pattern `pattern` (0..=31) of 303 `synth` in the one
/// sequence of the number's arrows, 303-1 A1..D8 then 303-2 A1..D8 (64 patterns): 8 goes on to
/// the next bank's 1, 303-1 D8 to 303-2 A1. `None` past the ends (303-1 A1, 303-2 D8).
pub fn step_pattern(synth: usize, pattern: u8, next: bool) -> Option<(usize, u8)> {
    let i = synth.min(1) * 32 + pattern.min(31) as usize;
    let j = if next {
        i.checked_add(1)?
    } else {
        i.checked_sub(1)?
    };
    (j < 64).then_some((j / 32, (j % 32) as u8))
}

/// What the editor's window asks for; [`App::editor_act`] runs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// The 303 switch: show 303 `synth` (0 or 1); in pattern mode its selected pattern, in song
    /// mode the same pattern number as before.
    Synth(usize),
    /// A bank button or the pattern number: show pattern `pattern` (0..=31) of 303 `synth`,
    /// selected on the panel too in pattern mode.
    Select {
        synth: usize,
        pattern: u8,
    },
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

/// The editor's state, kept by the app.
#[derive(Clone, Debug, Default)]
pub struct Editor {
    /// Whether the window is open.
    pub open: bool,
    /// The 303 shown (0 or 1) and its pattern (0..=31).
    pub synth: usize,
    pub pattern: u8,
    /// The window's placing and place.
    win: Placing,
    gesture: Option<Gesture>,
    /// The 303 rows' selected patterns at the last frame (pattern mode follows a change).
    seen: [u8; 2],
    /// Whether the last frame was in song mode.
    was_song_mode: bool,
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
    /// The editor's state.
    pub fn pattern_editor(&self) -> &Editor {
        &self.editor
    }

    /// Edit > 303 Pattern Editor... and Cmd/Ctrl+E: opens the window on the selected 303's active
    /// pattern (the focused row when it is a 303, else the 303 the editor showed last), or
    /// brings it to the front and switches it to that pattern. Nothing without a song.
    pub fn open_pattern_editor(&mut self) {
        if self.link.is_none() {
            return;
        }
        let e = &mut self.editor;
        if self.panel.focus < 2 {
            e.synth = self.panel.focus;
        }
        e.pattern = self.panel.rows[e.synth].pattern;
        e.seen = [self.panel.rows[0].pattern, self.panel.rows[1].pattern];
        e.was_song_mode = self.panel.transport.song_mode;
        e.win.open(e.open);
        e.open = true;
        e.gesture = None;
    }

    /// Where the window drew step `step`'s cell at `pitch` of the piano roll last.
    pub fn pattern_editor_note_cell(&self, step: usize, pitch: u8) -> Option<Rect> {
        let r = self.editor.hits.notes?;
        Some(cell_rect(r, step, (PITCHES - 1 - pitch) as usize))
    }

    /// Where the window drew step `step`'s cell of flag row `flag` last.
    pub fn pattern_editor_flag_cell(&self, flag: Flag, step: usize) -> Option<Rect> {
        let r = self.editor.hits.flags?;
        let row = Flag::ALL.iter().position(|&f| f == flag)?;
        Some(cell_rect(r, step, row))
    }

    /// Where the window was drawn last (`None` before its first frame).
    pub fn pattern_editor_window(&self) -> Option<Rect> {
        self.editor.win.window.filter(|_| self.editor.open)
    }

    /// The playing step the grid showed last, and whether it dimmed the steps beyond the
    /// pattern's length.
    pub fn pattern_editor_shown(&self) -> (Option<usize>, bool) {
        (self.editor.hits.playing, self.editor.hits.dimmed)
    }

    /// Where the window drew its button or value `name` last ("303-1", "A", "number",
    /// "Copy", "Length", "Shuffle", "shift_left", "transpose_up", ...).
    pub fn pattern_editor_hit(&self, name: &str) -> Option<Rect> {
        self.editor
            .hits
            .buttons
            .iter()
            .find(|(n, _)| *n == name)
            .map(|&(_, r)| r)
    }

    /// The selection rules (docs/ui_spec.md section 23): in pattern mode the editor shows the
    /// selected pattern of its 303, and a new selection of one 303 row moves it to that 303;
    /// on the switch into pattern mode it takes the panel's selection. In song mode it keeps
    /// its own pattern.
    fn editor_follow(&mut self) {
        let song = self.panel.transport.song_mode;
        let now = [self.panel.rows[0].pattern, self.panel.rows[1].pattern];
        let e = &mut self.editor;
        if !song {
            // one row's new selection (not both, as a song open makes) moves the editor to it
            let changed: Vec<usize> = (0..2)
                .filter(|&k| !e.was_song_mode && now[k] != e.seen[k])
                .collect();
            if let [k] = changed[..] {
                e.synth = k;
            }
            e.pattern = now[e.synth];
        }
        e.seen = now;
        e.was_song_mode = song;
    }

    /// The editor's pattern as the link holds it, after the panel's pending events were noted
    /// (an edit of the panel forgets the Undo first) and sent; `None` without a song.
    fn editor_bytes(&mut self) -> Option<Vec<u8>> {
        self.link.as_ref()?;
        self.note_edits();
        let link = self.link.as_mut()?;
        link.forward(&mut self.panel);
        Some(
            link.pattern(self.editor.synth, self.editor.pattern)
                .to_vec(),
        )
    }

    /// Writes `f` of the editor's pattern for the command `what`; with `keep` (a drag that
    /// wrote already) the Undo stays the one of the drag's first write. Whether it changed.
    fn editor_edit(
        &mut self,
        what: &'static str,
        keep: bool,
        f: impl FnOnce(&[u8]) -> Vec<u8>,
    ) -> bool {
        let Some(before) = self.editor_bytes() else {
            return false;
        };
        let (k, p) = (self.editor.synth, self.editor.pattern);
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

    /// Runs what the window asked for.
    pub fn editor_act(&mut self, a: Act) {
        const EDIT: &str = "Edit Pattern";
        if self.link.is_none() {
            return;
        }
        let (k, p) = (self.editor.synth, self.editor.pattern);
        match a {
            Act::Synth(synth) => {
                let synth = synth.min(1);
                let pattern = if self.panel.transport.song_mode {
                    p
                } else {
                    self.panel.rows[synth].pattern
                };
                self.editor_select(synth, pattern);
            }
            Act::Select { synth, pattern } => self.editor_select(synth, pattern),
            Act::Copy => {
                if self.editor_bytes().is_some() {
                    self.copy_pattern_at(k, p, false);
                }
            }
            Act::Paste => {
                if self.editor_bytes().is_some() {
                    self.paste_pattern_at(k, p);
                }
            }
            Act::Clear => {
                if self.editor_edit("Clear Pattern", false, |b| emptied_pattern(k, b)) {
                    let name = crate::menu::pattern_name(p);
                    self.status = Some(format!("{}: {name} cleared", crate::menu::ROW_NAMES[k]));
                }
            }
            Act::Length { n, drag } => {
                let keep = drag && self.editor.length_wrote;
                let changed = self.editor_edit("Pattern Length", keep, |b| with_length(b, n));
                self.editor.length_wrote = drag && (keep || changed);
            }
            Act::Shuffle => {
                self.editor_edit("Pattern Shuffle", false, |b| with_shuffle(b, b[0] == 0));
            }
            Act::Shift { right } => {
                self.editor_edit("Shift Pattern", false, |b| shifted(b, right));
            }
            Act::Transpose { up } => {
                self.editor_edit("Transpose Pattern", false, |b| transposed(b, up));
            }
            Act::NoteDown { step, pitch } => {
                let Some(bytes) = self.editor_bytes() else {
                    return;
                };
                let lit = note(&bytes, step) == Some(pitch);
                // a press on the lit cell waits: a click there makes a pause, a drag paints
                let wrote = !lit && self.editor_edit(EDIT, false, |b| with_note(b, step, pitch));
                self.editor.gesture = Some(Gesture::Notes {
                    last: (step, pitch),
                    pause: lit,
                    wrote,
                });
            }
            Act::NoteMove { step, pitch } => {
                let Some(Gesture::Notes { last, wrote, .. }) = self.editor.gesture else {
                    return;
                };
                if last == (step, pitch) {
                    return;
                }
                let cells = crossed(last.0, last.1, step, pitch);
                let changed = self.editor_edit(EDIT, wrote, |b| {
                    cells
                        .iter()
                        .fold(b.to_vec(), |b, &(s, n)| with_note(&b, s, n))
                });
                self.editor.gesture = Some(Gesture::Notes {
                    last: (step, pitch),
                    pause: false,
                    wrote: wrote || changed,
                });
            }
            Act::FlagDown { flag: f, step } => {
                let Some(bytes) = self.editor_bytes() else {
                    return;
                };
                let on = !flag(&bytes, step, f);
                let wrote = self.editor_edit(EDIT, false, |b| with_flag(b, step, f, on));
                self.editor.gesture = Some(Gesture::Flags {
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
                }) = self.editor.gesture
                else {
                    return;
                };
                if last == step {
                    return;
                }
                let changed = self.editor_edit(EDIT, wrote, |b| {
                    (last.min(step)..=last.max(step))
                        .fold(b.to_vec(), |b, s| with_flag(&b, s, f, on))
                });
                self.editor.gesture = Some(Gesture::Flags {
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
                }) = self.editor.gesture.take()
                {
                    self.editor_edit(EDIT, false, |b| with_pause(b, step));
                }
            }
        }
    }

    /// The editor's selector: in song mode it chooses what the editor shows only; in pattern
    /// mode the pattern is selected on the panel too, as a click on its button selects it.
    fn editor_select(&mut self, synth: usize, pattern: u8) {
        let (synth, pattern) = (synth.min(1), pattern.min(31));
        self.editor.synth = synth;
        self.editor.pattern = pattern;
        self.editor.gesture = None;
        if !self.panel.transport.song_mode && self.panel.rows[synth].pattern != pattern {
            self.select_pattern(synth, pattern);
        }
        self.editor.seen = [self.panel.rows[0].pattern, self.panel.rows[1].pattern];
    }

    /// Draws the editor's window when it is open, and runs what it asks for.
    pub(crate) fn pattern_editor_ui(&mut self, ctx: &Context) {
        if !self.editor.open {
            return;
        }
        if self.link.is_none() {
            self.editor.open = false;
            return;
        }
        self.editor_follow();
        let Some(link) = &self.link else {
            return;
        };
        let (k, p) = (self.editor.synth, self.editor.pattern);
        let shown = Shown {
            bytes: link.pattern(k, p).to_vec(),
            synth: k,
            pattern: p,
            playing: self.panel.playing_step303[k].filter(|_| self.panel.rows[k].pattern == p),
            paste: self.clip.as_ref().is_some_and(|c| c.row < 2),
        };
        let plugin = self.is_plugin();
        let saved = (!plugin)
            .then_some(self.settings.pattern_editor_pos)
            .flatten();
        let mut acts = Vec::new();
        let mut length_drag = false;
        let mut win = std::mem::take(&mut self.editor.win);
        let editor = &mut self.editor;
        let spec = frame::Spec {
            id: egui::Id::unique("orb_pattern_editor"),
            title: "303 Pattern Editor",
            saved,
            guess: vec2(530.0, 340.0),
            keep: !plugin,
        };
        let out = frame::show_window(ctx, spec, &mut win, |ui| {
            length_drag = window_ui(ui, editor, &shown, &mut acts);
        });
        self.editor.win = win;
        if !length_drag {
            self.editor.length_wrote = false;
        }
        for a in acts {
            self.editor_act(a);
        }
        // the place is saved once the window is let go (the standalone app's only)
        if let Some(pos) = out.save {
            self.settings.set_pattern_editor_pos(Some(pos));
            if let Err(e) = self.settings.save() {
                self.fail(e);
            }
        }
        if !out.open {
            self.editor.open = false;
            self.editor.gesture = None;
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
        for k in 0..2 {
            let name = ["303-1", "303-2"][k];
            let r = ui.selectable_label(s.synth == k, name);
            e.hits.buttons.push((name, r.rect));
            if r.clicked() && s.synth != k {
                acts.push(Act::Synth(k));
            }
        }
        ui.separator();
        if let Some(b) = frame::bank_buttons(ui, s.pattern, &mut e.hits.buttons) {
            acts.push(Act::Select {
                synth: s.synth,
                pattern: b * 8 + s.pattern % 8,
            });
        }
        // the number between its arrows, which step through 303-1 A1..D8 then 303-2 A1..D8
        // and stop at the two ends (`step_pattern`)
        let prev = step_pattern(s.synth, s.pattern, false);
        let next = step_pattern(s.synth, s.pattern, true);
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
            Some(Number::Set(n)) => Some((s.synth, bank * 8 + n)),
            None => None,
        };
        if let Some((synth, pattern)) = to {
            acts.push(Act::Select { synth, pattern });
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
    let id = egui::Id::unique("orb_pattern_editor_notes");
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
    if frame::arrow_button(ui, up, vec2(0.0, -1.0), b, SALT, "transpose_up") {
        acts.push(Act::Transpose { up: true });
    }
    if frame::arrow_button(ui, dn, vec2(0.0, 1.0), b, SALT, "transpose_down") {
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
    let id = egui::Id::unique("orb_pattern_editor_flags");
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
    if frame::arrow_button(ui, left, vec2(-1.0, 0.0), b, SALT, "shift_left") {
        acts.push(Act::Shift { right: false });
    }
    if frame::arrow_button(ui, right, vec2(1.0, 0.0), b, SALT, "shift_right") {
        acts.push(Act::Shift { right: true });
    }
    length_drag
}

/// The salt of the window's arrow buttons' ids.
const SALT: &str = "orb_pattern_editor";

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
    fn the_number_arrows_run_one_sequence_of_64_and_stop_at_its_ends() {
        assert_eq!(step_pattern(0, 7, true), Some((0, 8)), "A8 to B1");
        assert_eq!(step_pattern(0, 8, false), Some((0, 7)));
        assert_eq!(
            step_pattern(0, 31, true),
            Some((1, 0)),
            "303-1 D8 to 303-2 A1"
        );
        assert_eq!(step_pattern(1, 0, false), Some((0, 31)));
        assert_eq!(step_pattern(0, 0, false), None, "303-1 A1 is the first");
        assert_eq!(step_pattern(1, 31, true), None, "303-2 D8 is the last");
        let mut at = (0, 0);
        let mut n = 1;
        while let Some(p) = step_pattern(at.0, at.1, true) {
            at = p;
            n += 1;
        }
        assert_eq!((at, n), ((1, 31), 64));
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
