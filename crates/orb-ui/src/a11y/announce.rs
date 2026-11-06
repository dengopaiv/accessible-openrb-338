//! Announcements and the read-out keys (task-189, A11Y-5 of docs/accessibility_plan.md,
//! docs/accessibility.md 3.5, docs/ui_spec.md section 20.3).
//!
//! The announcer is one node of the accessibility tree, a polite live region outside the focus
//! order, whose text is the last message. Each message is a new node (its id carries the
//! message's serial): `accesskit_windows` 0.35.2 raises UI Automation's LiveRegionChanged for an
//! added node that has a name (`adapter.rs:256`), but for a changed node only when its name
//! changed (`adapter.rs:313`), so the same text set again on one node would be silent. A new node
//! speaks every message, a repeated one too, in the frame it is set, with no invisible text.
//!
//! What is announced, in screen reader mode only ([`App::announce`]): the status line's
//! messages ([`App::set_status`]), a song opened, File > New, a mod chosen, the keyboard mode,
//! and what an original key did to a control that does not have the focus ([`key_effect`], from
//! `App::act`). Nothing else: the engine's side of the panel (position, meters, LEDs, running
//! lights) never sets the announcer.

use eframe::egui::{self, Key, Rect, accesskit};

use super::{drive, grid, hit_id_of, names};
use crate::app::App;
use crate::model::Panel;

/// The last message and how many there were.
#[derive(Clone, Debug, Default)]
pub struct Announcer {
    text: String,
    serial: u64,
}

impl Announcer {
    /// A new message: the node of the next frame is a new one with this text.
    pub fn say(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.serial += 1;
    }

    /// The last message (None before the first).
    pub fn text(&self) -> Option<&str> {
        (self.serial > 0).then_some(self.text.as_str())
    }

    /// The number of messages so far.
    pub fn serial(&self) -> u64 {
        self.serial
    }

    /// The egui id of the node of the last message.
    pub fn node_id(&self) -> egui::Id {
        egui::Id::unique(("orb announcement", self.serial))
    }
}

/// The announcer's node, built once per frame at the end of `App::show`: a Label (UI Automation
/// names a Label from its value), live Polite, without bounds and without the focus. None
/// before the first message (an empty live region would raise an empty event).
pub fn show(ctx: &egui::Context, a: &Announcer) {
    let Some(text) = a.text() else {
        return;
    };
    let text = text.to_string();
    ctx.accesskit_node_builder(a.node_id(), |node| {
        node.set_role(accesskit::Role::Label);
        node.set_value(text.clone());
        node.set_label(text);
        node.set_live(accesskit::Live::Polite);
    });
}

/// The egui id of the status line's node.
pub fn status_id() -> egui::Id {
    egui::Id::unique("orb status line")
}

/// The status line's node (role Status, UI Automation's StatusBar, not live: the announcer
/// speaks the message): its text as the name, the bar as the bounds.
pub fn describe_status(ctx: &egui::Context, rect: Rect, text: &str) {
    let text = text.to_string();
    ctx.accesskit_node_builder(status_id(), |node| {
        node.set_role(accesskit::Role::Status);
        node.set_label(text);
        node.set_bounds(accesskit::Rect {
            x0: rect.min.x.into(),
            y0: rect.min.y.into(),
            x1: rect.max.x.into(),
            y1: rect.max.y.into(),
        });
    });
}

/// What an original key can change that an announcement names (`App::act` takes it before and
/// after the key's action).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyState {
    pub playing: bool,
    pub recording: bool,
    pub looping: bool,
    pub song_mode: bool,
    /// The tempo display (BPM).
    pub tempo: i32,
    /// The bar display (None in pattern mode).
    pub bar: Option<i32>,
    /// The selected pattern of each row.
    pub patterns: [u8; 4],
}

impl KeyState {
    pub fn of(panel: &Panel) -> Self {
        let t = &panel.transport;
        Self {
            playing: t.playing,
            recording: t.recording,
            looping: t.looping,
            song_mode: t.song_mode,
            tempo: t.tempo_display(),
            bar: panel.display("transport.bar"),
            patterns: std::array::from_fn(|r| panel.rows[r].pattern),
        }
    }

    /// `self` with only the quantity a held arrow key `arrow` changes taken from `now`: the
    /// tempo for the tempo arrows, the bar for the bar arrows and the wind buttons. What
    /// playback moved meanwhile (the bar, the patterns of song mode) is not the key's doing.
    pub fn with_arrow_from(&self, arrow: &str, now: &KeyState) -> Self {
        let mut s = self.clone();
        if arrow.starts_with("transport.tempo_") {
            s.tempo = now.tempo;
        } else if arrow.starts_with("transport.bar_")
            || matches!(arrow, "transport.rewind" | "transport.ff")
        {
            s.bar = now.bar;
            s.recording = now.recording;
        }
        s
    }
}

/// The spoken name of row `r` (0..=3): "Bass Line 1", ..., "TR-909".
fn row_name(r: usize) -> &'static str {
    names::section_name(&format!("pattern{}", r + 1)).unwrap_or("Row")
}

fn on_off(on: bool) -> &'static str {
    if on { "on" } else { "off" }
}

/// What an original key did, from the state before and after it, without what the focused
/// control `focused` (egui's focus) shows: the screen reader speaks that control's own value
/// change. Play and Stop have no control that shows them and are always said. None when the
/// key changed nothing that is announced (the 303's and the drums' keys: tasks 190 and 191).
pub fn key_effect(
    before: &KeyState,
    after: &KeyState,
    focused: Option<egui::Id>,
) -> Option<String> {
    let unfocused = |kind: &str, id: &str| focused != Some(hit_id_of(kind, id));
    let mut said: Vec<String> = Vec::new();
    if before.playing != after.playing {
        said.push(if after.playing { "playing" } else { "stopped" }.into());
    }
    if before.song_mode != after.song_mode && unfocused("switch", "transport.mode") {
        said.push(
            if after.song_mode {
                "song mode"
            } else {
                "pattern mode"
            }
            .into(),
        );
    }
    if before.recording != after.recording && unfocused("toggle", "transport.record") {
        said.push(format!("recording {}", on_off(after.recording)));
    }
    if before.looping != after.looping && unfocused("toggle", "transport.loop") {
        said.push(format!("loop {}", on_off(after.looping)));
    }
    if before.tempo != after.tempo && unfocused("display", "transport.tempo") {
        let shown = names::display_text("transport.tempo", Some(after.tempo));
        said.push(format!("tempo {shown}"));
    }
    if let (Some(a), Some(b)) = (before.bar, after.bar)
        && a != b
        && unfocused("display", "transport.bar")
    {
        said.push(format!("bar {b}"));
    }
    for r in 0..4 {
        if before.patterns[r] != after.patterns[r] && focused != Some(drive::selector_id(r)) {
            let p = names::pattern_text(after.patterns[r]);
            said.push(format!("{} pattern {p}", row_name(r)));
        }
    }
    if said.is_empty() {
        return None;
    }
    // "Stopped, recording off": the first letter of the message in upper case
    let text = said.join(", ");
    let mut c = text.chars();
    let first = c.next()?;
    Some(first.to_uppercase().chain(c).collect())
}

/// The row (0..=3) of a layout section: the machine's pattern section, the machine itself and
/// its mixer channel (decision A11Y-D3). None for the transport, the master and the effects.
pub fn row_of_section(section: &str) -> Option<usize> {
    let name = names::section_name(section)?;
    (0..4).find(|&r| row_name(r) == name)
}

/// F2: the pattern row of `r`: "Bass Line 1, pattern B3, 16 steps, shuffle off" (", pattern
/// section off" when the row's power switch is off). Tasks 190 and 191 add the synths' and the
/// drums' steps.
pub fn pattern_readout(panel: &Panel, r: usize) -> String {
    let row = &panel.rows[r];
    let p = row.pattern as usize;
    let steps = names::display_text(
        &format!("pattern{}.steps", r + 1),
        Some(i32::from(row.len[p])),
    );
    let mut s = format!(
        "{}, pattern {}, {steps}, shuffle {}",
        row_name(r),
        names::pattern_text(row.pattern),
        on_off(row.shuffle[p])
    );
    if !row.enabled {
        s.push_str(", pattern section off");
    }
    s
}

/// F3: the transport: "Playing, bar 5, 128 BPM, song mode, loop 1 to 8, recording off"; in
/// pattern mode, where the bar, the loop and Record do nothing, "Stopped, 140 BPM, pattern mode".
pub fn transport_readout(panel: &Panel) -> String {
    let t = &panel.transport;
    let mut parts: Vec<String> = vec![if t.playing { "Playing" } else { "Stopped" }.into()];
    if t.song_mode {
        parts.push(format!("bar {}", t.bar_display()));
    }
    parts.push(names::display_text(
        "transport.tempo",
        Some(t.tempo_display()),
    ));
    if t.song_mode {
        parts.push("song mode".into());
        if t.looping {
            let start = t.loop_start_display();
            let end = start + t.loop_length_display() - 1;
            parts.push(if end > start {
                format!("loop {start} to {end}")
            } else {
                format!("loop bar {start}")
            });
        } else {
            parts.push("loop off".into());
        }
        parts.push(format!("recording {}", on_off(t.recording)));
    } else {
        parts.push("pattern mode".into());
    }
    parts.join(", ")
}

/// F4: the levels as the panel shows them: the master meters' lit segments of 10 and a count
/// of the compressor's gain-reduction LEDs lit left of its "0" LED, of 4 (a count of lit LEDs,
/// not an amount; they hold their last state). Numbers, not decibels: the meters' scale was
/// not read in the original ([G], docs/ui_spec.md section 16). A meter the engine has not
/// reported yet (no `Link` sync since the panel was made; the "0" LED is lit by every sync)
/// reads "no reading", not 0.
pub fn levels_readout(panel: &Panel) -> String {
    let m = |id: &str| match panel.meters.get(id) {
        Some(v) => format!("{} of 10", v.min(&10)),
        None => "no reading".into(),
    };
    let gr = if panel.gr_leds == [0; 9] {
        "no reading".to_string()
    } else {
        let n = panel.gr_leds[..4].iter().filter(|&&l| l > 0).count();
        format!("{n} of 4")
    };
    format!(
        "Master left {}, right {}, gain reduction {gr}",
        m("master.meter_l"),
        m("master.meter_r")
    )
}

impl App {
    /// Says `text` through the announcer, in screen reader mode only.
    pub fn announce(&mut self, text: impl Into<String>) {
        if self.screen_reader_mode() {
            self.announcer.say(text);
        }
    }

    /// Puts `text` on the status line and announces it (every status message comes here).
    pub fn set_status(&mut self, text: impl Into<String>) {
        let text = text.into();
        self.announce(text.clone());
        self.status = Some(text);
    }

    /// Announces what an original key did since `before` ([`key_effect`]), egui's focus being
    /// `App::key_focus`. `App::act` calls it, so a key that goes through `act` (TASK-188's F5,
    /// F9, F11 and F12 as `Action::PlayStop` and `Action::Press`) is announced.
    pub(crate) fn announce_key_effect(&mut self, before: &KeyState) {
        let after = KeyState::of(&self.panel);
        if let Some(text) = key_effect(before, &after, self.key_focus) {
            self.announce(text);
        }
    }

    /// The read-out keys of screen reader mode (decision A11Y-D6): F2 the pattern row of the
    /// section egui's focus is in (else of the row with the row focus), F3 the transport, F4
    /// the levels. True when `key` is one of them (it is then the read-out's only).
    pub fn readout_key(&mut self, ui: &egui::Ui, key: Key) -> bool {
        let text = match key {
            Key::F2 => {
                let focused = ui.memory(|m| m.focused());
                let r = focused
                    .and_then(focused_row)
                    .unwrap_or(self.panel.focus.min(3));
                // a drum machine's steps (task-191), a 303's steps (task-190)
                match read_drum_pattern(&self.panel, r) {
                    Some(text) => text,
                    None => {
                        let row = pattern_readout(&self.panel, r);
                        match super::step303::read_303_pattern(&self.panel, r) {
                            Some(steps) => format!("{row}. {steps}"),
                            None => row,
                        }
                    }
                }
            }
            Key::F3 => transport_readout(&self.panel),
            Key::F4 => levels_readout(&self.panel),
            _ => return false,
        };
        self.announce(text);
        true
    }
}

/// F2 on a drum machine's row (2 the 808, 3 the 909; task-191): the pattern row, then each
/// instrument that has steps with its steps ([`grid::drum_readout`]): "TR-808, pattern A1, 16
/// steps, shuffle off. Bass drum: 1, 5, 9, 13. Closed hi-hat: 3, 7, 11, 15". The steps are
/// those of the pattern the grid edits; when playback has moved the row to another pattern
/// (decision task-209), that is said before them: "Grid on pattern A1: Bass drum: 1". None for
/// the 303s.
pub fn read_drum_pattern(panel: &Panel, r: usize) -> Option<String> {
    let is808 = match r {
        2 => true,
        3 => false,
        _ => return None,
    };
    let steps = grid::drum_readout(panel, is808);
    let p = panel.edit_pattern(r);
    let steps = if p == panel.rows[r].pattern {
        steps
    } else {
        format!("Grid on pattern {}: {steps}", names::pattern_text(p))
    };
    Some(format!("{}. {steps}", pattern_readout(panel, r)))
}

/// The row of the section whose control has egui's focus `f`.
fn focused_row(f: egui::Id) -> Option<usize> {
    if let Some(r) = (0..4).find(|&r| drive::selector_id(r) == f) {
        return Some(r);
    }
    if let Some(is808) = grid::machine_of(f) {
        return Some(grid::row_of(is808));
    }
    crate::layout::controls()
        .iter()
        .find(|c| super::hit_id(c) == f)
        .and_then(|c| row_of_section(&c.section))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(panel: &Panel) -> KeyState {
        KeyState::of(panel)
    }

    #[test]
    fn a_repeated_message_is_a_new_node() {
        let mut a = Announcer::default();
        assert_eq!(a.text(), None);
        a.say("Saved x");
        let first = a.node_id();
        a.say("Saved x");
        assert_eq!(a.text(), Some("Saved x"));
        assert_ne!(a.node_id(), first);
        assert_eq!(a.serial(), 2);
    }

    #[test]
    fn key_effects_name_what_changed() {
        let mut p = Panel::new();
        let b = state(&p);
        p.press("transport.mode", 0.0);
        p.press("transport.record", 0.0);
        let a = state(&p);
        assert_eq!(
            key_effect(&b, &a, None).as_deref(),
            Some("Song mode, recording on")
        );
        let b = a;
        p.press("transport.play", 0.0);
        p.press("transport.loop", 0.0);
        p.transport.tempo_step(1);
        let a = state(&p);
        assert_eq!(
            key_effect(&b, &a, None).as_deref(),
            Some("Playing, loop on, tempo 141 BPM")
        );
        let b = a;
        p.press("pattern3.pattern4", 0.0);
        p.press("transport.stop", 0.0);
        let a = state(&p);
        assert_eq!(
            key_effect(&b, &a, None).as_deref(),
            Some("Stopped, recording off, TR-808 pattern A4")
        );
        assert_eq!(key_effect(&a, &a, None), None);
    }

    #[test]
    fn the_focused_control_is_not_announced() {
        let mut p = Panel::new();
        let b = state(&p);
        p.transport.tempo_step(-1);
        p.press("pattern1.pattern2", 0.0);
        let a = state(&p);
        let tempo = Some(hit_id_of("display", "transport.tempo"));
        assert_eq!(
            key_effect(&b, &a, tempo).as_deref(),
            Some("Bass Line 1 pattern A2")
        );
        let sel = Some(drive::selector_id(0));
        assert_eq!(key_effect(&b, &a, sel).as_deref(), Some("Tempo 139 BPM"));
        // Play has no control that shows it
        let b = state(&p);
        p.press("transport.play", 0.0);
        let play = Some(hit_id_of("trigger", "transport.play"));
        assert_eq!(key_effect(&b, &state(&p), play).as_deref(), Some("Playing"));
    }

    #[test]
    fn a_held_arrow_takes_only_its_own_quantity() {
        let mut p = Panel::new();
        p.press("transport.mode", 0.0);
        let seen = state(&p);
        p.transport.tempo_step(5);
        p.transport.position += 4 * 768;
        p.rows[2].pattern = 9;
        let now = state(&p);
        let after = seen.with_arrow_from("transport.tempo_up", &now);
        assert_eq!(
            key_effect(&seen, &after, None).as_deref(),
            Some("Tempo 145 BPM")
        );
        let after = seen.with_arrow_from("transport.ff", &now);
        assert_eq!(key_effect(&seen, &after, None).as_deref(), Some("Bar 5"));
    }

    #[test]
    fn readouts() {
        let mut p = Panel::new();
        assert_eq!(
            pattern_readout(&p, 0),
            "Bass Line 1, pattern A1, 16 steps, shuffle off"
        );
        p.rows[3].pattern = 10;
        p.rows[3].len[10] = 1;
        p.rows[3].shuffle[10] = true;
        p.rows[3].enabled = false;
        assert_eq!(
            pattern_readout(&p, 3),
            "TR-909, pattern B3, 1 step, shuffle on, pattern section off"
        );
        assert_eq!(transport_readout(&p), "Stopped, 140 BPM, pattern mode");
        p.press("transport.mode", 0.0);
        p.press("transport.play", 0.0);
        p.transport.position = 4 * 768;
        assert_eq!(
            transport_readout(&p),
            "Playing, bar 5, 140 BPM, song mode, loop off, recording off"
        );
        p.press("transport.loop", 0.0);
        p.press("transport.record", 0.0);
        assert_eq!(
            transport_readout(&p),
            "Playing, bar 5, 140 BPM, song mode, loop 1 to 4, recording on"
        );
        assert_eq!(
            levels_readout(&p),
            "Master left no reading, right no reading, gain reduction no reading"
        );
        p.meters.insert("master.meter_l".into(), 7);
        p.meters.insert("master.meter_r".into(), 6);
        p.gr_leds = [0, 0, 2, 2, 1, 0, 0, 0, 0];
        assert_eq!(
            levels_readout(&p),
            "Master left 7 of 10, right 6 of 10, gain reduction 2 of 4"
        );
    }

    #[test]
    fn rows_of_sections() {
        assert_eq!(row_of_section("tb303_2"), Some(1));
        assert_eq!(row_of_section("mixer3"), Some(2));
        assert_eq!(row_of_section("pattern4"), Some(3));
        assert_eq!(row_of_section("transport"), None);
        assert_eq!(row_of_section("delay"), None);
    }
}
