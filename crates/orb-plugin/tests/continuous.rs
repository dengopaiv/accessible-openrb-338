//! task-275 (GitHub issue 21): the 0..127 parameters are continuous for the host, so that a
//! host draws and plays their automation as a ramp (Studio One drew a stepped parameter's as
//! steps); the engine still gets integers, and the released build's (v2026.1118.0919) saved
//! values, states and automation keep their meaning (`docs/plugin.md` sections 3 and 5). The
//! exported bundles' flags are checked in `exports.rs`.

mod common;

use common::{activate, set, with_song};
use nice_plug::plugin::{ParamValue, PluginState};
use nice_plug::prelude::{IntParam, IntRange, Param, Params, Plugin};
use orb_plugin::OrbPlugin;
use orb_plugin::instance::{HostParam, OrbParams, migrate_state, set_normalized};
use orb_plugin::params::{self, COUNT, Kind, SPECS, Target};

/// The released build's parameter of a spec: every one an `IntParam` over its range
/// (`OrbParams::new` at v2026.1118.0919).
fn released(i: usize) -> IntParam {
    let s = SPECS[i];
    IntParam::new(
        params::name(s.target),
        s.default as i32,
        IntRange::Linear {
            min: s.min as i32,
            max: s.max as i32,
        },
    )
}

/// The split: the `Kind::Value` parameters over 0..127 are continuous, the rest stepped.
#[test]
fn the_0_127_values_are_continuous_and_the_rest_stepped() {
    use orb_engine::song::Device::*;
    let p = OrbParams::new();
    let mut continuous = 0;
    for (i, s) in SPECS.iter().enumerate() {
        let want = s.kind == Kind::Value && (s.min, s.max) == (0, 127);
        assert_eq!(s.continuous(), want, "{}", params::id(s.target));
        assert_eq!(
            p.param_at(i).is_continuous(),
            want,
            "{}",
            params::id(s.target)
        );
        continuous += want as usize;
    }
    // what stays stepped: the switches, the patterns, the delay steps, the PCF wave and mode,
    // the inserts
    let stepped: Vec<Target> = SPECS
        .iter()
        .filter(|s| !s.continuous())
        .map(|s| s.target)
        .collect();
    for t in [
        Target::CompId,
        Target::PcfId,
        Target::Ctrl(Tb303_1, 1),
        Target::Ctrl(Tb303_2, 1),
        Target::Ctrl(Tr808, 1),
        Target::Ctrl(Tr909, 1),
        Target::Ctrl(Delay, 1),
        Target::Ctrl(Pcf, 4),
        Target::Ctrl(Pcf, 6),
        Target::Ctrl(Tb303_1, 8),
        Target::Ctrl(Tr808, 11),
        Target::Channel(0, 0),
        Target::Channel(3, 4),
    ] {
        assert!(stepped.contains(&t), "{t:?} stays stepped");
    }
    assert!(
        stepped
            .iter()
            .all(|t| SPECS.iter().find(|s| s.target == *t).unwrap().max < 127)
    );
    for t in [
        Target::Shuffle,
        Target::Master,
        Target::Channel(0, 1),
        Target::Channel(0, 2),
        Target::Channel(0, 3),
        Target::Ctrl(Tb303_2, 3),
        Target::Ctrl(Delay, 3),
        Target::Ctrl(Pcf, 1),
    ] {
        assert!(!stepped.contains(&t), "{t:?} is continuous");
    }
    // the counts docs/plugin.md section 3 gives
    assert_eq!((continuous, COUNT - continuous), (86, 33));
}

/// The normalized value of every integer is the released build's, bit for bit, and any host
/// value (automation saved with the released build, a ramp of a host) stands for the integer
/// the released build's `IntParam` made of it.
#[test]
fn the_normalized_mapping_is_the_released_builds() {
    let p = OrbParams::new();
    for (i, s) in SPECS.iter().enumerate() {
        let old = released(i);
        let new = p.param_at(i);
        for v in s.min as i32..=s.max as i32 {
            assert_eq!(
                new.preview_normalized(v).to_bits(),
                old.preview_normalized(v).to_bits(),
                "{} {v}",
                params::id(s.target)
            );
        }
        // the host's values: a dense grid, each step's value and the midpoints between them
        let n = (s.max - s.min) as u32;
        let grid = (0..=20_000).map(|k| k as f32 / 20_000.0);
        let steps = (0..=n).map(|k| k as f32 / n as f32);
        let mids = (0..n).map(|k| (k as f32 + 0.5) / n as f32);
        for x in grid.chain(steps).chain(mids) {
            unsafe { set_normalized(new, x) };
            assert_eq!(
                new.value(),
                old.preview_plain(x),
                "{} at {x}",
                params::id(s.target)
            );
        }
    }
}

/// The rule: the nearest integer, a half away from zero, within the range.
#[test]
fn a_continuous_value_rounds_to_the_nearest_integer() {
    let cutoff = SPECS
        .iter()
        .find(|s| s.target == Target::Ctrl(orb_engine::song::Device::Tb303_1, 3))
        .unwrap();
    for (x, want) in [
        (0.0, 0),
        (0.49, 0),
        (0.5, 1),
        (63.49, 63),
        (63.5, 64),
        (64.0, 64),
        (126.5, 127),
        (127.0, 127),
        (130.0, 127),
        (-0.4, 0),
        (-3.0, 0),
        (f32::NAN, 0),
    ] {
        assert_eq!(params::round(cutoff, x), want, "{x}");
    }
    let p = OrbParams::new();
    let c = p.param("d1c3").unwrap();
    assert!(c.is_continuous());
    for (plain, want) in [(63.4f32, 63), (63.5, 64), (63.6, 64), (64.49, 64)] {
        unsafe { set_normalized(c, plain / 127.0) };
        assert_eq!(c.value(), want, "{plain}");
    }
}

/// The host's text of a continuous parameter is the integer, and the integer's text parses to
/// its value.
#[test]
fn the_text_is_the_integer() {
    let p = OrbParams::new();
    let HostParam::Continuous(c, _) = p.param("d1c3").unwrap() else {
        panic!("d1c3 is continuous")
    };
    assert_eq!(c.normalized_value_to_string(64.0 / 127.0, false), "64");
    assert_eq!(c.normalized_value_to_string(63.6 / 127.0, false), "64");
    assert_eq!(c.normalized_value_to_string(63.4 / 127.0, false), "63");
    assert_eq!(c.string_to_normalized_value("64"), Some(64.0 / 127.0));
    assert_eq!(c.string_to_normalized_value(" 200 "), Some(1.0));
    assert_eq!(c.string_to_normalized_value("loud"), None);
    let HostParam::Continuous(pan, _) = p.param("d0c7").unwrap() else {
        panic!("channel 1 pan is continuous")
    };
    assert_eq!(pan.normalized_value_to_string(0.5, false), "64");
    // a stepped one is unchanged: the pattern's name
    let HostParam::Stepped(pat) = p.param("d1c1").unwrap() else {
        panic!("the pattern is stepped")
    };
    assert_eq!(pat.normalized_value_to_string(1.0, false), "D8");
}

/// A host ramp changes a continuous parameter by small amounts; only a change of the rounded
/// integer sends an engine message.
#[test]
fn only_a_change_of_the_integer_sends_a_message() {
    let p = with_song("full_mix");
    let id = "d1c3";
    set(&p, id, 60);
    let c = p.param(id).unwrap();
    let mut core = activate(&p, 44_100);
    let (mut l, mut r) = (vec![0f32; 64], vec![0f32; 64]);
    core.process(&mut l, &mut r, &p.values());
    let base = core.param_messages();
    // 59.5 .. 60.49: 100 host values, all 60
    for k in 0..100 {
        unsafe { set_normalized(c, (59.5 + k as f32 * 0.0099) / 127.0) };
        assert_eq!(c.value(), 60);
        core.process(&mut l, &mut r, &p.values());
    }
    assert_eq!(
        core.param_messages(),
        base,
        "no message while the integer holds"
    );
    // a ramp from 60 to 70 in steps of a tenth: one message per integer
    for k in 0..=100 {
        unsafe { set_normalized(c, (60.0 + k as f32 * 0.1) / 127.0) };
        core.process(&mut l, &mut r, &p.values());
    }
    assert_eq!(c.value(), 70);
    assert_eq!(core.param_messages(), base + 10, "one message per integer");
    let cutoff = params::engine_byte(
        core.audio().newest(),
        SPECS[p.ids().iter().position(|i| i == id).unwrap()].target,
    );
    assert_eq!(cutoff, Some(70));
}

/// A state the released build saved (every parameter `{"i32": v}`) restores the same values:
/// the continuous ones' become `F32` before nice-plug restores them (`filter_state`); the
/// stepped ones' are left as they are. The state the new build saves (`F32` for the continuous
/// ones) round-trips through the same filter.
#[test]
fn a_released_builds_state_restores_its_values() {
    let mut state = PluginState {
        version: "0.1.0".into(),
        params: SPECS
            .iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    params::id(s.target),
                    ParamValue::I32(s.min as i32 + i as i32 % 2),
                )
            })
            .collect(),
        fields: Default::default(),
    };
    migrate_state(&mut state);
    for (i, s) in SPECS.iter().enumerate() {
        let v = s.min as i32 + i as i32 % 2;
        match (&state.params[&params::id(s.target)], s.continuous()) {
            (ParamValue::F32(x), true) => assert_eq!(*x, v as f32),
            (ParamValue::I32(x), false) => assert_eq!(*x, v),
            (x, _) => panic!("{}: {x:?}", params::id(s.target)),
        }
    }
    // the new build's state: unchanged by the filter (a fractional host value included)
    state.params.insert("d1c3".into(), ParamValue::F32(63.75));
    let before = format!("{:?}", state.params);
    migrate_state(&mut state);
    assert_eq!(format!("{:?}", state.params), before);
}

/// task-291: nothing compares the state's `version` (nice-plug writes the plugin's `VERSION`
/// into it): a state labelled `0.1.0`, as the released build saved it, and the same state under
/// the new version string both go through the code that runs on a load (`filter_state`, then
/// nice-plug's restore of the parameters and the fields) and end with the same parameters and
/// the same song. (nice-plug's `deserialize_object` is crate-private, so the restore is the
/// one `state.rs` mirrors.)
#[test]
fn the_states_version_string_does_not_matter() {
    let src = with_song("full_mix");
    let make = |version: &str| PluginState {
        version: version.into(),
        params: src
            .ids()
            .iter()
            .cloned()
            .zip(src.values())
            .map(|(id, v)| (id, ParamValue::I32(v)))
            .collect(),
        fields: src.serialize_fields(),
    };
    let load = |version: &str| {
        let mut state = make(version);
        <OrbPlugin as Plugin>::filter_state(&mut state);
        assert_eq!(
            state.version, version,
            "the filter leaves the version alone"
        );
        let p = OrbParams::new();
        for (id, v) in &state.params {
            let v = match *v {
                ParamValue::I32(i) => i,
                ParamValue::F32(f) => f.round() as i32,
                ref x => panic!("{id}: {x:?}"),
            };
            set(&p, id, v);
        }
        p.deserialize_fields(&state.fields);
        p
    };
    let (old, new) = (load("0.1.0"), load(orb_ui::about::VERSION));
    assert_eq!(old.values(), new.values());
    assert_eq!(old.values(), src.values());
    assert_eq!(old.serialize_fields(), new.serialize_fields());
    assert!(old.song().is_some());
    assert_eq!(old.song(), new.song());
}
