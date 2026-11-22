//! The drum pattern editors (task-280 and task-281, docs/ui_spec.md sections 24 and 25): a
//! floating window that edits one drum machine pattern on a step grid of 11 drum rows and the
//! AC row. The TR-808's is [`Machine::Tr808`]; the TR-909's is [`Machine::Tr909`] on the same
//! frame, which adds only its cell values, colors and entry switch ([`Entry`]).
//!
//! A drum pattern is the 194 bytes of `docs/RBS42.txt`'s "808 " and "909 " chunks
//! (`docs/song_load.md` sections 4.7 and 4.8): the shuffle flag, the length, and 16 steps of 12
//! bytes, one per instrument in RBS order with the accent (AC) first. The grid's drum rows are
//! bytes 1..=11 top to bottom, the AC row byte 0. On the 808 every byte is 0 or 1; on the 909 a
//! drum byte is 0 off, 1 low, 2 high, 3 flam, and the AC byte 0 or 2 (a song may hold any byte:
//! the editor draws one above 3 as flam and keeps it until that cell is edited).
//!
//! Every change is one whole-pattern write through [`App::write_pattern`] (the Edit menu's
//! path, `crate::link::Link::set_pattern`), as in the 303 editor (`crate::pattern_editor`): it
//! is heard, marks the song modified, shows on the panel and is undone by the one-level Undo. A
//! drag is one change. The window's frame is `crate::editor_frame`, shared with the 303 editor.

use egui::{Color32, Context, CornerRadius, Rect, RichText, Sense, Ui, Vec2, pos2, vec2};

use crate::app::App;
use crate::behavior::{self, Mods};
use crate::editor_frame::{
    self as frame, ARROW, CELL_H, HEAD_H, KEY_W, LABEL_SIZE, Number, Placing, Press, SPLIT_H,
    STEPS, cell_rect, dimmed, paint_columns,
};
use crate::link::emptied_pattern;
use crate::model::Panel;
use crate::palette::{AMBER, BG, INK_DIM, SLATE_DARK};
use crate::settings::Settings;

/// The bytes of a drum pattern: the shuffle flag, the length, 16 steps of 12 bytes.
pub const PATTERN_BYTES: usize = 194;
/// The bytes of a step: one per instrument, the accent first.
pub const STEP_BYTES: usize = 12;
/// The drum rows of the grid (bytes 1..=11 of a step); the AC row is byte 0.
pub const DRUMS: usize = 11;
/// The step byte of the accent.
pub const AC: usize = 0;

/// The step bytes of the TR-909's hi-hats: entering one clears the other on that step.
pub const CH909: usize = 8;
pub const OH909: usize = 9;

/// The colors of the TR-909's cell values 1 (low), 2 (high) and 3 (flam), as the panel's step
/// LEDs draw them (the modern skin's `step909_1` .. `step909_3`; the original's orange, red and
/// green, docs/ui_spec.md section 7).
pub const LOW: Color32 = Color32::from_rgb(0xff, 0x9a, 0x30);
pub const HIGH: Color32 = Color32::from_rgb(0xff, 0x4a, 0x36);
pub const FLAM: Color32 = Color32::from_rgb(0x5c, 0xff, 0x4a);

/// A drum machine with an editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Machine {
    Tr808,
    Tr909,
}

/// The TR-909's 11 drum rows, step bytes 1..=11.
const LABELS_909: [&str; DRUMS] = [
    "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CH", "OH", "CC", "RC",
];

/// What a click on a TR-909 drum cell enters (task-281): the editor's entry switch, or the
/// modifiers held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Entry {
    /// Off -> low -> high -> off, as the panel's plain click.
    #[default]
    Cycle,
    /// Set low (1), or clear a cell that holds low.
    Low,
    /// Set high (2), or clear a cell that holds high.
    High,
    /// Set flam (3), or clear a cell that holds flam.
    Flam,
}

impl Entry {
    pub const ALL: [Entry; 4] = [Entry::Cycle, Entry::Low, Entry::High, Entry::Flam];

    /// The switch's text.
    pub fn name(self) -> &'static str {
        match self {
            Entry::Cycle => "Cycle",
            Entry::Low => "Low",
            Entry::High => "High",
            Entry::Flam => "Flam",
        }
    }

    /// The entry the modifiers select while held, as on the panel's step buttons
    /// ([`behavior::step_mods`]): the command key (Cmd on macOS, Ctrl elsewhere) Low, Alt
    /// (Option) High, both Flam; `None` without them.
    pub fn held(m: egui::Modifiers) -> Option<Entry> {
        match behavior::step_mods(m) {
            Mods {
                ctrl: true,
                alt: true,
            } => Some(Entry::Flam),
            Mods { ctrl: true, .. } => Some(Entry::Low),
            Mods { alt: true, .. } => Some(Entry::High),
            _ => None,
        }
    }

    /// The panel's modifiers that enter the same ([`behavior::tr909_step_value`]).
    fn mods(self) -> Mods {
        let (ctrl, alt) = match self {
            Entry::Cycle => (false, false),
            Entry::Low => (true, false),
            Entry::High => (false, true),
            Entry::Flam => (true, true),
        };
        Mods { ctrl, alt }
    }

    /// Its value's color, `None` for Cycle.
    fn color(self) -> Option<Color32> {
        match self {
            Entry::Cycle => None,
            Entry::Low => Some(LOW),
            Entry::High => Some(HIGH),
            Entry::Flam => Some(FLAM),
        }
    }
}

/// The TR-808's switched slots 3..=7: the lower legend's sound (switch value non-zero, the
/// switch down) and the upper label's (0), `docs/ui_spec.md` section 6.
const SLOTS_808: [(&str, &str); 5] = [
    ("LT", "LC"),
    ("MT", "MC"),
    ("HT", "HC"),
    ("RS", "CL"),
    ("CP", "MA"),
];

impl Machine {
    pub const ALL: [Machine; 2] = [Machine::Tr808, Machine::Tr909];

    /// Its place in `App`'s editors.
    pub fn index(self) -> usize {
        match self {
            Machine::Tr808 => 0,
            Machine::Tr909 => 1,
        }
    }

    /// Its row on the panel (0 and 1 are the 303s).
    pub fn row(self) -> usize {
        match self {
            Machine::Tr808 => 2,
            Machine::Tr909 => 3,
        }
    }

    /// The machine of panel row `row`, `None` for a 303's.
    pub fn of_row(row: usize) -> Option<Machine> {
        Machine::ALL.into_iter().find(|m| m.row() == row)
    }

    /// The panel's name of it.
    pub fn name(self) -> &'static str {
        match self {
            Machine::Tr808 => "TR-808",
            Machine::Tr909 => "TR-909",
        }
    }

    /// The window's title.
    pub fn title(self) -> &'static str {
        match self {
            Machine::Tr808 => "808 Pattern Editor",
            Machine::Tr909 => "909 Pattern Editor",
        }
    }

    /// The Edit menu's item (the ellipsis: it opens a window).
    pub fn menu_item(self) -> &'static str {
        match self {
            Machine::Tr808 => "808 Pattern Editor...",
            Machine::Tr909 => "909 Pattern Editor...",
        }
    }

    /// The salt of the window's ids.
    pub fn salt(self) -> &'static str {
        match self {
            Machine::Tr808 => "orb_drum_editor_808",
            Machine::Tr909 => "orb_drum_editor_909",
        }
    }

    /// Whether its editor has the entry switch (the 909's values).
    pub fn has_entry(self) -> bool {
        self == Machine::Tr909
    }

    /// Every input its editor handles, as Help > Keyboard Shortcuts lists them.
    pub fn inputs(self) -> &'static [Input] {
        match self {
            Machine::Tr808 => &Input::ALL_808,
            Machine::Tr909 => &Input::ALL_909,
        }
    }

    /// The names of the 11 drum rows, top to bottom (step bytes 1..=11), as the panel names
    /// them now: the 808's five switched slots by their switches (`tr808.slot3_switch` ..
    /// `tr808.slot7_switch`, song controllers the editor does not change).
    pub fn labels(self, panel: &Panel) -> [&'static str; DRUMS] {
        match self {
            Machine::Tr808 => {
                let slot = |k: usize| {
                    let (lower, upper) = SLOTS_808[k - 3];
                    if panel.switch_position(&format!("tr808.slot{k}_switch")) != 0 {
                        lower
                    } else {
                        upper
                    }
                };
                [
                    "BD",
                    "SD",
                    slot(3),
                    slot(4),
                    slot(5),
                    slot(6),
                    slot(7),
                    "CB",
                    "CY",
                    "OH",
                    "CH",
                ]
            }
            Machine::Tr909 => LABELS_909,
        }
    }

    /// The value a click with the entry `entry` writes into a cell of step byte `column` that
    /// holds `v`.
    /// - The 808: a toggle, 0 <-> 1 (a stored byte above 1 counts as on); `entry` is not read.
    /// - The 909: the panel's rule ([`behavior::tr909_step_value`], the original's
    ///   `CTR909PatternView`) with `entry` as its modifiers, on the value the cell shows (a byte
    ///   above 3 shows and counts as flam): Cycle 0 -> 1 -> 2 -> 0 (flam -> 1), Low / High / Flam
    ///   that value, or 0 when the cell holds it. The AC row toggles 0 <-> 2 whatever `entry`.
    pub fn clicked(self, column: usize, v: u8, entry: Entry) -> u8 {
        match self {
            Machine::Tr808 => u8::from(v == 0),
            Machine::Tr909 => {
                behavior::tr909_step_value(v.min(3), column == AC, false, entry.mods())
            }
        }
    }

    /// `bytes` with instrument `column` of step `s` set to `v`; on the 909 a non-zero CH clears
    /// OH on that step and the other way round, as the panel's step buttons
    /// (`sub_447153(9 or 8, step, 0)`, docs/ui_spec.md section 7; `Panel::set_step909`).
    pub fn put(self, bytes: &[u8], s: usize, column: usize, v: u8) -> Vec<u8> {
        let mut b = with_cell(bytes, s, column, v);
        if self == Machine::Tr909 && v != 0 && (column == CH909 || column == OH909) {
            b[at(s, CH909 + OH909 - column)] = 0;
        }
        b
    }

    /// The color of a cell that holds `v`, `None` for an empty cell.
    pub fn color(self, v: u8) -> Option<Color32> {
        match self {
            Machine::Tr808 => (v != 0).then_some(AMBER),
            Machine::Tr909 => match v {
                0 => None,
                1 => Some(LOW),
                2 => Some(HIGH),
                // 3, and every byte above it, as the panel and the original draw it
                _ => Some(FLAM),
            },
        }
    }

    /// The window's place saved in the settings.
    fn saved(self, s: &Settings) -> Option<[i32; 2]> {
        match self {
            Machine::Tr808 => s.pattern_editor_808_pos,
            Machine::Tr909 => s.pattern_editor_909_pos,
        }
    }

    fn save(self, s: &mut Settings, pos: [i32; 2]) {
        match self {
            Machine::Tr808 => s.set_pattern_editor_808_pos(Some(pos)),
            Machine::Tr909 => s.set_pattern_editor_909_pos(Some(pos)),
        }
    }
}

/// The full name of a row's label (its tooltip).
pub fn label_name(label: &str) -> &'static str {
    match label {
        "BD" => "Bass Drum",
        "SD" => "Snare Drum",
        "LT" => "Low Tom",
        "LC" => "Low Conga",
        "MT" => "Mid Tom",
        "MC" => "Mid Conga",
        "HT" => "High Tom",
        "HC" => "High Conga",
        "RS" => "Rim Shot",
        "CL" => "Claves",
        "CP" => "Hand Clap",
        "MA" => "Maracas",
        "CB" => "Cowbell",
        "CY" => "Cymbal",
        "OH" => "Open Hi-Hat",
        "CH" => "Closed Hi-Hat",
        "CC" => "Crash Cymbal",
        "RC" => "Ride Cymbal",
        "AC" => AC_TIP,
        _ => "",
    }
}

/// The AC row's tooltip.
pub const AC_TIP: &str = "Accent: every drum on this step louder";

/// Where step `s`'s byte `column` is.
fn at(s: usize, column: usize) -> usize {
    2 + STEP_BYTES * s + column
}

/// The byte of instrument `column` (0 the accent) on step `s`.
pub fn cell(bytes: &[u8], s: usize, column: usize) -> u8 {
    bytes[at(s, column)]
}

/// `bytes` with instrument `column` of step `s` set to `v`.
pub fn with_cell(bytes: &[u8], s: usize, column: usize, v: u8) -> Vec<u8> {
    let mut b = bytes.to_vec();
    b[at(s, column)] = v;
    b
}

/// `bytes` with the steps rotated by one step, right (later) or left, wrapping within the
/// pattern's length; the steps beyond the length stay.
pub fn shifted(bytes: &[u8], right: bool) -> Vec<u8> {
    frame::shifted(bytes, STEP_BYTES, right)
}

/// The pattern after (`next`) or before pattern `pattern` (0..=31) in the number's arrows' one
/// sequence A1..D8: 8 goes on to the next bank's 1. `None` past the ends (A1, D8).
pub fn step_pattern(pattern: u8, next: bool) -> Option<u8> {
    let p = pattern.min(31);
    if next {
        (p < 31).then_some(p + 1)
    } else {
        p.checked_sub(1)
    }
}

/// What the keys and the mouse do in a drum editor: each is a line of Help > Keyboard
/// Shortcuts (`crate::key_help`, which lists every one of [`Machine::inputs`]) and a case of
/// the tests (`tests/drum_editor.rs` `every_input_of_the_help_does_what_it_says`, a match on
/// every one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// Ctrl/Cmd+E with the drum machine's row focused (the standalone app).
    OpenKey,
    /// A click on a cell.
    Click,
    /// A drag from a cell.
    Drag,
    /// The 909: a click on the entry switch.
    EntrySwitch,
    /// The 909: the modifiers of Low, High and Flam held.
    HoldLow,
    HoldHigh,
    HoldFlam,
    /// A click or drag with another modifier held: the same as without.
    Modified,
    /// A drag up or down on the pattern number or Length.
    ValueDrag,
    /// The same with Shift held: finer.
    ValueFineDrag,
    /// A click on the pattern number or Length, a value typed and Enter.
    ValueType,
    /// Escape while typing a value.
    ValueCancel,
    /// Edit > Undo's key.
    Undo,
}

impl Input {
    /// The 808 editor's inputs.
    pub const ALL_808: [Input; 9] = [
        Input::OpenKey,
        Input::Click,
        Input::Drag,
        Input::Modified,
        Input::ValueDrag,
        Input::ValueFineDrag,
        Input::ValueType,
        Input::ValueCancel,
        Input::Undo,
    ];

    /// The 909 editor's inputs.
    pub const ALL_909: [Input; 13] = [
        Input::OpenKey,
        Input::Click,
        Input::Drag,
        Input::EntrySwitch,
        Input::HoldLow,
        Input::HoldHigh,
        Input::HoldFlam,
        Input::Modified,
        Input::ValueDrag,
        Input::ValueFineDrag,
        Input::ValueType,
        Input::ValueCancel,
        Input::Undo,
    ];

    /// The keys or the mouse action in machine `m`'s editor, as the help writes them for a
    /// platform.
    pub fn keys(self, m: Machine, is_mac: bool) -> String {
        let cmd = if is_mac { "Cmd" } else { "Ctrl" };
        let alt = if is_mac { "Option" } else { "Alt" };
        match self {
            Input::OpenKey => format!("{cmd}+E"),
            Input::Click => "Click".into(),
            Input::Drag => "Drag".into(),
            Input::EntrySwitch => "Click Cycle / Low / High / Flam".into(),
            Input::HoldLow => format!("Hold {cmd}"),
            Input::HoldHigh => format!("Hold {alt}"),
            Input::HoldFlam => format!("Hold {cmd}+{alt}"),
            Input::Modified => match (m.has_entry(), is_mac) {
                (false, true) => "Shift / Cmd / Option / Ctrl + click or drag",
                (false, false) => "Shift / Ctrl / Alt + click or drag",
                (true, true) => "Shift / Ctrl + click or drag",
                (true, false) => "Shift + click or drag",
            }
            .into(),
            Input::ValueDrag => "Drag up / down".into(),
            Input::ValueFineDrag => "Shift+drag up / down".into(),
            Input::ValueType => "Click, type, Enter".into(),
            Input::ValueCancel => "Escape".into(),
            Input::Undo => format!("{cmd}+Z"),
        }
    }

    /// What it does in machine `m`'s editor, as the help writes it.
    pub fn what(self, m: Machine) -> &'static str {
        match (self, m) {
            (Input::OpenKey, Machine::Tr808) => {
                "Open the editor with the TR-808 row focused (Edit > 808 Pattern Editor...), \
                 or bring it to the front on the active pattern. Not in the plugin, which \
                 leaves keys to the host: there it is the menu"
            }
            (Input::OpenKey, Machine::Tr909) => {
                "Open the editor with the TR-909 row focused (Edit > 909 Pattern Editor...), \
                 or bring it to the front on the active pattern. Not in the plugin, which \
                 leaves keys to the host: there it is the menu"
            }
            (Input::Click, Machine::Tr808) => "Toggle a cell: a drum or AC on a step",
            (Input::Click, Machine::Tr909) => {
                "Enter the entry switch's value in a drum cell: Cycle goes off, low (orange), \
                 high (red), off; Low, High or Flam (green) sets that value, or clears a cell \
                 that holds it. Entering CH clears OH on that step, and the other way round. \
                 On the AC row: on or off, whatever the switch"
            }
            (Input::Drag, _) => {
                "Paint the first cell's new value along its row (the drag stays in the row)"
            }
            (Input::EntrySwitch, _) => {
                "Choose what a click on a drum cell enters (Cycle when the app starts)"
            }
            (Input::HoldLow, _) => {
                "Low while held: the switch shows it and returns to its setting on release \
                 (the panel's 909 step buttons take the same key for low)"
            }
            (Input::HoldHigh, _) => {
                "High while held: the switch shows it and returns to its setting on release \
                 (the panel's 909 step buttons take the same key for high)"
            }
            (Input::HoldFlam, _) => {
                "Flam while held: the switch shows it and returns to its setting on release \
                 (the panel's 909 step buttons take the same keys for flam)"
            }
            (Input::Modified, Machine::Tr808) => {
                "The same as without: the editor reads no modifier"
            }
            (Input::Modified, Machine::Tr909) => {
                "The same as without: the editor reads no other modifier"
            }
            (Input::ValueDrag, _) => "On the pattern number or Length: change it",
            (Input::ValueFineDrag, _) => "On the pattern number or Length: change it slower",
            (Input::ValueType, _) => "On the pattern number or Length: type a value and set it",
            (Input::ValueCancel, _) => "While typing a value: cancel it",
            (Input::Undo, _) => {
                "Edit > Undo: take the editor's last change back (a drag is one change). In \
                 the plugin, the menu item"
            }
        }
    }
}

/// What the editor's window asks for; [`App::drum_editor_act`] runs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrumAct {
    /// A bank button, the pattern number or its arrows: show pattern `0..=31`, selected on the
    /// panel too in pattern mode.
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
    /// The 909's entry switch set to an entry.
    Entry(Entry),
    /// Shift right (later) or left.
    Shift {
        right: bool,
    },
    /// The mouse went down on instrument `column`'s cell (0 the AC row) of step `step`.
    CellDown {
        column: usize,
        step: usize,
    },
    /// The held mouse is over step `step`'s column (the drag stays in its row).
    CellMove {
        step: usize,
    },
    /// The mouse went up: the end of a drag.
    Release,
}

/// A drag on the grid: the row, the value the first cell took, the last step painted, and
/// whether it changed the pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Paint {
    column: usize,
    value: u8,
    last: usize,
    wrote: bool,
}

/// An editor's state, kept by the app.
#[derive(Clone, Debug, Default)]
pub struct DrumEditor {
    /// Whether the window is open.
    pub open: bool,
    /// The pattern shown (0..=31).
    pub pattern: u8,
    /// The 909's entry switch as set (Cycle at start; not saved).
    pub entry: Entry,
    /// The entry the modifiers held at the last frame select (the 909's), over `entry`.
    held: Option<Entry>,
    win: Placing,
    gesture: Option<Paint>,
    /// A drag of the Length value wrote already (it keeps the Undo of its first change).
    length_wrote: bool,
    /// Whether the mouse was down on the drum rows and on the AC row at the last frame.
    drums_down: bool,
    ac_down: bool,
    hits: Hits,
}

/// Where the window drew its parts last.
#[derive(Clone, Debug, Default)]
struct Hits {
    drums: Option<Rect>,
    ac: Option<Rect>,
    labels: Vec<&'static str>,
    buttons: Vec<(&'static str, Rect)>,
    playing: Option<usize>,
    dimmed: bool,
    /// The entry the 909's switch showed (`None` for the 808, which has none).
    entry: Option<Entry>,
}

impl DrumEditor {
    /// The entry a click enters now: the one the modifiers held select, else the switch's.
    pub fn entry_now(&self) -> Entry {
        self.held.unwrap_or(self.entry)
    }
}

/// The least width of a step column.
const COL_W: f32 = 27.0;

/// What the window shows of the song in a frame.
struct Shown {
    m: Machine,
    bytes: Vec<u8>,
    pattern: u8,
    playing: Option<usize>,
    paste: bool,
    labels: [&'static str; DRUMS],
}

impl App {
    /// The editor of drum machine `m`.
    pub fn drum_editor(&self, m: Machine) -> &DrumEditor {
        &self.drum_editors[m.index()]
    }

    /// Ctrl/Cmd+E: the editor of the focused row, the 808's with the TR-808 focused, the 909's
    /// with the TR-909 focused, else the 303 editor.
    pub fn open_focused_editor(&mut self) {
        match Machine::of_row(self.panel.focus) {
            Some(m) => self.open_drum_editor(m),
            None => self.open_pattern_editor(),
        }
    }

    /// Edit > 808 or 909 Pattern Editor... (and Ctrl/Cmd+E with its row focused): opens the
    /// window on the machine's active pattern, or brings it to the front and switches it to that
    /// pattern. Nothing without a song.
    pub fn open_drum_editor(&mut self, m: Machine) {
        if self.link.is_none() {
            return;
        }
        let pattern = self.panel.rows[m.row()].pattern;
        let e = &mut self.drum_editors[m.index()];
        e.pattern = pattern;
        e.win.open(e.open);
        e.open = true;
        e.gesture = None;
    }

    /// Where the window drew instrument `column`'s cell (0 the AC row, 1..=11 the drum rows) of
    /// step `step` last.
    pub fn drum_editor_cell(&self, m: Machine, column: usize, step: usize) -> Option<Rect> {
        let h = &self.drum_editor(m).hits;
        if column == AC {
            Some(cell_rect(h.ac?, step, 0))
        } else {
            Some(cell_rect(h.drums?, step, column - 1))
        }
    }

    /// Where the window was drawn last (`None` when closed).
    pub fn drum_editor_window(&self, m: Machine) -> Option<Rect> {
        let e = self.drum_editor(m);
        e.win.window.filter(|_| e.open)
    }

    /// Where the window drew its button or value `name` last ("A", "number", "number_prev",
    /// "Copy", "Length", "Shuffle", "shift_left", ...).
    pub fn drum_editor_hit(&self, m: Machine, name: &str) -> Option<Rect> {
        self.drum_editor(m)
            .hits
            .buttons
            .iter()
            .find(|(n, _)| *n == name)
            .map(|&(_, r)| r)
    }

    /// The drum rows' labels the window drew last, top to bottom, then "AC".
    pub fn drum_editor_labels(&self, m: Machine) -> &[&'static str] {
        &self.drum_editor(m).hits.labels
    }

    /// The playing step the grid showed last, and whether it dimmed the steps beyond the
    /// pattern's length.
    pub fn drum_editor_shown(&self, m: Machine) -> (Option<usize>, bool) {
        let h = &self.drum_editor(m).hits;
        (h.playing, h.dimmed)
    }

    /// The entry the 909's switch showed last (`None` for the 808 or before a frame).
    pub fn drum_editor_entry_shown(&self, m: Machine) -> Option<Entry> {
        self.drum_editor(m).hits.entry
    }

    /// The selection rule of pattern mode: the editor shows the machine's selected pattern (a
    /// selection on the panel moves it; so does the switch into pattern mode). In song mode it
    /// keeps its own.
    fn drum_follow(&mut self, m: Machine) {
        if !self.panel.transport.song_mode {
            self.drum_editors[m.index()].pattern = self.panel.rows[m.row()].pattern;
        }
    }

    /// The editor's pattern as the link holds it, after the panel's pending events were noted
    /// and sent; `None` without a song.
    fn drum_bytes(&mut self, m: Machine) -> Option<Vec<u8>> {
        self.link.as_ref()?;
        self.note_edits();
        let link = self.link.as_mut()?;
        link.forward(&mut self.panel);
        let p = self.drum_editors[m.index()].pattern;
        Some(link.pattern(m.row(), p).to_vec())
    }

    /// Writes `f` of the editor's pattern for the command `what`; with `keep` (a drag that
    /// wrote already) the Undo stays the one of the drag's first write. Whether it changed.
    fn drum_edit(
        &mut self,
        m: Machine,
        what: &'static str,
        keep: bool,
        f: impl FnOnce(&[u8]) -> Vec<u8>,
    ) -> bool {
        let Some(before) = self.drum_bytes(m) else {
            return false;
        };
        let (r, p) = (m.row(), self.drum_editors[m.index()].pattern);
        let after = f(&before);
        let prev = if keep { self.undo.clone() } else { None };
        let changed = self.write_pattern(r, p, &after, what);
        if changed
            && let Some(u) = prev
            && u.row == r
            && u.pattern == p
        {
            self.undo = Some(u);
        }
        changed
    }

    /// Runs what the window of drum machine `m` asked for.
    pub fn drum_editor_act(&mut self, m: Machine, a: DrumAct) {
        const EDIT: &str = "Edit Pattern";
        if self.link.is_none() {
            return;
        }
        let (r, p) = (m.row(), self.drum_editors[m.index()].pattern);
        match a {
            DrumAct::Select(pattern) => {
                let pattern = pattern.min(31);
                let e = &mut self.drum_editors[m.index()];
                e.pattern = pattern;
                e.gesture = None;
                if !self.panel.transport.song_mode && self.panel.rows[r].pattern != pattern {
                    self.select_pattern(r, pattern);
                }
            }
            DrumAct::Copy => {
                if self.drum_bytes(m).is_some() {
                    self.copy_pattern_at(r, p, false);
                }
            }
            DrumAct::Paste => {
                if self.drum_bytes(m).is_some() {
                    self.paste_pattern_at(r, p);
                }
            }
            DrumAct::Clear => {
                if self.drum_edit(m, "Clear Pattern", false, |b| emptied_pattern(r, b)) {
                    let name = crate::menu::pattern_name(p);
                    self.status = Some(format!("{}: {name} cleared", crate::menu::ROW_NAMES[r]));
                }
            }
            DrumAct::Length { n, drag } => {
                let keep = drag && self.drum_editors[m.index()].length_wrote;
                let changed = self.drum_edit(m, "Pattern Length", keep, |b| {
                    crate::pattern_editor::with_length(b, n)
                });
                self.drum_editors[m.index()].length_wrote = drag && (keep || changed);
            }
            DrumAct::Shuffle => {
                self.drum_edit(m, "Pattern Shuffle", false, |b| {
                    crate::pattern_editor::with_shuffle(b, b[0] == 0)
                });
            }
            DrumAct::Entry(entry) => self.drum_editors[m.index()].entry = entry,
            DrumAct::Shift { right } => {
                self.drum_edit(m, "Shift Pattern", false, |b| shifted(b, right));
            }
            DrumAct::CellDown { column, step } => {
                let Some(bytes) = self.drum_bytes(m) else {
                    return;
                };
                let entry = self.drum_editors[m.index()].entry_now();
                let value = m.clicked(column, cell(&bytes, step, column), entry);
                let wrote = self.drum_edit(m, EDIT, false, |b| m.put(b, step, column, value));
                self.drum_editors[m.index()].gesture = Some(Paint {
                    column,
                    value,
                    last: step,
                    wrote,
                });
            }
            DrumAct::CellMove { step } => {
                let Some(g) = self.drum_editors[m.index()].gesture else {
                    return;
                };
                if g.last == step {
                    return;
                }
                let changed = self.drum_edit(m, EDIT, g.wrote, |b| {
                    (g.last.min(step)..=g.last.max(step))
                        .fold(b.to_vec(), |b, s| m.put(&b, s, g.column, g.value))
                });
                self.drum_editors[m.index()].gesture = Some(Paint {
                    last: step,
                    wrote: g.wrote || changed,
                    ..g
                });
            }
            DrumAct::Release => self.drum_editors[m.index()].gesture = None,
        }
    }

    /// Draws the drum editors' windows that are open, and runs what they ask for.
    pub(crate) fn drum_editors_ui(&mut self, ctx: &Context) {
        for m in Machine::ALL {
            self.drum_editor_ui(ctx, m);
        }
    }

    fn drum_editor_ui(&mut self, ctx: &Context, m: Machine) {
        let i = m.index();
        if !self.drum_editors[i].open {
            return;
        }
        if self.link.is_none() {
            self.drum_editors[i].open = false;
            return;
        }
        self.drum_follow(m);
        let Some(link) = &self.link else {
            return;
        };
        let (r, p) = (m.row(), self.drum_editors[i].pattern);
        let playing = match m {
            Machine::Tr808 => self.panel.playing_step808,
            Machine::Tr909 => self.panel.playing_step909,
        };
        let shown = Shown {
            m,
            bytes: link.pattern(r, p).to_vec(),
            pattern: p,
            playing: playing.filter(|_| self.panel.rows[r].pattern == p),
            paste: self.clip.as_ref().is_some_and(|c| c.row == r),
            labels: m.labels(&self.panel),
        };
        let plugin = self.is_plugin();
        let mut acts = Vec::new();
        let mut length_drag = false;
        let mut win = std::mem::take(&mut self.drum_editors[i].win);
        let editor = &mut self.drum_editors[i];
        let spec = frame::Spec {
            id: egui::Id::unique(m.salt()),
            title: m.title(),
            saved: (!plugin).then(|| m.saved(&self.settings)).flatten(),
            guess: vec2(520.0, if m.has_entry() { 385.0 } else { 360.0 }),
            keep: !plugin,
        };
        let out = frame::show_window(ctx, spec, &mut win, |ui| {
            length_drag = window_ui(ui, editor, &shown, &mut acts);
        });
        self.drum_editors[i].win = win;
        if !length_drag {
            self.drum_editors[i].length_wrote = false;
        }
        for a in acts {
            self.drum_editor_act(m, a);
        }
        if let Some(pos) = out.save {
            m.save(&mut self.settings, pos);
            if let Err(e) = self.settings.save() {
                self.fail(e);
            }
        }
        if !out.open {
            self.drum_editors[i].open = false;
            self.drum_editors[i].gesture = None;
        }
    }
}

/// The window's content; what it asks for goes to `acts`. Whether the Length value is being
/// dragged.
fn window_ui(ui: &mut Ui, e: &mut DrumEditor, s: &Shown, acts: &mut Vec<DrumAct>) -> bool {
    e.hits.buttons.clear();
    let mut length_drag = false;
    let bank = s.pattern / 8;
    // ---- the top row ----
    ui.horizontal(|ui| {
        frame::compact(ui);
        if let Some(b) = frame::bank_buttons(ui, s.pattern, &mut e.hits.buttons) {
            acts.push(DrumAct::Select(b * 8 + s.pattern % 8));
        }
        // the number between its arrows, which step through A1..D8 and stop at the ends
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
        if let Some(p) = to {
            acts.push(DrumAct::Select(p));
        }
        ui.separator();
        match frame::edit_buttons(ui, s.paste, &mut e.hits.buttons) {
            Some("Copy") => acts.push(DrumAct::Copy),
            Some("Paste") => acts.push(DrumAct::Paste),
            Some(_) => acts.push(DrumAct::Clear),
            None => {}
        }
        ui.separator();
        let h = frame::length_shuffle(ui, &s.bytes, &mut e.hits.buttons);
        length_drag = h.length_drag;
        if let Some((n, drag)) = h.length {
            acts.push(DrumAct::Length { n, drag });
        }
        if h.shuffle {
            acts.push(DrumAct::Shuffle);
        }
    });
    // ---- the 909's entry switch, a second header row (the top row has no room left on the
    // smallest panel) ----
    e.hits.entry = None;
    if s.m.has_entry() {
        e.held = Entry::held(ui.input(|i| i.modifiers));
        let now = e.entry_now();
        e.hits.entry = Some(now);
        let is_mac = ui.ctx().os().is_mac();
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            frame::compact(ui);
            // the segments alone, no label (the user's review)
            for x in Entry::ALL {
                let mut text = RichText::new(x.name());
                if let Some(c) = x.color() {
                    text = text.color(c);
                }
                let r = ui
                    .selectable_label(now == x, text)
                    .on_hover_text(entry_tip(x, is_mac));
                e.hits.buttons.push((x.name(), r.rect));
                if r.clicked() {
                    acts.push(DrumAct::Entry(x));
                }
            }
        });
    }
    ui.add_space(6.0);

    let len = s.bytes[1].clamp(1, STEPS as u8) as usize;
    // the step columns fill the width the top row gives
    let content_w = ui.min_rect().width();
    let grid_w = (content_w - KEY_W).max(COL_W * STEPS as f32);
    let cw = grid_w / STEPS as f32;
    let block_w = KEY_W + grid_w;
    // ---- the step numbers ----
    let (head, _) = ui.allocate_exact_size(vec2(block_w, HEAD_H), Sense::hover());
    frame::step_numbers(ui, head, head.left() + KEY_W, cw, len, s.playing);
    e.hits.playing = s.playing;
    e.hits.dimmed = len < STEPS;
    e.hits.labels.clear();

    // ---- the drum rows ----
    let painter = ui.painter().clone();
    let drums = rows_block(ui, e, s, block_w, grid_w, &s.labels, 1);
    e.hits.drums = Some(drums);
    let id = egui::Id::unique((s.m.salt(), "drums"));
    match frame::grid_press(ui, id, drums, DRUMS, &mut e.drums_down) {
        Some(Press::Down(step, row)) => acts.push(DrumAct::CellDown {
            column: row + 1,
            step,
        }),
        Some(Press::Held {
            column: Some(step), ..
        }) => acts.push(DrumAct::CellMove { step }),
        Some(Press::Release) => acts.push(DrumAct::Release),
        _ => {}
    }

    // ---- the divider, then the AC row ----
    let (split, _) = ui.allocate_exact_size(vec2(block_w, SPLIT_H), Sense::hover());
    frame::divider(&painter, split, drums.right());
    let ac = rows_block(ui, e, s, block_w, grid_w, &["AC"], AC);
    e.hits.ac = Some(ac);
    let id = egui::Id::unique((s.m.salt(), "ac"));
    match frame::grid_press(ui, id, ac, 1, &mut e.ac_down) {
        Some(Press::Down(step, _)) => acts.push(DrumAct::CellDown { column: AC, step }),
        Some(Press::Held {
            column: Some(step), ..
        }) => acts.push(DrumAct::CellMove { step }),
        Some(Press::Release) => acts.push(DrumAct::Release),
        _ => {}
    }

    // ---- Shift at the bottom, centered under the step grid ----
    ui.add_space(6.0);
    let (row, _) = ui.allocate_exact_size(vec2(block_w, ARROW.y), Sense::hover());
    let mid = drums.center().x;
    let left = Rect::from_min_size(pos2(mid - 2.0 - ARROW.x, row.top()), ARROW);
    let right = Rect::from_min_size(pos2(mid + 2.0, row.top()), ARROW);
    let b = &mut e.hits.buttons;
    if frame::arrow_button(ui, left, vec2(-1.0, 0.0), b, s.m.salt(), "shift_left") {
        acts.push(DrumAct::Shift { right: false });
    }
    if frame::arrow_button(ui, right, vec2(1.0, 0.0), b, s.m.salt(), "shift_right") {
        acts.push(DrumAct::Shift { right: true });
    }
    length_drag
}

/// The tooltip of an entry switch's button (the user's text, review of task-281), with the
/// platform's key that holds it ("Hold Cmd" on macOS, "Hold Ctrl" elsewhere, ...).
pub fn entry_tip(x: Entry, is_mac: bool) -> String {
    let m = Machine::Tr909;
    let hold = |i: Input, value: &str| {
        format!(
            "Click sets cell to {value}. {} to activate",
            i.keys(m, is_mac)
        )
    };
    match x {
        Entry::Cycle => "Click sets cell off -> low -> high -> off".into(),
        Entry::Low => hold(Input::HoldLow, "low"),
        Entry::High => hold(Input::HoldHigh, "high"),
        Entry::Flam => hold(Input::HoldFlam, "flam"),
    }
}

/// A block of rows labeled `labels` (each with its full name as a tooltip) over the 16 step
/// columns, the first row instrument `first` (the next rows the next instruments); its grid's
/// rect.
fn rows_block(
    ui: &mut Ui,
    e: &mut DrumEditor,
    s: &Shown,
    block_w: f32,
    grid_w: f32,
    labels: &[&'static str],
    first: usize,
) -> Rect {
    let n = labels.len();
    let (block, _) = ui.allocate_exact_size(vec2(block_w, CELL_H * n as f32), Sense::hover());
    let grid = Rect::from_min_size(
        pos2(block.left() + KEY_W, block.top()),
        vec2(grid_w, block.height()),
    );
    let painter = ui.painter().clone();
    let len = s.bytes[1].clamp(1, STEPS as u8) as usize;
    for (r, &label) in labels.iter().enumerate() {
        let y = block.top() + r as f32 * CELL_H;
        let lrect = Rect::from_min_size(pos2(block.left(), y), Vec2::new(KEY_W, CELL_H));
        painter.text(
            pos2(lrect.right() - 6.0, lrect.center().y),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::proportional(LABEL_SIZE),
            INK_DIM,
        );
        ui.interact(
            lrect,
            egui::Id::unique((s.m.salt(), "label", first + r)),
            Sense::hover(),
        )
        .on_hover_text(label_name(label));
        e.hits.labels.push(label);
        let row = Rect::from_min_size(pos2(grid.left(), y), vec2(grid_w, CELL_H));
        let fill: Color32 = if r % 2 == 0 { SLATE_DARK } else { BG };
        painter.rect_filled(row, 0.0, fill);
    }
    paint_columns(&painter, grid, len, s.playing);
    for r in 0..n {
        for st in 0..STEPS {
            if let Some(c) = s.m.color(cell(&s.bytes, st, first + r)) {
                let rect = cell_rect(grid, st, r).shrink2(vec2(3.0, 3.0));
                painter.rect_filled(rect, CornerRadius::same(2), dimmed(c, st >= len));
            }
        }
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty() -> Vec<u8> {
        let mut b = vec![0u8; PATTERN_BYTES];
        b[1] = 16;
        b
    }

    #[test]
    fn a_cell_is_one_byte_of_a_step_in_rbs_order() {
        let b = with_cell(&empty(), 3, 1, 1);
        assert_eq!(b[2 + 12 * 3 + 1], 1);
        assert_eq!(cell(&b, 3, 1), 1);
        let b = with_cell(&b, 15, AC, 1);
        assert_eq!(b[2 + 12 * 15], 1);
        assert_eq!(b.iter().filter(|&&v| v != 0).count(), 3, "and the length");
    }

    #[test]
    fn a_click_on_the_808_toggles() {
        let m = Machine::Tr808;
        for e in Entry::ALL {
            assert_eq!(m.clicked(1, 0, e), 1);
            assert_eq!(m.clicked(1, 1, e), 0);
            assert_eq!(m.clicked(AC, 0, e), 1);
            // a byte above 1 is on: a click clears it
            assert_eq!(m.clicked(5, 7, e), 0);
        }
        assert_eq!(m.color(0), None);
        assert_eq!(m.color(1), Some(AMBER));
        // no hi-hat rule on the 808
        let b = m.put(&with_cell(&empty(), 2, OH909, 1), 2, CH909, 1);
        assert_eq!((cell(&b, 2, CH909), cell(&b, 2, OH909)), (1, 1));
    }

    #[test]
    fn a_click_on_the_909_follows_the_entry() {
        let m = Machine::Tr909;
        // Cycle: off -> low -> high -> off; flam (and a byte above 3, shown as flam) -> low
        let cycle: Vec<u8> = [0, 1, 2, 3, 4, 200]
            .iter()
            .map(|&v| m.clicked(1, v, Entry::Cycle))
            .collect();
        assert_eq!(cycle, [1, 2, 0, 1, 1, 1]);
        // Low / High / Flam: set, clear the same, replace another in one click
        for (e, v) in [(Entry::Low, 1), (Entry::High, 2), (Entry::Flam, 3)] {
            for c in 0..=3 {
                let want = if c == v { 0 } else { v };
                assert_eq!(m.clicked(4, c, e), want, "{e:?} on {c}");
            }
        }
        // a byte above 3 shows as flam: Flam clears it, Low replaces it
        assert_eq!(m.clicked(4, 9, Entry::Flam), 0);
        assert_eq!(m.clicked(4, 9, Entry::Low), 1);
        // the AC row toggles 0 <-> 2 whatever the entry
        for e in Entry::ALL {
            assert_eq!(m.clicked(AC, 0, e), 2, "{e:?}");
            assert_eq!(m.clicked(AC, 2, e), 0, "{e:?}");
            assert_eq!(m.clicked(AC, 1, e), 0, "{e:?}");
        }
        // the colors; above 3 as flam
        let colors: Vec<_> = [0, 1, 2, 3, 4, 255].iter().map(|&v| m.color(v)).collect();
        assert_eq!(
            colors,
            [
                None,
                Some(LOW),
                Some(HIGH),
                Some(FLAM),
                Some(FLAM),
                Some(FLAM)
            ]
        );
    }

    #[test]
    fn a_909_hi_hat_entered_clears_the_other_on_its_step() {
        let m = Machine::Tr909;
        let b = with_cell(&with_cell(&empty(), 5, OH909, 2), 6, OH909, 1);
        let b = m.put(&b, 5, CH909, 1);
        assert_eq!((cell(&b, 5, CH909), cell(&b, 5, OH909)), (1, 0));
        assert_eq!(cell(&b, 6, OH909), 1, "another step stays");
        let b = m.put(&b, 6, CH909, 3);
        let b = m.put(&b, 6, OH909, 2);
        assert_eq!((cell(&b, 6, CH909), cell(&b, 6, OH909)), (0, 2));
        // clearing one leaves the other; another row clears nothing
        let both = with_cell(&with_cell(&empty(), 0, CH909, 1), 0, OH909, 1);
        let c = m.put(&both, 0, CH909, 0);
        assert_eq!(cell(&c, 0, OH909), 1);
        let c = m.put(&both, 0, 10, 1);
        assert_eq!((cell(&c, 0, CH909), cell(&c, 0, OH909)), (1, 1));
    }

    #[test]
    fn the_modifiers_select_the_entry_while_held() {
        use egui::Modifiers;
        let mac_cmd = Modifiers {
            mac_cmd: true,
            command: true,
            ..Modifiers::NONE
        };
        let win_ctrl = Modifiers {
            ctrl: true,
            command: true,
            ..Modifiers::NONE
        };
        assert_eq!(Entry::held(Modifiers::NONE), None);
        assert_eq!(Entry::held(mac_cmd), Some(Entry::Low));
        assert_eq!(Entry::held(win_ctrl), Some(Entry::Low));
        assert_eq!(Entry::held(Modifiers::ALT), Some(Entry::High));
        assert_eq!(Entry::held(mac_cmd | Modifiers::ALT), Some(Entry::Flam));
        assert_eq!(Entry::held(win_ctrl | Modifiers::ALT), Some(Entry::Flam));
        // macOS's Ctrl and Shift select nothing
        assert_eq!(Entry::held(Modifiers::CTRL), None);
        assert_eq!(Entry::held(Modifiers::SHIFT), None);
        // the held entry enters what the panel's step buttons enter with those keys
        for mods in [mac_cmd, Modifiers::ALT, win_ctrl | Modifiers::ALT] {
            let e = Entry::held(mods).unwrap();
            for c in 0..=3 {
                assert_eq!(
                    Machine::Tr909.clicked(3, c, e),
                    behavior::tr909_step_value(c, false, false, behavior::step_mods(mods))
                );
            }
        }
    }

    #[test]
    fn the_909_rows_are_its_instruments_in_rbs_order() {
        let p = Panel::new();
        assert_eq!(
            Machine::Tr909.labels(&p),
            [
                "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CH", "OH", "CC", "RC"
            ]
        );
        assert_eq!(Machine::Tr909.labels(&p)[CH909 - 1], "CH");
        assert_eq!(Machine::Tr909.labels(&p)[OH909 - 1], "OH");
        for l in Machine::Tr909.labels(&p) {
            assert!(!label_name(l).is_empty(), "{l}");
        }
        assert_eq!(Machine::of_row(3), Some(Machine::Tr909));
        assert_eq!(Machine::of_row(2), Some(Machine::Tr808));
        assert_eq!(Machine::of_row(1), None);
    }

    #[test]
    fn shift_wraps_within_the_length_and_moves_whole_steps() {
        let mut b = crate::pattern_editor::with_length(&empty(), 4);
        for s in 0..STEPS {
            b = with_cell(&b, s, s % 12, 1);
        }
        let r = shifted(&b, true);
        // step 0 takes step 3, whole
        assert_eq!(&r[at(0, 0)..at(0, 0) + 12], &b[at(3, 0)..at(3, 0) + 12]);
        assert_eq!(&r[at(1, 0)..at(1, 0) + 12], &b[at(0, 0)..at(0, 0) + 12]);
        assert_eq!(&r[at(4, 0)..], &b[at(4, 0)..], "beyond the length stays");
        assert_eq!(shifted(&r, false), b, "left undoes right");
    }

    #[test]
    fn the_number_arrows_run_a1_to_d8_and_stop_at_the_ends() {
        assert_eq!(step_pattern(7, true), Some(8), "A8 to B1");
        assert_eq!(step_pattern(8, false), Some(7));
        assert_eq!(step_pattern(0, false), None);
        assert_eq!(step_pattern(31, true), None);
    }

    #[test]
    fn the_808_labels_follow_the_slot_switches() {
        let mut p = Panel::new();
        let m = Machine::Tr808;
        // the switches start down: the lower legend, LT MT HT RS CP
        assert_eq!(
            m.labels(&p),
            [
                "BD", "SD", "LT", "MT", "HT", "RS", "CP", "CB", "CY", "OH", "CH"
            ]
        );
        for k in 3..=7 {
            p.set_switch(&format!("tr808.slot{k}_switch"), 0);
        }
        assert_eq!(
            m.labels(&p),
            [
                "BD", "SD", "LC", "MC", "HC", "CL", "MA", "CB", "CY", "OH", "CH"
            ]
        );
        p.set_switch("tr808.slot5_switch", 1);
        assert_eq!(m.labels(&p)[4], "HT");
        for l in m
            .labels(&p)
            .iter()
            .chain(["LT", "MT", "RS", "CP", "AC"].iter())
        {
            assert!(!label_name(l).is_empty(), "{l}");
        }
    }

    #[test]
    fn the_entry_switchs_tooltips_are_the_users_text_with_the_platforms_keys() {
        let tips = |is_mac| Entry::ALL.map(|x| entry_tip(x, is_mac));
        assert_eq!(
            tips(true),
            [
                "Click sets cell off -> low -> high -> off",
                "Click sets cell to low. Hold Cmd to activate",
                "Click sets cell to high. Hold Option to activate",
                "Click sets cell to flam. Hold Cmd+Option to activate",
            ]
        );
        assert_eq!(
            tips(false),
            [
                "Click sets cell off -> low -> high -> off",
                "Click sets cell to low. Hold Ctrl to activate",
                "Click sets cell to high. Hold Alt to activate",
                "Click sets cell to flam. Hold Ctrl+Alt to activate",
            ]
        );
    }

    #[test]
    fn the_help_texts_name_the_platforms_keys() {
        let (m8, m9) = (Machine::Tr808, Machine::Tr909);
        assert_eq!(Input::OpenKey.keys(m8, true), "Cmd+E");
        assert_eq!(Input::OpenKey.keys(m9, false), "Ctrl+E");
        assert_eq!(Input::Undo.keys(m8, false), "Ctrl+Z");
        assert!(Input::Modified.keys(m8, true).contains("Option"));
        assert!(Input::Modified.keys(m8, false).contains("Alt"));
        // the 909's held entries
        assert_eq!(Input::HoldLow.keys(m9, true), "Hold Cmd");
        assert_eq!(Input::HoldLow.keys(m9, false), "Hold Ctrl");
        assert_eq!(Input::HoldHigh.keys(m9, true), "Hold Option");
        assert_eq!(Input::HoldHigh.keys(m9, false), "Hold Alt");
        assert_eq!(Input::HoldFlam.keys(m9, true), "Hold Cmd+Option");
        assert_eq!(Input::HoldFlam.keys(m9, false), "Hold Ctrl+Alt");
        // the other modifiers: no Cmd, Option or Alt (they are the entries')
        assert_eq!(
            Input::Modified.keys(m9, true),
            "Shift / Ctrl + click or drag"
        );
        assert_eq!(Input::Modified.keys(m9, false), "Shift + click or drag");
        assert!(Input::OpenKey.what(m9).contains("TR-909"));
        for m in Machine::ALL {
            for &i in m.inputs() {
                assert!(!i.what(m).is_empty());
            }
        }
    }
}
