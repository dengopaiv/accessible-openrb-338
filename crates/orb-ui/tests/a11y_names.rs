//! Every control of the panel has a name, a role and a value (task-186, A11Y-2,
//! docs/accessibility.md 3.3): the test walks `assets/ui/layout.tsv` and reads each part's
//! node in the accessibility tree the app builds headless.

#[path = "common/a11y.rs"]
mod a11y;

use std::collections::HashMap;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, Role, Toggled};
use orb_ui::a11y::{drive, names, units};
use orb_ui::layout;
use orb_ui::screen_reader::ScreenReaderSetting;

/// The role each interactive kind must have (A11Y-2).
fn expected_role(kind: &str) -> Role {
    match kind {
        "knob" | "fader" => Role::Slider,
        "toggle" => Role::CheckBox,
        "radio" | "grid" => Role::RadioButton,
        "trigger" | "repeat" | "wind" => Role::Button,
        "switch" | "hswitch" | "vswitch" => Role::Switch,
        k => panic!("{k} is not an interactive kind"),
    }
}

#[test]
fn every_interactive_part_has_a_name_a_role_and_a_value() {
    let h = AppTree::new();
    let t = h.tree();
    // (spoken section, name) -> layout id
    let mut seen: HashMap<(String, String), String> = HashMap::new();
    let mut checked = 0;
    for c in layout::controls() {
        // the arrows of a spin button and the pattern and bank buttons are read through their
        // spin button and pattern selector (task-187, tests/a11y_drive.rs)
        if !names::is_interactive(&c.kind) || drive::joined(&c) {
            continue;
        }
        let what = format!("{} ({})", c.id, c.kind);
        let n = t
            .by_layout(&c.kind, &c.id)
            .unwrap_or_else(|| panic!("{what} has no node"));
        let name = a11y::name(n).unwrap_or_default().to_string();
        assert!(!name.is_empty(), "{what} has no name");
        assert_eq!(a11y::role(n), expected_role(&c.kind), "{what}: role");
        assert!(a11y::supports(n, Action::Focus), "{what} takes the focus");
        match c.kind.as_str() {
            "knob" | "fader" => {
                let v = a11y::numeric_value(n).unwrap_or_else(|| panic!("{what}: no value"));
                assert!((0.0..=127.0).contains(&v), "{what}: {v}");
                assert!(a11y::value(n).is_some_and(|s| !s.is_empty()), "{what}");
            }
            "switch" | "hswitch" | "vswitch" => {
                let v = a11y::value(n).unwrap_or_else(|| panic!("{what}: no value"));
                assert!(names::switch_words(&c.id).contains(&v), "{what}: {v}");
            }
            "toggle" | "radio" | "grid" => {
                assert!(
                    matches!(a11y::toggled(n), Some(Toggled::True | Toggled::False)),
                    "{what}: no state"
                );
            }
            _ => {}
        }
        let section = names::section_name(&c.section).unwrap().to_string();
        if let Some(other) = seen.insert((section.clone(), name.clone()), c.id.clone()) {
            panic!("{} and {other} are both {name:?} in {section}", c.id);
        }
        checked += 1;
    }
    // 244 interactive parts, of which 20 arrows and 48 pattern and bank buttons are joined
    assert_eq!(
        checked,
        244 - 68,
        "interactive parts in layout.tsv with a node"
    );
}

/// The app with screen reader mode set to `mode` (not the system's answer), settled.
fn app_in(mode: ScreenReaderSetting, raw: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.settings.screen_reader = mode;
    h.app.settings.speak_raw_values = raw;
    h.frame(vec![]);
    h.frame(vec![]);
    h
}

#[test]
fn some_nodes_read_as_expected() {
    let h = app_in(ScreenReaderSetting::Off, false);
    let t = h.tree();
    let cutoff = t.by_layout("knob", "tb303_1.cutoff").unwrap();
    assert_eq!(a11y::name(cutoff), Some("Cutoff"));
    let v = h.app.panel.value("tb303_1.cutoff").unwrap();
    assert_eq!(a11y::numeric_value(cutoff), Some(f64::from(v)));
    assert_eq!(a11y::value(cutoff), Some(v.to_string().as_str()));

    let pan = t.by_layout("knob", "mixer2.pan").unwrap();
    assert_eq!(a11y::name(pan), Some("Channel 2 pan"));
    let v = h.app.panel.value("mixer2.pan").unwrap();
    let words = names::control_value_text(&h.app.panel, "mixer2.pan", v, names::Speech::Plain);
    assert_eq!(a11y::value(pan), Some(words.as_str()));

    let mode = t.by_layout("switch", "transport.mode").unwrap();
    assert_eq!(a11y::name(mode), Some("Mode"));
    let expected = if h.app.panel.transport.song_mode {
        "song"
    } else {
        "pattern"
    };
    assert_eq!(a11y::value(mode), Some(expected));

    let accent = t.by_layout("trigger", "tb303_1.accent").unwrap();
    assert_eq!(a11y::name(accent), Some("Step accent"));
    // the tempo display with its arrows is a spin button (task-187)
    let tempo = t.by_layout("display", "transport.tempo").unwrap();
    assert_eq!(a11y::role(tempo), Role::SpinButton);
    let bpm = h.app.panel.display("transport.tempo").unwrap();
    assert_eq!(a11y::name(tempo), Some("Tempo"));
    let text = format!("{bpm} BPM");
    assert_eq!(a11y::value(tempo), Some(text.as_str()));
    // the 303's edit step has no arrows: a label
    let edit = t.by_layout("display", "tb303_1.edit_step").unwrap();
    assert_eq!(a11y::role(edit), Role::Label);
    let step = h.app.panel.display("tb303_1.edit_step").unwrap();
    let text = format!("Edit step {step}");
    assert_eq!(a11y::name(edit), Some(text.as_str()));
    // UI Automation names a Label from its value
    assert_eq!(a11y::value(edit), Some(text.as_str()));
    assert!(
        !a11y::supports(edit, Action::Focus),
        "a display takes no focus"
    );
}

/// Task-192: in screen reader mode every knob and fader speaks its unit, with the number after
/// it when "Speak raw values" is on; with the mode off the number alone, as before.
#[test]
fn knobs_and_faders_speak_their_units_in_screen_reader_mode() {
    let knobs: Vec<_> = layout::controls()
        .into_iter()
        .filter(|c| c.kind == "knob" || c.kind == "fader")
        .collect();
    assert_eq!(knobs.len(), 86);
    for (mode, raw) in [
        (ScreenReaderSetting::On, false),
        (ScreenReaderSetting::On, true),
        (ScreenReaderSetting::Off, false),
        (ScreenReaderSetting::Off, true),
    ] {
        let h = app_in(mode, raw);
        let t = h.tree();
        for c in &knobs {
            let n = t.by_layout(&c.kind, &c.id).unwrap();
            let v = h.app.panel.value(&c.id).unwrap();
            let ctx = names::units_context(&h.app.panel, &c.id);
            let unit = units::text(&c.id, v, &ctx).unwrap();
            let want = match (mode, raw) {
                (ScreenReaderSetting::On, false) => unit,
                (ScreenReaderSetting::On, true) => format!("{unit}, {v}"),
                // task-186's text
                _ if c.id.ends_with(".pan") => {
                    let d = i32::from(v) - 64;
                    match d {
                        0 => "center".to_string(),
                        d if d < 0 => format!("left {}", -d),
                        d => format!("right {d}"),
                    }
                }
                _ => v.to_string(),
            };
            assert_eq!(
                a11y::value(n),
                Some(want.as_str()),
                "{} {mode:?} {raw}",
                c.id
            );
            assert_eq!(a11y::numeric_value(n), Some(f64::from(v)), "{}", c.id);
        }
    }
}

#[test]
fn the_unit_follows_the_controls_it_depends_on() {
    let mut h = app_in(ScreenReaderSetting::On, false);
    h.app.panel.set_knob("tb303_1.cutoff", 500);
    h.app.panel.set_knob("tb303_1.reso", 0);
    let id = a11y::hit_id("knob", "tb303_1.cutoff");
    let low = a11y::value(h.frame(vec![]).by_id(id).unwrap())
        .unwrap()
        .to_string();
    h.app.panel.set_knob("tb303_1.reso", 1000);
    let high = a11y::value(h.frame(vec![]).by_id(id).unwrap())
        .unwrap()
        .to_string();
    let v = h.app.panel.value("tb303_1.cutoff").unwrap();
    assert_ne!(low, high, "the resonance moves the 303's pole frequency");
    assert!(
        low.ends_with("hertz") && high.ends_with("hertz"),
        "{low} / {high}"
    );
    assert_eq!(
        low,
        names::control_value_text(
            &{
                let mut p = h.app.panel.clone();
                p.set_knob("tb303_1.reso", 0);
                p
            },
            "tb303_1.cutoff",
            v,
            names::Speech::Units
        )
    );
}

#[test]
fn the_nodes_follow_the_panel() {
    let mut h = app_in(ScreenReaderSetting::Off, false);
    let id = a11y::hit_id("toggle", "transport.loop");
    let before = h.app.panel.transport.looping;
    assert_eq!(
        a11y::toggled(h.tree().by_id(id).unwrap()),
        Some(before.into())
    );
    h.app.panel.press("transport.loop", 0.0);
    let after = a11y::toggled(h.frame(vec![]).by_id(id).unwrap());
    assert_eq!(after, Some((!before).into()));

    h.app.panel.set_knob("tb303_2.reso", 0);
    let n = h.frame(vec![]).by_layout("knob", "tb303_2.reso").unwrap();
    assert_eq!(a11y::numeric_value(n), Some(0.0));
    assert_eq!(a11y::value(n), Some("0"));
}

#[test]
fn leds_meters_steps_and_areas_take_no_focus() {
    let h = AppTree::new();
    let t = h.tree();
    for c in layout::controls() {
        if names::is_interactive(&c.kind) || c.kind == "display" {
            continue;
        }
        if let Some(n) = t.by_layout(&c.kind, &c.id) {
            assert!(
                !a11y::supports(n, Action::Focus),
                "{} can take the focus",
                c.id
            );
        }
    }
    let focusable = t
        .nodes()
        .filter(|(_, n)| a11y::supports(n, Action::Focus))
        .count();
    assert!(focusable < 254, "{focusable} focusable nodes");
}
