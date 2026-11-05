//! What a screen reader says for each part of the panel (task-186, docs/accessibility.md 3.3):
//! the spoken name of every layout id of `assets/ui/layout.tsv`, the spoken section it belongs
//! to, the accessibility role of each kind and the value text of a control.
//!
//! The names follow the original's manual as docs/ui_spec.md sections 3 to 9 give them, spelled
//! out for speech ("Bass drum level", not "BD level"; "C sharp", not "C#"). A machine's pattern
//! section and its mixer channel belong to the machine's spoken section (decision A11Y-D3,
//! task-179). Knobs and faders read the 0..127 value the original stores until task-192 gives
//! them their units (decision A11Y-D5, task-181): that is [`control_value_text`], the one
//! function to extend.

use eframe::egui::accesskit::Role;

use crate::layout::Ctl;
use crate::model::Panel;

/// The spoken section of a layout section; None for a section the layout does not have.
pub fn section_name(section: &str) -> Option<&'static str> {
    Some(match section {
        "transport" => "Transport",
        "pattern1" | "tb303_1" | "mixer1" => "Bass Line 1",
        "pattern2" | "tb303_2" | "mixer2" => "Bass Line 2",
        "pattern3" | "tr808" | "mixer3" => "TR-808",
        "pattern4" | "tr909" | "mixer4" => "TR-909",
        "master" | "comp" => "Master and compressor",
        "pcf" => "PCF",
        "delay" => "Delay",
        "dist" => "Distortion",
        _ => return None,
    })
}

/// The accessibility role of a kind of the layout; None for the parts that are not controls
/// and get no node (LEDs, meters, the step parts, the click areas, the mod picture).
pub fn role(kind: &str) -> Option<Role> {
    Some(match kind {
        "knob" | "fader" => Role::Slider,
        "toggle" => Role::CheckBox,
        // a pattern or bank button lights when it is the selected one
        "radio" | "grid" => Role::RadioButton,
        "trigger" | "repeat" | "wind" => Role::Button,
        // a two-position switch: its value is the word of the position
        "switch" | "hswitch" | "vswitch" => Role::Switch,
        "display" => Role::Label,
        _ => return None,
    })
}

/// The kinds a screen reader can act on: every kind but the LEDs, meters, step parts, click
/// areas, mod picture and displays.
pub fn is_interactive(kind: &str) -> bool {
    !matches!(
        kind,
        "led" | "meter" | "steps" | "area" | "modview" | "display"
    )
}

/// The names of the parts, by `family.name` (the family is the section without its number:
/// `pattern`, `tb303`, `mixer`); `@kind` follows when two kinds share an id (the 303's accent
/// knob and accent button). `{n}` is the section's number. The numbered parts (pattern and bank
/// buttons, the 303's keys and their lights, the drum steps, the gain reduction lights) are
/// named by [`numbered`].
const NAMES: &[(&str, &str)] = &[
    // transport
    ("transport.shuffle", "Shuffle amount"),
    ("transport.tempo", "Tempo"),
    ("transport.tempo_up", "Tempo up"),
    ("transport.tempo_down", "Tempo down"),
    ("transport.mode", "Mode"),
    ("transport.play", "Play"),
    ("transport.stop", "Stop"),
    ("transport.rewind", "Rewind"),
    ("transport.ff", "Fast forward"),
    ("transport.record", "Record"),
    ("transport.bar", "Bar"),
    ("transport.bar_up", "Bar up"),
    ("transport.bar_down", "Bar down"),
    ("transport.loop_start", "Loop start"),
    ("transport.loop_start_up", "Loop start up"),
    ("transport.loop_start_down", "Loop start down"),
    ("transport.loop_length", "Loop length"),
    ("transport.loop_length_up", "Loop length up"),
    ("transport.loop_length_down", "Loop length down"),
    ("transport.loop", "Loop"),
    // a machine's pattern section
    ("pattern.on", "Pattern section on"),
    ("pattern.shuffle", "Shuffle"),
    ("pattern.steps", "Pattern length"),
    ("pattern.steps_up", "Pattern length up"),
    ("pattern.steps_down", "Pattern length down"),
    // TB-303
    ("tb303.wave", "Waveform"),
    ("tb303.tune", "Tune"),
    ("tb303.cutoff", "Cutoff"),
    ("tb303.reso", "Resonance"),
    ("tb303.envmod", "Envelope mod"),
    ("tb303.decay", "Decay"),
    ("tb303.accent@knob", "Accent"),
    ("tb303.edit_step", "Edit step"),
    ("tb303.step", "Step forward"),
    ("tb303.back", "Step back"),
    ("tb303.pitch_button", "Pitch mode"),
    ("tb303.clear", "Clear pattern"),
    ("tb303.down", "Octave down"),
    ("tb303.up", "Octave up"),
    ("tb303.accent@trigger", "Step accent"),
    ("tb303.slide", "Slide"),
    ("tb303.note_pause", "Note or pause"),
    ("tb303.pitch_led", "Pitch mode light"),
    ("tb303.down_led", "Octave down light"),
    ("tb303.up_led", "Octave up light"),
    ("tb303.accent_led", "Step accent light"),
    ("tb303.slide_led", "Slide light"),
    ("tb303.note_led", "Note light"),
    // TR-808
    ("tr808.ac_level", "Accent level"),
    ("tr808.bd_level", "Bass drum level"),
    ("tr808.bd_tone", "Bass drum tone"),
    ("tr808.bd_decay", "Bass drum decay"),
    ("tr808.sd_level", "Snare drum level"),
    ("tr808.sd_tone", "Snare drum tone"),
    ("tr808.sd_snappy", "Snare drum snappy"),
    ("tr808.lt_level", "Low tom or conga level"),
    ("tr808.lt_tuning", "Low tom or conga tuning"),
    ("tr808.mt_level", "Mid tom or conga level"),
    ("tr808.mt_tuning", "Mid tom or conga tuning"),
    ("tr808.ht_level", "High tom or conga level"),
    ("tr808.ht_tuning", "High tom or conga tuning"),
    ("tr808.rs_level", "Rim shot or claves level"),
    ("tr808.cp_level", "Hand clap or maracas level"),
    ("tr808.cb_level", "Cowbell level"),
    ("tr808.cy_level", "Cymbal level"),
    ("tr808.cy_tone", "Cymbal tone"),
    ("tr808.cy_decay", "Cymbal decay"),
    ("tr808.oh_level", "Open hi-hat level"),
    ("tr808.oh_decay", "Open hi-hat decay"),
    ("tr808.ch_level", "Closed hi-hat level"),
    ("tr808.slot3_switch", "Low tom or conga switch"),
    ("tr808.slot4_switch", "Mid tom or conga switch"),
    ("tr808.slot5_switch", "High tom or conga switch"),
    ("tr808.slot6_switch", "Rim shot or claves switch"),
    ("tr808.slot7_switch", "Hand clap or maracas switch"),
    ("tr808.mod_view", "Mod picture"),
    ("tr808.select_bd", "Instrument selection"),
    // TR-909
    ("tr909.ac_level", "Accent level"),
    ("tr909.bd_tune", "Bass drum tune"),
    ("tr909.bd_attack", "Bass drum attack"),
    ("tr909.bd_level", "Bass drum level"),
    ("tr909.bd_decay", "Bass drum decay"),
    ("tr909.sd_tune", "Snare drum tune"),
    ("tr909.sd_tone", "Snare drum tone"),
    ("tr909.sd_level", "Snare drum level"),
    ("tr909.sd_snappy", "Snare drum snappy"),
    ("tr909.lt_tune", "Low tom tune"),
    ("tr909.lt_decay", "Low tom decay"),
    ("tr909.lt_level", "Low tom level"),
    ("tr909.mt_tune", "Mid tom tune"),
    ("tr909.mt_decay", "Mid tom decay"),
    ("tr909.mt_level", "Mid tom level"),
    ("tr909.ht_tune", "High tom tune"),
    ("tr909.ht_decay", "High tom decay"),
    ("tr909.ht_level", "High tom level"),
    ("tr909.rs_level", "Rim shot level"),
    ("tr909.cp_level", "Hand clap level"),
    ("tr909.ch_decay", "Closed hi-hat decay"),
    ("tr909.hh_level", "Hi-hat level"),
    ("tr909.oh_decay", "Open hi-hat decay"),
    ("tr909.cc_level", "Crash cymbal level"),
    ("tr909.cc_tune", "Crash cymbal tune"),
    ("tr909.rc_level", "Ride cymbal level"),
    ("tr909.rc_tune", "Ride cymbal tune"),
    ("tr909.flam", "Flam"),
    ("tr909.flam_button", "Flam entry"),
    ("tr909.select_bd", "Instrument selection"),
    // a machine's mixer channel; the toggle is lit while the channel plays
    ("mixer.mute", "Channel {n} on"),
    ("mixer.meter", "Channel {n} meter"),
    ("mixer.pan", "Channel {n} pan"),
    ("mixer.delay", "Channel {n} delay send"),
    ("mixer.level", "Channel {n} level"),
    ("mixer.dist", "Channel {n} distortion"),
    ("mixer.pcf", "Channel {n} PCF"),
    ("mixer.comp", "Channel {n} compressor"),
    // master and effects
    ("master.meter_l", "Master left meter"),
    ("master.meter_r", "Master right meter"),
    ("master.fader", "Master level"),
    ("master.comp", "Master compressor"),
    ("pcf.on", "PCF on"),
    ("pcf.meter", "PCF input meter"),
    ("pcf.pattern", "PCF pattern"),
    ("pcf.pattern_up", "PCF pattern up"),
    ("pcf.pattern_down", "PCF pattern down"),
    ("pcf.mode", "PCF mode"),
    ("pcf.freq", "Frequency"),
    ("pcf.q", "Q"),
    ("pcf.amt", "Amount"),
    ("pcf.dec", "Decay"),
    ("delay.on", "Delay on"),
    ("delay.meter", "Delay input meter"),
    ("delay.steps", "Delay steps"),
    ("delay.steps_up", "Delay steps up"),
    ("delay.steps_down", "Delay steps down"),
    ("delay.triplet", "Step length"),
    ("delay.pan", "Delay pan"),
    ("delay.feedback", "Feedback"),
    ("dist.on", "Distortion on"),
    ("dist.meter", "Distortion input meter"),
    ("dist.amount", "Amount"),
    ("dist.shape", "Shape"),
    ("comp.on", "Compressor on"),
    ("comp.meter", "Compressor input meter"),
    ("comp.ratio", "Ratio"),
    ("comp.threshold", "Threshold"),
];

/// The 303's key names by note index (0 = low C .. 12 = high C).
const NOTES: [&str; 13] = [
    "C", "C sharp", "D", "D sharp", "E", "F", "F sharp", "G", "G sharp", "A", "A sharp", "B",
    "high C",
];

/// The section's family (without its number) and its number: `pattern2` -> (`pattern`, 2),
/// `tb303_1` -> (`tb303`, 1), `tr808` -> (`tr808`, None).
fn family(section: &str) -> (&str, Option<u32>) {
    for prefix in ["pattern", "mixer", "tb303_"] {
        if let Some(n) = section.strip_prefix(prefix).and_then(|s| s.parse().ok()) {
            return (prefix.trim_end_matches('_'), Some(n));
        }
    }
    (section, None)
}

/// The names of the numbered parts.
fn numbered(family: &str, local: &str) -> Option<String> {
    let num = |prefix: &str| local.strip_prefix(prefix)?.parse::<usize>().ok();
    match family {
        "pattern" => {
            if let Some(k) = num("pattern") {
                return Some(format!("Pattern {k}"));
            }
            let bank = local.strip_prefix("bank")?;
            Some(format!("Bank {bank}"))
        }
        "tb303" => {
            if let Some(k) = local
                .strip_suffix("_led")
                .and_then(|s| s.strip_prefix("key"))
            {
                return Some(format!("{} light", NOTES.get(k.parse::<usize>().ok()?)?));
            }
            Some(format!("Key {}", NOTES.get(num("key")?)?))
        }
        "tr808" | "tr909" => Some(format!("Step {}", num("step")?)),
        "comp" => Some(format!("Gain reduction light {}", num("gr")?)),
        _ => None,
    }
}

/// The spoken name of a layout part; None for an id this module does not know (the test of
/// `layout.tsv` keeps that from happening).
pub fn name(c: &Ctl) -> Option<String> {
    let (family, n) = family(&c.section);
    let local = c.name();
    if let Some(s) = numbered(family, local) {
        return Some(s);
    }
    let key = format!("{family}.{local}");
    let with_kind = format!("{key}@{}", c.kind);
    let s = NAMES
        .iter()
        .find(|(k, _)| *k == with_kind)
        .or_else(|| NAMES.iter().find(|(k, _)| *k == key))?
        .1;
    Some(match n {
        Some(n) => s.replace("{n}", &n.to_string()),
        None => s.to_string(),
    })
}

/// The keys of the name table, for the test that checks them against `layout.tsv`.
pub fn table_keys() -> impl Iterator<Item = &'static str> {
    NAMES.iter().map(|(k, _)| *k)
}

/// The value text of a knob or fader whose stored value is `v` (0..127). The 0..127 number,
/// and pan in words ("center", "left 20", "right 20"; the knob's middle sends 64). Task-192
/// (A11Y-14) extends this function with the physical units derived from the engine.
pub fn control_value_text(id: &str, v: u8) -> String {
    if id.ends_with(".pan") {
        let d = i32::from(v) - 64;
        return match d {
            0 => "center".to_string(),
            d if d < 0 => format!("left {}", -d),
            d => format!("right {d}"),
        };
    }
    v.to_string()
}

/// The words of a two-position switch's positions 0 and 1 (the position it draws,
/// `Panel::switch_position`): docs/ui_spec.md sections 3, 5, 6 and 9.
pub fn switch_words(id: &str) -> [&'static str; 2] {
    let local = id.split_once('.').map_or(id, |(_, n)| n);
    match local {
        "mode" if id.starts_with("transport") => ["pattern", "song"],
        "mode" => ["band pass", "low pass"],
        "wave" => ["sawtooth", "square"],
        "triplet" => ["triplets", "sixteenths"],
        "slot3_switch" => ["low conga", "low tom"],
        "slot4_switch" => ["mid conga", "mid tom"],
        "slot5_switch" => ["high conga", "high tom"],
        "slot6_switch" => ["claves", "rim shot"],
        "slot7_switch" => ["maracas", "hand clap"],
        _ => ["off", "on"],
    }
}

/// What a display shows, with its unit where the display's name does not say it (the node
/// reads name and text: "Tempo 140 BPM", "Bar 3", "Pattern length 16 steps"); "off" for a
/// display the mode turns off (the transport's bar and loop displays in pattern mode).
pub fn display_text(id: &str, shown: Option<i32>) -> String {
    let Some(v) = shown else {
        return "off".to_string();
    };
    let plural = |one: &str, many: &str| format!("{v} {}", if v == 1 { one } else { many });
    match id {
        "transport.tempo" => format!("{v} BPM"),
        "transport.loop_length" => plural("bar", "bars"),
        _ if id.starts_with("pattern") && id.ends_with(".steps") => plural("step", "steps"),
        _ => v.to_string(),
    }
}

/// The value text of a part in the panel's state, by its kind: knobs and faders
/// ([`control_value_text`]), switches (their words), displays (with their units). None for the
/// kinds whose state is a toggled state or nothing.
pub fn value_text(panel: &Panel, c: &Ctl) -> Option<String> {
    match c.kind.as_str() {
        "knob" | "fader" => Some(control_value_text(&c.id, panel.value(&c.id)?)),
        "switch" | "hswitch" | "vswitch" => {
            Some(switch_words(&c.id)[usize::from(panel.switch_position(&c.id).min(1))].into())
        }
        "display" => Some(display_text(&c.id, panel.display(&c.id))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use super::*;
    use crate::layout;

    #[test]
    fn every_layout_part_has_a_name_and_a_section() {
        for c in layout::controls() {
            let n = name(&c).unwrap_or_else(|| panic!("{} ({}) has no name", c.id, c.kind));
            assert!(!n.is_empty() && !n.contains('{'), "{}: {n:?}", c.id);
            assert!(section_name(&c.section).is_some(), "{}", c.section);
        }
    }

    #[test]
    fn every_table_entry_is_a_layout_part() {
        let parts: HashSet<String> = layout::controls()
            .iter()
            .flat_map(|c| {
                let (family, _) = family(&c.section);
                let key = format!("{family}.{}", c.name());
                [format!("{key}@{}", c.kind), key]
            })
            .collect();
        for k in table_keys() {
            assert!(parts.contains(k), "{k} is not in layout.tsv");
        }
    }

    #[test]
    fn names_are_unique_in_a_spoken_section() {
        let mut seen: HashMap<(&str, String), String> = HashMap::new();
        for c in layout::controls() {
            let key = (section_name(&c.section).unwrap(), name(&c).unwrap());
            if let Some(other) = seen.insert(key.clone(), c.id.clone()) {
                panic!("{} and {other} are both {:?} in {}", c.id, key.1, key.0);
            }
        }
    }

    #[test]
    fn some_names() {
        let get = |id: &str, kind: &str| {
            let c = layout::controls()
                .into_iter()
                .find(|c| c.id == id && c.kind == kind)
                .unwrap();
            (section_name(&c.section).unwrap(), name(&c).unwrap())
        };
        assert_eq!(
            get("tb303_2.cutoff", "knob"),
            ("Bass Line 2", "Cutoff".into())
        );
        assert_eq!(
            get("tb303_1.accent", "knob"),
            ("Bass Line 1", "Accent".into())
        );
        assert_eq!(
            get("tb303_1.accent", "trigger"),
            ("Bass Line 1", "Step accent".into())
        );
        assert_eq!(
            get("tb303_1.key6", "trigger"),
            ("Bass Line 1", "Key F sharp".into())
        );
        assert_eq!(
            get("mixer3.mute", "toggle"),
            ("TR-808", "Channel 3 on".into())
        );
        assert_eq!(get("pattern4.bankC", "grid"), ("TR-909", "Bank C".into()));
        assert_eq!(
            get("tr808.bd_level", "knob"),
            ("TR-808", "Bass drum level".into())
        );
        assert_eq!(
            get("comp.threshold", "knob"),
            ("Master and compressor", "Threshold".into())
        );
    }

    #[test]
    fn value_texts() {
        assert_eq!(control_value_text("tb303_1.cutoff", 100), "100");
        assert_eq!(control_value_text("mixer1.pan", 64), "center");
        assert_eq!(control_value_text("mixer1.pan", 44), "left 20");
        assert_eq!(control_value_text("delay.pan", 127), "right 63");
        assert_eq!(switch_words("transport.mode"), ["pattern", "song"]);
        assert_eq!(switch_words("pcf.mode"), ["band pass", "low pass"]);
        assert_eq!(switch_words("tb303_2.wave"), ["sawtooth", "square"]);
        assert_eq!(display_text("transport.tempo", Some(140)), "140 BPM");
        assert_eq!(display_text("transport.bar", None), "off");
        assert_eq!(display_text("transport.loop_length", Some(1)), "1 bar");
        assert_eq!(display_text("pattern2.steps", Some(16)), "16 steps");
        assert_eq!(display_text("tb303_1.edit_step", Some(5)), "5");
        assert_eq!(display_text("delay.steps", Some(3)), "3");
        assert_eq!(display_text("transport.bar", Some(3)), "3");
    }

    #[test]
    fn the_panel_values() {
        let p = Panel::demo();
        for c in layout::controls() {
            let v = value_text(&p, &c);
            let valued = matches!(
                c.kind.as_str(),
                "knob" | "fader" | "switch" | "hswitch" | "vswitch" | "display"
            );
            assert_eq!(v.is_some(), valued, "{} ({})", c.id, c.kind);
        }
    }
}
