//! Help > Keyboard Shortcuts (task-105): a window that lists every key orb-ui handles, by group,
//! with the current keyboard mode marked.
//!
//! The list is one table ([`groups`]). Each row carries, besides its text, the [`Bind`]s it
//! stands for: a key, the modes and focus rows it is active in and the [`keys::Action`] it
//! must give. The tests run every bind through [`keys::map`] and run every key `keys::map`
//! gives an action through the binds, so the window cannot drift from the bindings: a changed
//! action fails the first, a new or removed binding the second. The menu shortcuts come from
//! [`crate::shortcuts`], the one definition of them.

use eframe::egui::{self, Color32, Context, Key, KeyboardShortcut, Modifiers, RichText};

use crate::a11y::window as a11y;
use crate::app::App;
use crate::keys::{Action, KeyMode, SrAction};
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

/// What a bind of screen reader mode must give from [`keys::map_screen_reader`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SrWant {
    Panel(Want),
    Control(bool),
    Section(bool),
    Activate,
    Help,
    /// No action from the key map (a value key, handled on a focused value control).
    Nothing,
    /// A read-out key (F2 to F4, task-189): handled by `App::readout_key` before the key map,
    /// which has no action for it.
    Readout,
}

impl SrWant {
    pub fn action(&self, focus: usize) -> Option<SrAction> {
        Some(match self {
            SrWant::Panel(w) => SrAction::Panel(w.action(focus)?),
            SrWant::Control(back) => SrAction::Control(*back),
            SrWant::Section(back) => SrAction::Section(*back),
            SrWant::Activate => SrAction::Activate,
            SrWant::Help => SrAction::Help,
            SrWant::Nothing | SrWant::Readout => return None,
        })
    }
}

/// One key of screen reader mode, with Ctrl held or not, and what
/// [`keys::map_screen_reader`] must make of it.
#[derive(Clone, Debug)]
pub struct SrBind {
    pub key: Key,
    pub shift: Shift,
    pub ctrl: bool,
    pub scope: Scope,
    pub want: SrWant,
}

fn sr(key: Key, shift: Shift, want: SrWant) -> SrBind {
    SrBind {
        key,
        shift,
        ctrl: false,
        scope: Scope::Always,
        want,
    }
}

fn sr_ctrl(key: Key, shift: Shift, scope: Scope, want: SrWant) -> SrBind {
    SrBind {
        key,
        shift,
        ctrl: true,
        scope,
        want,
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
    /// The bindings of screen reader mode the line stands for (its group's only).
    pub sr_binds: Vec<SrBind>,
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
        sr_binds: vec![],
    }
}

fn sr_row(keys: &str, what: &'static str, sr_binds: Vec<SrBind>) -> Row {
    Row {
        keys: keys.into(),
        what,
        binds: vec![],
        sr_binds,
    }
}

/// The keys of screen reader mode (task-188, decisions A11Y-D2 and A11Y-D6), listed first when
/// the mode is on, with the read-out keys F2 to F4 (task-189).
fn screen_reader_group(is_mac: bool) -> Group {
    use Shift::{Any, No, Yes};
    let press = |s: &str| SrWant::Panel(id(s));
    let s303 = Scope::Synth303;
    let value_keys = [
        Key::ArrowUp,
        Key::ArrowDown,
        Key::PageUp,
        Key::PageDown,
        Key::Home,
        Key::End,
    ];
    Group {
        title: "Screen reader mode",
        mode: None,
        note: "On while Options > Screen Reader Mode says so. These keys replace the ones \
               below where they differ; every other key keeps its meaning, whatever control \
               of the panel has the focus. While a menu or a window (this one, Info, About, \
               Mods > Info) has the focus, or a window is open and nothing has it, the \
               panel's keys wait, but for F1 to F6, F9, F11 and F12. F2, F3 and F4 are \
               spoken, without moving the focus.",
        rows: vec![
            sr_row("F1", "This window", vec![sr(Key::F1, Any, SrWant::Help)]),
            sr_row(
                "F2",
                "Read the pattern of the focused section (else of the row with the row focus)",
                vec![sr(Key::F2, No, SrWant::Readout)],
            ),
            sr_row(
                "F3",
                "Read the transport",
                vec![sr(Key::F3, No, SrWant::Readout)],
            ),
            sr_row(
                "F4",
                "Read the levels",
                vec![sr(Key::F4, No, SrWant::Readout)],
            ),
            sr_row(
                "F5",
                "Play if stopped, Stop otherwise",
                vec![sr(Key::F5, Any, SrWant::Panel(Want::PlayStop))],
            ),
            sr_row(
                "F6 / Shift+F6",
                "The first control of the next / previous section (wraps). The sections: \
                 Transport, Bass Line 1, Bass Line 2, TR-808, TR-909 (each with its pattern \
                 row and mixer channel), Master and compressor, PCF, Delay, Distortion",
                vec![
                    sr(Key::F6, No, SrWant::Section(false)),
                    sr(Key::F6, Yes, SrWant::Section(true)),
                ],
            ),
            sr_row(
                "F9",
                "Record (Insert too)",
                vec![sr(Key::F9, Any, press("transport.record"))],
            ),
            sr_row("F10  or  Alt alone", "The menu bar; again, back", vec![]),
            sr_row(
                "C F V G B N J M K , L . /",
                "A 303's pitch keys set the step's note only (a rest stays a rest, as in the \
                 original); with Options > Pitch Keys Make Notes on, they also turn a rest \
                 into a note. - and = turn a step into a rest and back either way",
                vec![],
            ),
            sr_row(
                "F11",
                "Loop on / off",
                vec![sr(Key::F11, Any, press("transport.loop"))],
            ),
            sr_row(
                "F12",
                "Song / pattern mode",
                vec![sr(Key::F12, Any, press("transport.mode"))],
            ),
            sr_row(
                &shortcuts::text_for(&shortcuts::CHOOSE_MOD, is_mac),
                "Mods > Choose Mod...: a dialog with a filter field and the list of the \
                 Mods menu's mods, as a tree",
                vec![],
            ),
            sr_row(
                "In Choose Mod",
                "Tab / Shift+Tab: between the filter and the list. Typing in the filter \
                 narrows the list by name, ASCII case ignored. Down in the filter: to the \
                 list. In the list: Up / Down, Home / End, PgUp / PgDn move the selection; \
                 Right opens a mod with variants, Left closes it or goes from a variant to \
                 its mod. Enter (in the filter too) chooses: the mod, for a mod with \
                 variants the preferred one, or the variant selected. Escape closes",
                vec![],
            ),
            sr_row(
                &format!(
                    "{} in Choose Mod",
                    shortcuts::text_for(&shortcuts::CHOOSER_INFO, is_mac)
                ),
                "Mod Info of the selected mod, without loading it",
                vec![],
            ),
            sr_row(
                "Tab / Shift+Tab",
                "The next / previous control of the panel, section by section (wraps). \
                 Entering a machine's section gives that machine the row focus",
                vec![
                    sr(Key::Tab, No, SrWant::Control(false)),
                    sr(Key::Tab, Yes, SrWant::Control(true)),
                ],
            ),
            sr_row(
                "Space",
                "Press or toggle the focused button, toggle or switch (not Play / Stop). On a \
                 drum grid: set the step as a click does; on the TR-909 with Ctrl, Alt or \
                 Ctrl+Alt as the mouse's (a note, an accent, a flam); Shift+Space is an \
                 accent too, without Windows' system menu that Alt+Space also opens",
                vec![sr(Key::Space, Any, SrWant::Activate)],
            ),
            sr_row(
                "Enter",
                "By the section of the focused control: in a 303's section that 303's Step \
                 (Program Synth mode), in any other section Play. The pitch and drum keys \
                 still go to the row focus, and Backspace is a 303's Back",
                vec![],
            ),
            sr_row(
                "Escape",
                "Close the open window, dialog (Choose Mod too) or menu",
                vec![],
            ),
            sr_row(
                "Up / Down",
                "On a knob or fader: to the next value that is spoken differently (one position \
                 with Options > Speak Raw Values on). On a spin button: one click of its \
                 arrow. On a pattern selector: the next / previous pattern. On a 303's step \
                 editor: the previous / next step (its Back / Step, wrapping at 16); Home / \
                 End step 1 / the pattern's last step. The step a programming key changed is \
                 spoken; F2 reads the whole pattern. On a drum grid: the instrument before / \
                 after",
                value_keys[..2]
                    .iter()
                    .map(|&k| sr(k, Any, SrWant::Nothing))
                    .collect(),
            ),
            sr_row(
                "PgUp / PgDn",
                "On a knob or fader: by 8 or more, to a value spoken differently, nothing when none \
                 lies that way (by 8 with Speak Raw Values on). On a spin button: 10 clicks. On a pattern \
                 selector: the next / previous bank",
                value_keys[2..4]
                    .iter()
                    .map(|&k| sr(k, Any, SrWant::Nothing))
                    .collect(),
            ),
            sr_row(
                "Home / End",
                "On a knob or fader: 0 / 127. On a pattern selector: A1 / D8. On the pattern \
                 length, delay steps and PCF pattern: the end of the range; nothing on \
                 tempo, bar, loop start and loop length. On a drum grid: the first / last \
                 step",
                value_keys[4..]
                    .iter()
                    .map(|&k| sr(k, Any, SrWant::Nothing))
                    .collect(),
            ),
            sr_row(
                "Left / Right",
                "On a drum grid: the step before / after, within the pattern's length. \
                 Nothing on a 303's step editor (Up / Down move its step)",
                [Key::ArrowLeft, Key::ArrowRight]
                    .iter()
                    .map(|&k| sr(k, Any, SrWant::Nothing))
                    .collect(),
            ),
            sr_row(
                "Ctrl+Up / Ctrl+Down",
                "Move the row focus to the previous / next row (wraps), without moving the \
                 focus",
                vec![
                    sr_ctrl(
                        Key::ArrowUp,
                        No,
                        Scope::Always,
                        SrWant::Panel(Want::Focus(-1)),
                    ),
                    sr_ctrl(
                        Key::ArrowDown,
                        No,
                        Scope::Always,
                        SrWant::Panel(Want::Focus(1)),
                    ),
                ],
            ),
            sr_row(
                "Ctrl+Tab / Ctrl+Shift+Tab",
                "Program Synth mode, a 303 focused: the playing step becomes a note / a rest \
                 (the original's Tab / Shift+Tab)",
                vec![
                    sr_ctrl(Key::Tab, No, s303, SrWant::Panel(Want::Tap303(false))),
                    sr_ctrl(Key::Tab, Yes, s303, SrWant::Panel(Want::Tap303(true))),
                    sr_ctrl(Key::Tab, Any, Scope::SynthDrums, SrWant::Nothing),
                    sr_ctrl(Key::Tab, Any, Scope::Patterns, SrWant::Nothing),
                ],
            ),
        ],
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

/// All the groups of the window, for a platform (`is_mac`: Cmd instead of Ctrl); with
/// `screen_reader` (the mode is on) the keys of screen reader mode come after the menu
/// shortcuts.
pub fn groups(is_mac: bool, screen_reader: bool) -> Vec<Group> {
    let mut g = original_groups(is_mac);
    if screen_reader {
        g.insert(1, screen_reader_group(is_mac));
    }
    g
}

fn original_groups(is_mac: bool) -> Vec<Group> {
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
        let due = self.window_focus_due(crate::app::Win::KeyHelp);
        let mut open = true;
        let screen = ctx.content_rect();
        let size = egui::vec2(
            (screen.width() - 40.0).clamp(200.0, 520.0),
            (screen.height() - 40.0).clamp(150.0, 440.0),
        );
        let current = self.key_mode;
        let screen_reader = self.screen_reader_mode();
        // centered the way File > Info is (info.rs): pivot, default place, an id per open
        let id = self.key_help_id();
        egui::Window::new("Keyboard Shortcuts")
            .id(id)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(screen.center())
            .default_size(size)
            .show(ctx, |ui| {
                crate::app::focus_into_window(ui, due);
                a11y::window(ui, screen_reader, id, "Keyboard Shortcuts");
                // in screen reader mode every line takes the focus, the first when the window
                // opens (the window has no control of its own but the close button)
                a11y::line(
                    ui,
                    screen_reader,
                    RichText::new(format!("Keyboard mode now: {}", mode_name(current)))
                        .strong()
                        .color(CURRENT),
                    false,
                    None,
                );
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let all = groups(ctx.os().is_mac(), screen_reader);
                        for (n, g) in all.iter().enumerate() {
                            ui.add_space(8.0);
                            let is_current = is_current(g, current);
                            let mut title = RichText::new(g.title).strong().size(15.0);
                            title = title.color(if is_current { CURRENT } else { INK });
                            ui.horizontal(|ui| {
                                a11y::line(ui, screen_reader, title, false, None);
                                if is_current {
                                    ui.label(RichText::new("(current mode)").color(CURRENT));
                                }
                            });
                            if !g.note.is_empty() {
                                let note = RichText::new(g.note).color(INK_DIM);
                                a11y::line(ui, screen_reader, note, false, None);
                            }
                            egui::Grid::new(("orb_key_help_grid", n))
                                .num_columns(2)
                                .show(ui, |ui| {
                                    for r in &g.rows {
                                        ui.label(RichText::new(&r.keys).monospace().color(INK));
                                        let spoken = format!("{}: {}", r.keys, r.what);
                                        let what = RichText::new(r.what);
                                        a11y::line(ui, screen_reader, what, true, Some(&spoken));
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
        groups(is_mac, true)
            .into_iter()
            .flat_map(|g| g.rows)
            .flat_map(|r| r.binds)
            .collect()
    }

    fn sr_binds() -> Vec<SrBind> {
        groups(false, true)
            .into_iter()
            .flat_map(|g| g.rows)
            .flat_map(|r| r.sr_binds)
            .collect()
    }

    /// The screen reader group is listed when the mode is on only, after the menu shortcuts,
    /// and carries the only screen reader binds.
    #[test]
    fn the_screen_reader_keys_are_listed_when_the_mode_is_on() {
        let titles =
            |on| -> Vec<&'static str> { groups(false, on).iter().map(|g| g.title).collect() };
        assert!(!titles(false).contains(&"Screen reader mode"));
        assert_eq!(titles(true)[1], "Screen reader mode");
        assert_eq!(titles(true).len(), titles(false).len() + 1);
        for g in groups(false, true) {
            let sr = g.rows.iter().any(|r| !r.sr_binds.is_empty());
            assert_eq!(sr, g.title == "Screen reader mode", "{}", g.title);
        }
    }

    /// Every listed key of screen reader mode does what the window says.
    #[test]
    fn every_listed_screen_reader_key_maps_to_its_action() {
        let all = sr_binds();
        assert!(all.len() > 20, "{} binds", all.len());
        for b in &all {
            for &shift in b.shift.states() {
                for (mode, focus) in b.scope.cases() {
                    assert_eq!(
                        keys::map_screen_reader(b.key, shift, b.ctrl, mode, focus),
                        b.want.action(focus),
                        "{:?} shift={shift} ctrl={} {mode:?} focus={focus}",
                        b.key,
                        b.ctrl
                    );
                }
            }
        }
    }

    /// Every key whose meaning screen reader mode changes is listed in its group: with Ctrl,
    /// every key it maps; without, every key it maps otherwise than `keys::map`.
    #[test]
    fn every_key_the_mode_changes_is_listed() {
        let all = sr_binds();
        let mut changed = 0;
        for &key in Key::ALL {
            for ctrl in [false, true] {
                for shift in [false, true] {
                    for mode in [KeyMode::Synth, KeyMode::Patterns] {
                        for focus in 0..4 {
                            let got = keys::map_screen_reader(key, shift, ctrl, mode, focus);
                            let plain = if ctrl {
                                None
                            } else {
                                keys::map(key, shift, mode, focus).map(SrAction::Panel)
                            };
                            if got == plain {
                                continue;
                            }
                            changed += 1;
                            let listed = all.iter().any(|b| {
                                b.key == key
                                    && b.ctrl == ctrl
                                    && b.shift.states().contains(&shift)
                                    && b.scope.cases().contains(&(mode, focus))
                                    && b.want.action(focus) == got
                            });
                            assert!(
                                listed,
                                "{key:?} shift={shift} ctrl={ctrl} {mode:?} focus={focus} -> \
                                 {got:?} is not listed in the screen reader group"
                            );
                        }
                    }
                }
            }
        }
        assert!(changed > 100, "{changed} changed cases");
    }

    /// The keys of the mode that `keys::map_screen_reader` does not map (the menu shortcuts,
    /// the keys of the step editor, the grid, the mod chooser and the dialogs, handled in
    /// `App::keyboard` and the windows) cannot be compared with the key map. This is a check
    /// by name only: it does not read the code, so a key added in `app.rs` is not found by
    /// it; its name must be added to these lists by hand. The first list is checked against
    /// the rows' `keys` field, the second against the rows' descriptions.
    #[test]
    fn the_keys_handled_outside_the_key_map_are_named_in_the_group() {
        let g = screen_reader_group(false);
        for k in [
            "F10",
            "Ctrl+M",
            "Escape",
            "Enter",
            "Space",
            "Left / Right",
            "Ctrl+Up / Ctrl+Down",
            "Ctrl+Tab",
            "PgUp / PgDn",
            "Home / End",
            "F6 / Shift+F6",
        ] {
            assert!(
                g.rows.iter().any(|r| r.keys.contains(k)),
                "no row of the screen reader group has the keys {k:?}"
            );
        }
        assert!(
            g.rows
                .iter()
                .any(|r| r.keys.contains("Ctrl+I in Choose Mod"))
        );
        for k in [
            "Shift+Space",
            "Ctrl+Alt",
            "Backspace",
            "step editor",
            "drum grid",
        ] {
            assert!(
                g.rows.iter().any(|r| r.what.contains(k)),
                "no row of the screen reader group describes {k:?}"
            );
        }
        let mac = screen_reader_group(true);
        assert!(mac.rows.iter().any(|r| r.keys.contains("Cmd+M")));
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
            groups(is_mac, true)[0]
                .rows
                .iter()
                .map(|r| r.keys.clone())
                .collect()
        };
        assert_eq!(text(true), ["Cmd+N", "Cmd+O", "Cmd+S", "Cmd+F", "Cmd+G"]);
        assert_eq!(
            text(false),
            ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+F", "Ctrl+G"]
        );
    }

    /// The current-mode mark: Synth marks exactly the two Program Synth groups, Patterns only
    /// Select Patterns.
    #[test]
    fn the_current_mode_marks_its_groups() {
        let marked = |mode| -> Vec<&'static str> {
            groups(false, true)
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
