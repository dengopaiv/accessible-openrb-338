//! task-197 AC #6: saving and restoring the state restores the song, the patterns, the
//! controls, the mode and an embedded third-party mod; instances are independent.

mod common;

use std::collections::BTreeMap;

use common::{activate, host, set, song_bytes, test_mod, with_song};
use nice_plug::prelude::Params;
use orb_engine::song::Device;
use orb_player::Command;
use orb_plugin::instance::{Kit, OrbParams};

const FRAMES: usize = 80_000;

/// What the host saves: the parameter values by id and the persisted fields.
fn save(p: &OrbParams) -> (Vec<(String, i32)>, BTreeMap<String, String>) {
    let values = p.ids().iter().cloned().zip(p.values()).collect();
    (values, p.serialize_fields())
}

/// What the host does on a restore (nice-plug's `deserialize_object`): the parameters, then
/// the fields.
fn restore(p: &OrbParams, saved: &(Vec<(String, i32)>, BTreeMap<String, String>)) {
    for (id, v) in &saved.0 {
        set(p, id, *v);
    }
    p.deserialize_fields(&saved.1);
}

/// An instance with a song, the test mod, pattern edits, changed controls, Vintage on and the
/// pattern mode.
fn edited() -> OrbParams {
    let p = with_song("full_mix");
    p.set_mod(Some(test_mod())).unwrap();
    for (device, step, field, value) in [
        (Device::Tb303_1, 3, 0, 9),
        (Device::Tr808, 1, 1, 1),
        (Device::Tr909, 2, 2, 2),
    ] {
        for pattern in 0..32 {
            p.edit(Command::StepField {
                device,
                pattern,
                step,
                field,
                value,
            })
            .unwrap();
        }
    }
    p.edit(Command::PatternField {
        device: Device::Tr909,
        pattern: 0,
        field: 1,
        value: 12,
    })
    .unwrap();
    for (id, v) in [
        ("d1c3", 100),
        ("d3c11", 1),
        ("d0c0", 80),
        ("d4c6", 40),
        ("g20", 100),
    ] {
        set(&p, id, v);
    }
    p.set_vintage(true).unwrap();
    p
}

#[test]
fn a_restored_state_plays_as_the_saved_one() {
    let a = edited();
    let saved = save(&a);
    let want = host(&a, &mut activate(&a, 44_100), FRAMES, 512, &[]);

    let b = OrbParams::new();
    restore(&b, &saved);
    assert_eq!(b.values(), a.values(), "the controls and the mode");
    let (da, db) = (a.main().doc.clone(), b.main().doc.clone());
    assert_eq!(db, da, "the song, patterns, sound model and mod");
    assert!(matches!(&db.kit, Kit::Mod { name, .. } if name == "Test Kit"));
    assert_eq!(db.song.as_deref(), Some(&song_bytes("full_mix")[..]));
    let got = host(&b, &mut activate(&b, 44_100), FRAMES, 512, &[]);
    let first = want.iter().zip(&got).position(|(x, y)| x != y);
    assert_eq!(first, None, "first difference at sample {first:?}");

    // what was restored matters: the unedited song, or the edited one with the built-in kit,
    // plays differently
    let plain = with_song("full_mix");
    assert_ne!(
        host(&plain, &mut activate(&plain, 44_100), FRAMES, 512, &[]),
        want
    );
    let c = OrbParams::new();
    restore(&c, &saved);
    c.set_mod(None).unwrap();
    assert_eq!(c.main().doc.kit, Kit::Standard);
    assert_ne!(host(&c, &mut activate(&c, 44_100), FRAMES, 512, &[]), want);
    // the built-in kit is not embedded
    assert!(!save(&c).1["orb"].contains("\"data\""));
    assert!(saved.1["orb"].contains("\"data\""));
}

/// A new instance is in Note Mode, and each mode survives a save and a restore (the persisted
/// fields are what the VST3 state carries, through nice-plug).
#[test]
fn the_mode_defaults_to_note_and_is_kept_in_the_state() {
    use orb_plugin::instance::Mode;
    assert_eq!(OrbParams::new().mode(), Mode::Note);
    for m in [Mode::Latch, Mode::Gate, Mode::Note] {
        let a = OrbParams::new();
        a.set_mode(m);
        let saved = save(&a);
        // restored into an instance in another mode
        let b = OrbParams::new();
        b.set_mode(if m == Mode::Gate {
            Mode::Latch
        } else {
            Mode::Gate
        });
        restore(&b, &saved);
        assert_eq!(b.mode(), m, "{m:?} survives the state");
    }
}

/// task-300: Options > Output Routing survives a save and a restore, by its stable keys; a state
/// without it (the released build's, v2026.1118.0919) restores with every box checked.
#[test]
fn the_output_routing_is_kept_in_the_state() {
    use orb_engine::frame::OutputRouting;
    assert_eq!(OrbParams::new().routing(), OutputRouting::default());
    let a = with_song("full_mix");
    let mut r = OutputRouting::default();
    r.to_mix[1] = false;
    r.to_808[2] = false;
    r.to_909[10] = false;
    a.set_routing(r);
    let saved = save(&a);
    let json = &saved.1["orb"];
    for key in [
        "\"mix.303-2\":false",
        "\"808.lt\":false",
        "\"909.rc\":false",
        "\"808.bd\":true",
    ] {
        assert!(json.contains(key), "{key} in {json:.200}");
    }
    let b = OrbParams::new();
    restore(&b, &saved);
    assert_eq!(b.routing(), r);
    // File > New keeps it
    b.new_song().unwrap();
    assert_eq!(b.routing(), r);
    // a state without the key: everything checked, the rest restored
    let mut old = saved.clone();
    let mut v: serde_json::Value = serde_json::from_str(&old.1["orb"]).unwrap();
    v.as_object_mut().unwrap().remove("routing").unwrap();
    old.1.insert("orb".into(), v.to_string());
    let c = OrbParams::new();
    c.set_routing(r);
    restore(&c, &old);
    assert_eq!(c.routing(), OutputRouting::default());
    assert_eq!(c.main().last_error, None);
    assert_eq!(c.main().doc.song, a.main().doc.song);
}

#[test]
fn a_restore_into_a_running_instance_replaces_its_song() {
    let a = edited();
    let saved = save(&a);
    let b = OrbParams::new();
    let mut core = activate(&b, 44_100);
    let silent = host(&b, &mut core, 4096, 512, &[]);
    assert!(silent.iter().all(|&x| x == 0.0), "a new song, stopped");
    restore(&b, &saved);
    let out = host(&b, &mut core, FRAMES, 512, &[]);
    assert!(common::peak(&out) > 0.05, "the restored song plays");
    assert_eq!(b.main().doc, a.main().doc);
    // the engine holds the edits: a 303 step and the 909 pattern length
    let e = core.audio().current();
    assert_eq!(
        orb_plugin::params::engine_byte(e, orb_plugin::params::Target::Ctrl(Device::Tb303_1, 3)),
        Some(100)
    );
    assert!(e.frame.vintage);
}

#[test]
fn instances_are_independent() {
    let a = with_song("full_mix");
    let b = with_song("full_mix");
    let mut ca = activate(&a, 44_100);
    let mut cb = activate(&b, 44_100);
    // interleaved calls, b with a change: a plays as alone
    let alone = {
        let x = with_song("full_mix");
        host(&x, &mut activate(&x, 44_100), FRAMES, 512, &[])
    };
    let mut out_a = Vec::new();
    for k in 0..FRAMES / 1000 {
        if k == 10 {
            set(&b, "d0c0", 10);
        }
        out_a.extend(host(&a, &mut ca, 1000, 1000, &[]));
        host(&b, &mut cb, 1000, 1000, &[]);
    }
    assert_eq!(out_a, alone);
}

#[test]
fn a_bad_state_gives_a_new_song() {
    let p = OrbParams::new();
    let mut f = BTreeMap::new();
    f.insert("orb".to_string(), "{\"version\": 99}".to_string());
    p.deserialize_fields(&f);
    assert!(p.main().last_error.is_some());
    assert_eq!(p.main().doc.song, None);
    let mut core = activate(&p, 44_100);
    host(&p, &mut core, 1000, 100, &[]);
}

/// The bytes of pattern `p` of row `r` in the patterns of a view.
fn pattern_of(patterns: &[Vec<u8>; 4], r: usize, p: usize) -> &[u8] {
    let len = if r < 2 { 98 } else { 194 };
    &patterns[r][p * len..(p + 1) * len]
}

/// task-288: a new instance, and File > New, hold the template in A1 of each row; every other
/// pattern is the same empty one.
#[test]
fn a_new_instance_and_file_new_hold_the_template() {
    let template = orb_ui::template::a1();
    let check = |p: &OrbParams, what: &str| {
        let (_, patterns) = p.view(&p.values());
        for (r, want) in template.iter().enumerate() {
            assert_eq!(pattern_of(&patterns, r, 0), want, "{what}: A1 of row {r}");
            for q in 2..32 {
                assert_eq!(
                    pattern_of(&patterns, r, q),
                    pattern_of(&patterns, r, 1),
                    "{what}: pattern {q} of row {r}"
                );
            }
            assert_ne!(pattern_of(&patterns, r, 1), want, "{what}: row {r}");
        }
    };
    check(&OrbParams::new(), "new instance");
    let q = with_song("full_mix");
    q.new_song().unwrap();
    check(&q, "File > New");
}

/// task-288: a saved state loads unchanged: one with no patterns of its own (saved before the
/// first build) does not get the template.
#[test]
fn a_saved_state_does_not_get_the_template() {
    let p = OrbParams::new();
    let mut f = BTreeMap::new();
    f.insert(
        "orb".to_string(),
        concat!(
            "{\"version\":1,\"song\":null,\"patterns\":[\"\",\"\",\"\",\"\"],\"vintage\":null,",
            "\"mod\":null,\"editor_size\":null,\"mode\":\"latch\",\"preview\":null}"
        )
        .to_string(),
    );
    p.deserialize_fields(&f);
    assert!(p.main().last_error.is_none(), "{:?}", p.main().last_error);
    assert!(p.main().doc.patterns.iter().all(|a| a.is_empty()));
    let (_, patterns) = p.view(&p.values());
    assert_ne!(pattern_of(&patterns, 0, 0), orb_ui::template::a1()[0]);
}

#[test]
fn the_logo_animation_plays_once_for_a_new_instance() {
    let p = OrbParams::new();
    assert!(p.take_intro(), "the first editor open of a new instance");
    assert!(!p.take_intro(), "a reopened editor does not replay it");
}

#[test]
fn a_restored_instance_never_plays_the_logo_animation() {
    let saved = save(&edited());
    // restored before the first editor open
    let p = OrbParams::new();
    restore(&p, &saved);
    assert!(!p.take_intro());
    // restored after the animation played: still not again
    let q = OrbParams::new();
    assert!(q.take_intro());
    restore(&q, &saved);
    assert!(!q.take_intro());
    // the flags are not in the state
    assert!(
        !saved
            .1
            .values()
            .any(|j| j.contains("intro") || j.contains("restored"))
    );
}
