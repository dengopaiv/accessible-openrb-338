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
/// mode at Play.
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
