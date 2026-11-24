//! Help > Keyboard Shortcuts (task-105): a window that lists every key orb-ui handles, by group,
//! with the current keyboard mode marked.
//!
//! The list is one table ([`groups`]). Each row carries, besides its text, the [`Bind`]s it
//! stands for: a key, the modes and focus rows it is active in and the [`keys::Action`] it
//! must give. The tests run every bind through [`keys::map`] and run every key `keys::map`
//! gives an action through the binds, so the window cannot drift from the bindings: a changed
//! action fails the first, a new or removed binding the second. The menu shortcuts come from
//! [`crate::shortcuts`], the one definition of them.

use egui::{self, Color32, Context, Key, KeyboardShortcut, Modifiers, RichText};

use crate::app::App;
use crate::drum_editor;
use crate::keys::{Action, KeyMode};
use crate::palette::{INK, INK_DIM};
use crate::shortcuts;

const CURRENT: Color32 = Color32::from_rgb(0xf0, 0xb4, 0x4c);

/// Where a bind is active: the keyboard mode and the focused row (0 and 1 are the 303s, 2 and
/// 3 the 808 and the 909).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Both modes, any row.
    Always,
    /// Program Synth mode, any row.
    Synth,
    /// Select Patterns mode, any row.
    Patterns,
    /// Program Synth mode, a 303 focused.
    Synth303,
    /// Program Synth mode, a drum machine focused.
    SynthDrums,
    /// Select Patterns mode, or Program Synth mode with a drum machine focused (where Enter
    /// and `/` are the transport's).
    NotSynth303,
}

impl Scope {
    /// Every (mode, focused row) the scope covers.
    pub fn cases(self) -> Vec<(KeyMode, usize)> {
        let mut v = Vec::new();
        for mode in [KeyMode::Synth, KeyMode::Patterns] {
            for focus in 0..4 {
                let on = match self {
                    Scope::Always => true,
                    Scope::Synth => mode == KeyMode::Synth,
                    Scope::Patterns => mode == KeyMode::Patterns,
                    Scope::Synth303 => mode == KeyMode::Synth && focus < 2,
                    Scope::SynthDrums => mode == KeyMode::Synth && focus >= 2,
                    Scope::NotSynth303 => !(mode == KeyMode::Synth && focus < 2),
                };
                if on {
                    v.push((mode, focus));
                }
            }
        }
        v
    }
}

/// Which Shift states a bind holds for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    /// With and without Shift.
    Any,
    /// Without Shift.
    No,
    /// With Shift.
    Yes,
}

#[cfg(test)]
impl Shift {
    fn states(self) -> &'static [bool] {
        match self {
            Shift::Any => &[false, true],
            Shift::No => &[false],
            Shift::Yes => &[true],
        }
    }
}

/// What a bind must give from [`keys::map`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Want {
    /// A press of this layout id; `{r}` is the focused 303 number (1 or 2).
    Id(String),
    Arrow(&'static str),
    PlayStop,
    Locate(bool),
    Focus(i32),
    Tap303(bool),
    Drum(usize),
    /// No action: the key does nothing there.
    Nothing,
}

impl Want {
    /// The action for a focused row (`None`: the key does nothing).
    pub fn action(&self, focus: usize) -> Option<Action> {
        Some(match self {
            Want::Id(id) => Action::Press(id.replace("{r}", &(focus + 1).to_string())),
            Want::Arrow(a) => Action::Arrow(a),
            Want::PlayStop => Action::PlayStop,
            Want::Locate(end) => Action::Locate(*end),
            Want::Focus(d) => Action::Focus(*d),
            Want::Tap303(rest) => Action::Tap303(*rest),
            Want::Drum(i) => Action::TapDrum(*i),
            Want::Nothing => return None,
        })
    }
}

/// One key and what [`keys::map`] must make of it.
#[derive(Clone, Debug)]
pub struct Bind {
    pub key: Key,
    pub shift: Shift,
    pub scope: Scope,
    pub want: Want,
}

fn bind(key: Key, scope: Scope, want: Want) -> Bind {
    Bind {
        key,
        shift: Shift::Any,
        scope,
        want,
    }
}

fn id(s: &str) -> Want {
    Want::Id(s.into())
}

impl Bind {
    fn shift(mut self, shift: Shift) -> Self {
        self.shift = shift;
        self
    }
}

/// A line of the window.
#[derive(Clone, Debug)]
pub struct Row {
    /// The keys, as written in the window.
    pub keys: String,
    /// What they do.
    pub what: &'static str,
    /// The bindings the line stands for (empty for a menu shortcut and for a plain note).
    pub binds: Vec<Bind>,
    /// The drum editor and its input the line stands for (tasks 280 and 281;
    /// tests/drum_editor.rs runs each).
    pub editor: Option<(drum_editor::Machine, drum_editor::Input)>,
}

/// The mouse on the panel's TR-909 step buttons (docs/ui_spec.md section 7; task-281 made the
/// original's Ctrl the command key, Cmd on macOS): each is a line of the help, and its test runs
/// each line's modifiers through the panel's rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step909 {
    Click,
    Low,
    High,
    Flam,
    Accent,
    Drag,
}

impl Step909 {
    pub const ALL: [Step909; 6] = [
        Step909::Click,
        Step909::Low,
        Step909::High,
        Step909::Flam,
        Step909::Accent,
        Step909::Drag,
    ];

    /// The keys and the mouse, with the platform's key names.
    pub fn keys(self, is_mac: bool) -> String {
        let (cmd, alt) = if is_mac {
            ("Cmd", "Option")
        } else {
            ("Ctrl", "Alt")
        };
        match self {
            Step909::Click => "Click".into(),
            Step909::Low => format!("{cmd}+click"),
            Step909::High => format!("{alt}+click"),
            Step909::Flam => format!("{cmd}+{alt}+click"),
            Step909::Accent => "Click, AC selected".into(),
            Step909::Drag => "Drag".into(),
        }
    }

    pub fn what(self) -> &'static str {
        match self {
            Step909::Click => "The selected instrument's step: off, low, high, off",
            Step909::Low => "Set low, or clear a low step",
            Step909::High => "Set high, or clear a high step",
            Step909::Flam => "Set flam, or clear a flam step (so does a click with Flam on)",
            Step909::Accent => "The accent on or off, whatever the modifiers",
            Step909::Drag => "Set every step crossed to the first step's new value",
        }
    }
}

/// A titled block of lines.
#[derive(Clone, Debug)]
pub struct Group {
    pub title: &'static str,
    /// The keyboard mode the group is about: it is marked when it is the current one.
    pub mode: Option<KeyMode>,
    pub note: &'static str,
    pub rows: Vec<Row>,
}

fn row(keys: &str, what: &'static str, binds: Vec<Bind>) -> Row {
    Row {
        keys: keys.into(),
        what,
        binds,
        editor: None,
    }
}

/// The line of a drum editor's input, with the platform's key names.
fn editor_row(is_mac: bool, m: drum_editor::Machine, i: drum_editor::Input) -> Row {
    Row {
        keys: i.keys(m, is_mac),
        what: i.what(m),
        binds: vec![],
        editor: Some((m, i)),
    }
}

/// The group of a drum editor: every input it handles.
fn editor_group(is_mac: bool, m: drum_editor::Machine) -> Group {
    Group {
        title: m.title(),
        mode: None,
        note: "Every key and mouse action of the editor's window; the panel's keys work as \
               without it, except while a value is typed.",
        rows: m
            .inputs()
            .iter()
            .map(|&i| editor_row(is_mac, m, i))
            .collect(),
    }
}

fn menu_row(is_mac: bool, sc: &KeyboardShortcut, what: &'static str) -> Row {
    row(&shortcuts::text_for(sc, is_mac), what, vec![])
}

/// The 303 pitch keys in note order (`keys.rs` `PITCH`).
const PITCH: [Key; 13] = [
    Key::C,
    Key::F,
    Key::V,
    Key::G,
    Key::B,
    Key::N,
    Key::J,
    Key::M,
    Key::K,
    Key::Comma,
    Key::L,
    Key::Period,
    Key::Slash,
];

/// The drum instrument keys in instrument order, 1..=11 (`keys.rs` `DRUM`).
const DRUM: [Key; 11] = [
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::G,
    Key::H,
    Key::J,
    Key::K,
    Key::L,
    Key::Semicolon,
    Key::Quote,
];

/// The pattern keys of the four rows (`keys.rs` `PATTERN`).
const PATTERN: [[Key; 8]; 4] = [
    [
        Key::Num1,
        Key::Num2,
        Key::Num3,
        Key::Num4,
        Key::Num5,
        Key::Num6,
        Key::Num7,
        Key::Num8,
    ],
    [
        Key::Q,
        Key::W,
        Key::E,
        Key::R,
        Key::T,
        Key::Y,
        Key::U,
        Key::I,
    ],
    [
        Key::A,
        Key::S,
        Key::D,
        Key::F,
        Key::G,
        Key::H,
        Key::J,
        Key::K,
    ],
    [
        Key::Z,
        Key::X,
        Key::C,
        Key::V,
        Key::B,
        Key::N,
        Key::M,
        Key::Comma,
    ],
];

/// All the groups of the window, for a platform (`is_mac`: Cmd instead of Ctrl).
pub fn groups(is_mac: bool) -> Vec<Group> {
    let s303 = Scope::Synth303;
    let pitch = PITCH
        .iter()
        .enumerate()
        .map(|(n, &k)| bind(k, s303, Want::Id(format!("tb303_{{r}}.key{n}"))))
        .collect();
    let drums = DRUM
        .iter()
        .enumerate()
        .map(|(n, &k)| bind(k, Scope::SynthDrums, Want::Drum(n + 1)))
        .collect();
    let patterns = |r: usize, what: &'static str, keys: &str| {
        let binds = PATTERN[r]
            .iter()
            .enumerate()
            .map(|(b, &k)| {
                bind(
                    k,
                    Scope::Patterns,
                    Want::Id(format!("pattern{}.pattern{}", r + 1, b + 1)),
                )
            })
            .collect();
        row(keys, what, binds)
    };
    let note_pause = |scope, want: Want| {
        vec![
            bind(Key::Minus, scope, want.clone()),
            bind(Key::Equals, scope, want).shift(Shift::No),
        ]
    };
    vec![
        Group {
            title: "Menu shortcuts",
            mode: None,
            note: "Keys with the command key held are left to the menu.",
            rows: vec![
                menu_row(is_mac, &shortcuts::NEW, "File > New"),
                menu_row(is_mac, &shortcuts::OPEN, "File > Open..."),
                menu_row(is_mac, &shortcuts::SAVE, "File > Save"),
                menu_row(is_mac, &shortcuts::UNDO, "Edit > Undo"),
                menu_row(is_mac, &shortcuts::CUT, "Edit > Cut Pattern"),
                menu_row(is_mac, &shortcuts::COPY, "Edit > Copy Pattern"),
                menu_row(is_mac, &shortcuts::PASTE, "Edit > Paste Pattern"),
                menu_row(is_mac, &shortcuts::DUPLICATE, "Edit > Duplicate Pattern"),
                menu_row(
                    is_mac,
                    &shortcuts::PATTERN_EDITOR,
                    "The focused row's pattern editor: Edit > 808 Pattern Editor... with the \
                     TR-808 focused, Edit > 909 Pattern Editor... with the TR-909 focused, \
                     else Edit > 303 Pattern Editor...",
                ),
                menu_row(
                    is_mac,
                    &shortcuts::KEYS_SYNTH,
                    "Options > Program Synth from Keyboard",
                ),
                menu_row(
                    is_mac,
                    &shortcuts::KEYS_PATTERNS,
                    "Options > Select Patterns from Keyboard",
                ),
            ],
        },
        Group {
            title: "Always active",
            mode: None,
            note: "Both keyboard modes, whatever row has the focus.",
            rows: vec![
                row(
                    "Space",
                    "Play if stopped, Stop otherwise",
                    vec![bind(Key::Space, Scope::Always, Want::PlayStop)],
                ),
                row(
                    "Up / Down",
                    "Move the focus to the previous / next row (wraps)",
                    vec![
                        bind(Key::ArrowUp, Scope::Always, Want::Focus(-1)),
                        bind(Key::ArrowDown, Scope::Always, Want::Focus(1)),
                    ],
                ),
                row(
                    "Insert",
                    "Record",
                    vec![bind(Key::Insert, Scope::Always, id("transport.record"))],
                ),
                row(
                    "Numpad 0 (and the main-row 0)",
                    "Stop",
                    vec![bind(Key::Num0, Scope::Always, id("transport.stop"))],
                ),
                row(
                    "Numpad +  or  Shift+=",
                    "Tempo up (held keys repeat)",
                    vec![
                        bind(Key::Plus, Scope::Always, Want::Arrow("transport.tempo_up")),
                        bind(
                            Key::Equals,
                            Scope::Always,
                            Want::Arrow("transport.tempo_up"),
                        )
                        .shift(Shift::Yes),
                    ],
                ),
                row(
                    "PgUp / PgDn",
                    "Nothing: the panel does not scroll",
                    vec![
                        bind(Key::PageUp, Scope::Always, Want::Nothing),
                        bind(Key::PageDown, Scope::Always, Want::Nothing),
                    ],
                ),
            ],
        },
        Group {
            title: "Transport keys that depend on the mode",
            mode: None,
            note: "egui cannot tell the numpad from the main keyboard, so these keys are \
                   resolved by the mode and the focus.",
            rows: vec![
                row(
                    "Enter",
                    "Play (in Program Synth mode only with a drum machine focused: with a \
                     303 focused it is Step)",
                    vec![bind(Key::Enter, Scope::NotSynth303, id("transport.play"))],
                ),
                row(
                    "/",
                    "Loop on / off (in Program Synth mode only with a drum machine \
                     focused: with a 303 focused it is the high C)",
                    vec![bind(Key::Slash, Scope::NotSynth303, id("transport.loop"))],
                ),
                row(
                    "-",
                    "Tempo down (held keys repeat). Select Patterns mode only: in \
                     Program Synth mode `-` is Note/Pause or AC, so tempo down has no key",
                    vec![bind(
                        Key::Minus,
                        Scope::Patterns,
                        Want::Arrow("transport.tempo_down"),
                    )],
                ),
                row(
                    "4 / 5",
                    "Rewind / Fast forward (Program Synth mode only)",
                    vec![
                        bind(Key::Num4, Scope::Synth, Want::Arrow("transport.rewind")),
                        bind(Key::Num5, Scope::Synth, Want::Arrow("transport.ff")),
                    ],
                ),
                row(
                    "7 / 8",
                    "Bar down / Bar up (Program Synth mode only)",
                    vec![
                        bind(Key::Num7, Scope::Synth, Want::Arrow("transport.bar_down")),
                        bind(Key::Num8, Scope::Synth, Want::Arrow("transport.bar_up")),
                    ],
                ),
                row(
                    "1 / 2",
                    "Locate to the loop start / end (Program Synth mode only)",
                    vec![
                        bind(Key::Num1, Scope::Synth, Want::Locate(false)),
                        bind(Key::Num2, Scope::Synth, Want::Locate(true)),
                    ],
                ),
                row(
                    "Numpad *",
                    "Nothing: egui gives no key for it (Insert records)",
                    vec![],
                ),
            ],
        },
        Group {
            title: "Program Synth mode, a TB-303 focused",
            mode: Some(KeyMode::Synth),
            note: "The keys press the focused 303's panel keys.",
            rows: vec![
                row(
                    "C F V G B N J M K , L . /",
                    "Piano keys C, C#, D, D#, E, F, F#, G, G#, A, A#, B, high C",
                    pitch,
                ),
                row(
                    "Enter",
                    "Step",
                    vec![bind(Key::Enter, s303, id("tb303_{r}.step"))],
                ),
                row(
                    "Backspace",
                    "Back",
                    vec![bind(Key::Backspace, s303, id("tb303_{r}.back"))],
                ),
                row(
                    "P",
                    "Accent",
                    vec![bind(Key::P, s303, id("tb303_{r}.accent"))],
                ),
                row(
                    "[",
                    "Slide",
                    vec![bind(Key::OpenBracket, s303, id("tb303_{r}.slide"))],
                ),
                row(
                    ";",
                    "Octave down",
                    vec![bind(Key::Semicolon, s303, id("tb303_{r}.down"))],
                ),
                row(
                    "'",
                    "Octave up",
                    vec![bind(Key::Quote, s303, id("tb303_{r}.up"))],
                ),
                row(
                    "-  or  =",
                    "Note / Pause",
                    note_pause(s303, id("tb303_{r}.note_pause")),
                ),
                row(
                    "Tab",
                    "The playing step becomes a note (tap recording)",
                    vec![bind(Key::Tab, s303, Want::Tap303(false)).shift(Shift::No)],
                ),
                row(
                    "Shift+Tab",
                    "The playing step becomes a rest",
                    vec![bind(Key::Tab, s303, Want::Tap303(true)).shift(Shift::Yes)],
                ),
            ],
        },
        Group {
            title: "Program Synth mode, a drum machine focused",
            mode: Some(KeyMode::Synth),
            note: "A key sets that instrument on the playing step (tap recording).",
            rows: vec![
                row(
                    "-  or  =",
                    "AC (accent)",
                    note_pause(Scope::SynthDrums, Want::Drum(0)),
                ),
                row(
                    "A S D F G H J K L ; '",
                    "Instruments 1..11. TR-808: BD SD LT MT HT RS CP CB CY OH CH. \
                     TR-909: BD SD LT MT HT RS CP CH OH CC RC",
                    drums,
                ),
            ],
        },
        Group {
            title: "Select Patterns mode",
            mode: Some(KeyMode::Patterns),
            note: "Each key row selects patterns 1..8 of one section, as a click on the \
                   pattern button.",
            rows: vec![
                patterns(0, "TB-303 #1", "1 2 3 4 5 6 7 8"),
                patterns(1, "TB-303 #2", "Q W E R T Y U I"),
                patterns(2, "TR-808", "A S D F G H J K"),
                patterns(3, "TR-909", "Z X C V B N M ,"),
            ],
        },
        Group {
            title: "The panel's TR-909 step buttons",
            mode: None,
            note: "The mouse on the TR-909's 16 step buttons; the 909 Pattern Editor's entry \
                   keys are the same.",
            rows: Step909::ALL
                .iter()
                .map(|&s| Row {
                    keys: s.keys(is_mac),
                    what: s.what(),
                    binds: vec![],
                    editor: None,
                })
                .collect(),
        },
        editor_group(is_mac, drum_editor::Machine::Tr808),
        editor_group(is_mac, drum_editor::Machine::Tr909),
        Group {
            title: "Windows and the file browser",
            mode: None,
            note: "Listed by hand from the browser's key handler; no test checks these lines.",
            rows: vec![
                row(
                    "Escape",
                    "Close the Info, About and this window; cancel the browser or the \
                     unsaved-changes prompt",
                    vec![],
                ),
                row(
                    "Up / Down, PgUp / PgDn",
                    "Move the browser's selection",
                    vec![],
                ),
                row("Home / End", "First / last entry of the browser", vec![]),
                row("Enter", "Open the selected entry", vec![]),
                row(
                    "Backspace  or  Alt+Up",
                    "The browser goes to the parent folder",
                    vec![],
                ),
                menu_row(is_mac, &shortcuts::EDIT_PATH, "Edit the browser's path"),
            ],
        },
        Group {
            title: "Prompts and questions",
            mode: None,
            note: "Listed by hand from the code (the prompt in menu.rs, the browser's question \
                   in browser.rs); no test checks these lines.",
            rows: vec![
                row("S  Y  Enter", "Unsaved-changes prompt: save", vec![]),
                row("D  N", "Unsaved-changes prompt: do not save", vec![]),
                row("Escape", "Unsaved-changes prompt: cancel", vec![]),
                row(
                    "Y",
                    "Overwrite question of the browser: replace the file",
                    vec![],
                ),
                row(
                    "Enter  Escape  N",
                    "Overwrite question of the browser: go back",
                    vec![],
                ),
            ],
        },
    ]
}

/// Whether a group is marked "(current mode)" while `current` is the keyboard mode.
pub fn is_current(g: &Group, current: KeyMode) -> bool {
    g.mode == Some(current)
}

/// The name of a mode as the window writes it.
fn mode_name(mode: KeyMode) -> &'static str {
    match mode {
        KeyMode::Synth => "Program Synth",
        KeyMode::Patterns => "Select Patterns",
    }
}

/// The id of the window of an open (`App::key_help_serial`).
pub fn key_help_window_id(serial: u32) -> egui::Id {
    egui::Id::unique(("orb_key_help_window", serial))
}

impl App {
    /// Help > Keyboard Shortcuts.
    pub fn show_key_help(&mut self) {
        if !self.key_help_open {
            // a new id: no stored place, so the window opens at its default, the center
            self.key_help_serial += 1;
        } else {
            self.raise_window(self.key_help_id());
        }
        self.key_help_open = true;
    }

    /// The id of the window now (for tests, to look its place up).
    pub fn key_help_id(&self) -> egui::Id {
        key_help_window_id(self.key_help_serial)
    }

    /// Whether the window is open.
    pub fn key_help_is_open(&self) -> bool {
        self.key_help_open
    }

    /// Draws the window when it is open. Escape and the close button close it.
    pub fn key_help_ui(&mut self, ctx: &Context) {
        if !self.key_help_open {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            self.key_help_open = false;
            return;
        }
        let mut open = true;
        let screen = ctx.content_rect();
        let size = egui::vec2(
            (screen.width() - 40.0).clamp(200.0, 520.0),
            (screen.height() - 40.0).clamp(150.0, 440.0),
        );
        let current = self.key_mode;
        // centered the way File > Info is (info.rs): pivot, default place, an id per open
        egui::Window::new("Keyboard Shortcuts")
            .id(self.key_help_id())
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            .default_size(size)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Keyboard mode now: {}", mode_name(current)))
                        .strong()
                        .color(CURRENT),
                );
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (n, g) in groups(ctx.os().is_mac()).iter().enumerate() {
                            ui.add_space(8.0);
                            let is_current = is_current(g, current);
                            let mut title = RichText::new(g.title).strong().size(15.0);
                            title = title.color(if is_current { CURRENT } else { INK });
                            ui.horizontal(|ui| {
                                ui.label(title);
                                if is_current {
                                    ui.label(RichText::new("(current mode)").color(CURRENT));
                                }
                            });
                            if !g.note.is_empty() {
                                ui.label(RichText::new(g.note).color(INK_DIM));
                            }
                            egui::Grid::new(("orb_key_help_grid", n))
                                .num_columns(2)
                                .show(ui, |ui| {
                                    for r in &g.rows {
                                        ui.label(RichText::new(&r.keys).monospace().color(INK));
                                        ui.add(egui::Label::new(r.what).wrap());
                                        ui.end_row();
                                    }
                                });
                        }
                    });
            });
        if !open {
            self.key_help_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys;

    fn binds(is_mac: bool) -> Vec<Bind> {
        groups(is_mac)
            .into_iter()
            .flat_map(|g| g.rows)
            .flat_map(|r| r.binds)
            .collect()
    }

    /// Every listed key does what the window says, in every mode and row the bind covers.
    #[test]
    fn every_listed_key_maps_to_its_action() {
        let all = binds(false);
        assert!(all.len() > 80, "{} binds", all.len());
        for b in &all {
            for &shift in b.shift.states() {
                for (mode, focus) in b.scope.cases() {
                    assert_eq!(
                        keys::map(b.key, shift, mode, focus),
                        b.want.action(focus),
                        "{:?} shift={shift} {mode:?} focus={focus}",
                        b.key
                    );
                }
            }
        }
    }

    /// Every key `keys::map` gives an action is listed with that action.
    #[test]
    fn every_mapped_key_is_listed() {
        let all = binds(false);
        let mut mapped = 0;
        for &key in Key::ALL {
            for shift in [false, true] {
                for mode in [KeyMode::Synth, KeyMode::Patterns] {
                    for focus in 0..4 {
                        let Some(action) = keys::map(key, shift, mode, focus) else {
                            continue;
                        };
                        mapped += 1;
                        let listed = all.iter().any(|b| {
                            b.key == key
                                && b.shift.states().contains(&shift)
                                && b.scope.cases().contains(&(mode, focus))
                                && b.want.action(focus).as_ref() == Some(&action)
                        });
                        assert!(
                            listed,
                            "{key:?} shift={shift} {mode:?} focus={focus} -> {action:?} is not \
                             listed in the Keyboard Shortcuts window"
                        );
                    }
                }
            }
        }
        assert!(mapped > 400, "{mapped} mapped cases");
    }

    /// No two binds claim the same (key, shift, mode, row) with different actions.
    #[test]
    fn the_binds_do_not_contradict_each_other() {
        let all = binds(false);
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                if a.key != b.key {
                    continue;
                }
                if !a
                    .shift
                    .states()
                    .iter()
                    .any(|s| b.shift.states().contains(s))
                {
                    continue;
                }
                for (mode, focus) in a.scope.cases() {
                    if b.scope.cases().contains(&(mode, focus)) {
                        assert_eq!(a.want.action(focus), b.want.action(focus), "{:?}", a.key);
                    }
                }
            }
        }
    }

    /// The menu rows carry the text of `shortcuts` for the platform.
    #[test]
    fn menu_rows_use_the_platform_text() {
        let text = |is_mac| -> Vec<String> {
            groups(is_mac)[0]
                .rows
                .iter()
                .map(|r| r.keys.clone())
                .collect()
        };
        let keys = ["N", "O", "S", "Z", "X", "C", "V", "D", "E", "F", "G"];
        assert_eq!(text(true), keys.map(|k| format!("Cmd+{k}")));
        assert_eq!(text(false), keys.map(|k| format!("Ctrl+{k}")));
    }

    /// Each drum editor's group (808 and 909) lists every input the editor handles
    /// (`Machine::inputs`, each of which tests/drum_editor.rs runs), once each and in order,
    /// with the platform's key names.
    #[test]
    fn each_drum_editor_group_lists_every_input_of_its_editor() {
        use drum_editor::{Input, Machine};
        for is_mac in [false, true] {
            let gs = groups(is_mac);
            for m in Machine::ALL {
                let g = gs.iter().find(|g| g.title == m.title()).expect("the group");
                let listed: Vec<_> = g.rows.iter().map(|r| r.editor).collect();
                let all: Vec<_> = m.inputs().iter().map(|&i| Some((m, i))).collect();
                assert_eq!(listed, all);
                for r in &g.rows {
                    let (_, i) = r.editor.unwrap();
                    assert_eq!(r.keys, i.keys(m, is_mac));
                    assert_eq!(r.what, i.what(m));
                }
                let keys: Vec<&str> = g.rows.iter().map(|r| r.keys.as_str()).collect();
                let (cmd, alt) = if is_mac {
                    ("Cmd", "Option")
                } else {
                    ("Ctrl", "Alt")
                };
                assert!(keys.contains(&format!("{cmd}+E").as_str()), "{keys:?}");
                assert!(keys.contains(&format!("{cmd}+Z").as_str()), "{keys:?}");
                // the open key is the menu's shortcut
                assert_eq!(
                    Input::OpenKey.keys(m, is_mac),
                    shortcuts::text_for(&shortcuts::PATTERN_EDITOR, is_mac)
                );
                assert_eq!(
                    Input::Undo.keys(m, is_mac),
                    shortcuts::text_for(&shortcuts::UNDO, is_mac)
                );
                // the 909's entry switch and its held keys, by the platform's names
                if m == Machine::Tr909 {
                    for k in [
                        format!("Hold {cmd}"),
                        format!("Hold {alt}"),
                        format!("Hold {cmd}+{alt}"),
                        "Click Cycle / Low / High / Flam".into(),
                    ] {
                        assert!(keys.contains(&k.as_str()), "{k}: {keys:?}");
                    }
                }
            }
            // no other group lists an editor input
            let titles = Machine::ALL.map(|m| m.title());
            let elsewhere = gs
                .iter()
                .filter(|o| !titles.contains(&o.title))
                .flat_map(|o| &o.rows)
                .any(|r| r.editor.is_some());
            assert!(!elsewhere);
        }
    }

    /// The panel's 909 step group lists each mouse action with the platform's key names, and
    /// the modifiers each line names give, through the panel's reading of them
    /// (`behavior::step_mods`, as egui reports the keys on that platform), what the line says.
    #[test]
    fn the_panels_909_step_group_names_the_keys_that_do_it() {
        use crate::behavior::{step_mods, tr909_step_value};
        for is_mac in [false, true] {
            let gs = groups(is_mac);
            let g = gs
                .iter()
                .find(|g| g.title == "The panel's TR-909 step buttons")
                .expect("the group");
            let keys: Vec<String> = g.rows.iter().map(|r| r.keys.clone()).collect();
            let want: Vec<String> = Step909::ALL.iter().map(|s| s.keys(is_mac)).collect();
            assert_eq!(keys, want);
            // egui's modifiers of the command key on the platform, and of Alt / Option
            let cmd = if is_mac {
                Modifiers {
                    mac_cmd: true,
                    command: true,
                    ..Modifiers::NONE
                }
            } else {
                Modifiers {
                    ctrl: true,
                    command: true,
                    ..Modifiers::NONE
                }
            };
            let (cmd_name, alt_name) = if is_mac {
                ("Cmd", "Option")
            } else {
                ("Ctrl", "Alt")
            };
            for s in Step909::ALL {
                let k = s.keys(is_mac);
                let mods = match s {
                    Step909::Low => cmd,
                    Step909::High => Modifiers::ALT,
                    Step909::Flam => cmd | Modifiers::ALT,
                    _ => Modifiers::NONE,
                };
                assert_eq!(k.contains(cmd_name), mods.command, "{k}");
                assert_eq!(k.contains(alt_name), mods.alt, "{k}");
                let m = step_mods(mods);
                let toggle = |v: u8, c: u8| if c == v { 0 } else { v };
                for c in 0..=3u8 {
                    let got = tr909_step_value(c, s == Step909::Accent, false, m);
                    let want = match s {
                        Step909::Click | Step909::Drag => (c + 1) % 3,
                        Step909::Low => toggle(1, c),
                        Step909::High => toggle(2, c),
                        Step909::Flam => toggle(3, c),
                        Step909::Accent => {
                            if c == 0 {
                                2
                            } else {
                                0
                            }
                        }
                    };
                    assert_eq!(got, want, "{k} on {c}");
                }
            }
        }
    }

    /// The Theme Viewer and its Space key are gone (task-267): no row names them.
    #[test]
    fn no_row_names_the_theme_viewer() {
        for g in groups(false).iter().chain(groups(true).iter()) {
            for r in &g.rows {
                assert!(!r.what.contains("Theme Viewer"), "{}", r.what);
            }
        }
    }

    /// The current-mode mark: Synth marks exactly the two Program Synth groups, Patterns only
    /// Select Patterns.
    #[test]
    fn the_current_mode_marks_its_groups() {
        let marked = |mode| -> Vec<&'static str> {
            groups(false)
                .iter()
                .filter(|g| is_current(g, mode))
                .map(|g| g.title)
                .collect()
        };
        assert_eq!(
            marked(KeyMode::Synth),
            [
                "Program Synth mode, a TB-303 focused",
                "Program Synth mode, a drum machine focused"
            ]
        );
        assert_eq!(marked(KeyMode::Patterns), ["Select Patterns mode"]);
    }
}
