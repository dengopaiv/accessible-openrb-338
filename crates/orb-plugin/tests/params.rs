//! task-197 AC #5: every control the plugin's panel shows is a host parameter with a stable id.
//! `param_ids.txt` lists them (id, what it sets, range, default); a change of the list fails.

use std::sync::Arc;

use orb_engine::tables::Tables;
use orb_player::Player;
use orb_plugin::instance::{OrbParams, values_of};
use orb_plugin::params::{self, COUNT, SPECS, Target};

fn listing() -> String {
    SPECS
        .iter()
        .map(|s| {
            format!(
                "{}\t{:?}\t{}..={}\t{}\n",
                params::id(s.target),
                s.target,
                s.min,
                s.max,
                s.default
            )
        })
        .collect()
}

#[test]
fn the_ids_do_not_change() {
    let want = include_str!("param_ids.txt");
    let got = listing();
    if got != want {
        let first = want
            .lines()
            .zip(got.lines())
            .find(|(a, b)| a != b)
            .map(|(a, b)| format!("committed {a:?}, now {b:?}"));
        panic!(
            "the parameter list changed ({} lines committed, {} now; {first:?}). The ids are \
             permanent: hosts save automation and state under them. Now:\n{got}",
            want.lines().count(),
            got.lines().count()
        );
    }
}

#[test]
fn the_ids_are_unique_and_short() {
    let p = OrbParams::new();
    let mut ids = p.ids().to_vec();
    assert_eq!(ids.len(), COUNT);
    // nice-plug hashes the ids; it asks for up to 6 characters
    assert!(ids.iter().all(|i| i.len() <= 6), "{ids:?}");
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), COUNT);
}

/// The visible controls (task-185): every controller of the nine devices (the 303s' and drum
/// machines' on/off and pattern included, the five 808 voice switches 11, 14, 17, 19, 21), the
/// mixer's master, inserts, mutes, levels, pans, sends and distortion switches and the shuffle.
/// Hidden in the plugin and not parameters: the pattern/song switch, loop, Rewind, Record, the
/// tempo buttons; not parameters either: Play/Stop (the preview) and the mode (task-217).
#[test]
fn every_visible_control_is_a_parameter() {
    use orb_engine::song::Device::*;
    let has = |t: Target| SPECS.iter().any(|s| s.target == t);
    for (d, n) in [
        (Tb303_1, 9),
        (Tb303_2, 9),
        (Tr808, 29),
        (Tr909, 30),
        (Delay, 5),
        (Distortion, 3),
        (Pcf, 7),
        (Compressor, 3),
    ] {
        for c in 0..n {
            assert!(has(Target::Ctrl(d, c)), "{d:?} {c}");
        }
    }
    for c in [11, 14, 17, 19, 21] {
        let s = SPECS
            .iter()
            .find(|s| s.target == Target::Ctrl(Tr808, c))
            .unwrap();
        assert_eq!((s.min, s.max), (0, 1), "808 switch {c}");
    }
    for c in 0..4 {
        for k in 0..5 {
            assert!(has(Target::Channel(c, k)));
        }
    }
    for t in [
        Target::Master,
        Target::CompId,
        Target::PcfId,
        Target::Shuffle,
    ] {
        assert!(has(t), "{t:?}");
    }
}

/// task-217: the mode (`g1`, orb's Play/Stop until then) is no host parameter: it is the
/// plugin state's pattern/note mode, and Play/Stop are the preview.
#[test]
fn the_mode_is_no_host_parameter() {
    let p = OrbParams::new();
    assert!(p.param("g1").is_none());
    assert!(!p.ids().iter().any(|i| i == "g1"));
    assert!(!include_str!("param_ids.txt").contains("g1\t"));
    assert_eq!(COUNT, 119);
    assert_eq!(
        p.param("g20").unwrap().value(),
        64,
        "the shuffle keeps its id"
    );
}

/// The defaults are the new song's values (`docs/song_load.md` section 4.9): a new instance's
/// parameters change nothing in a new song.
#[test]
fn the_defaults_are_the_new_songs_values() {
    let (p, _a) = Player::new(
        Arc::new(Tables::new()),
        Some(rbm::standard::builtin_samples()),
    );
    let e = p.build_new(Some(rbm::standard::builtin_samples()));
    let engine = values_of(&e);
    let ours = OrbParams::new().values();
    for (i, s) in SPECS.iter().enumerate() {
        assert!(
            params::engine_byte(&e, s.target).is_some(),
            "{:?}",
            s.target
        );
        assert_eq!(ours[i], engine[i], "{}", params::id(s.target));
    }
}

#[test]
fn the_values_read_and_parse() {
    for s in &SPECS {
        for v in s.min as i32..=s.max as i32 {
            let t = params::display(s, v);
            assert_eq!(params::parse(s, &t), Some(v), "{:?} {v} {t:?}", s.target);
        }
    }
    let pat = SPECS
        .iter()
        .find(|s| s.target == Target::Ctrl(orb_engine::song::Device::Tr909, 1))
        .unwrap();
    assert_eq!(params::display(pat, 0), "A1");
    assert_eq!(params::display(pat, 31), "D8");
}
