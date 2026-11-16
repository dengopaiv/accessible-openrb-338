//! task-217: the pattern preview (the panel's Play) through the plugin's process path. The
//! preview plays the patterns free-running at the host's tempo (the song's when it reports
//! none), its step not following the host's position, whatever the host's transport does,
//! ignores host notes, the Standard Mapping and parameter changes, and plays the preview
//! set, which the state keeps; leaving it returns to the host's parameter values.

mod common;

use std::collections::BTreeMap;

use common::{activate, peak, set, with_song};
use nice_plug::prelude::Params;
use orb_engine::song::Device;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};
use orb_plugin::params::{self, Target};

/// One call of `n` frames; the output interleaved.
fn call(p: &OrbParams, core: &mut Core, n: usize, t: &HostTransport, ev: &[HostEvent]) -> Vec<f32> {
    let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
    core.process_with(&mut l, &mut r, &p.values(), t, ev);
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

fn index(p: &OrbParams, id: &str) -> usize {
    p.ids().iter().position(|x| x == id).unwrap()
}

/// The engine's byte of a parameter.
fn byte(core: &Core, t: Target) -> u8 {
    params::engine_byte(core.audio().current(), t).unwrap()
}

const CUTOFF: Target = Target::Ctrl(Device::Tb303_1, 3);

fn master(core: &Core) -> u8 {
    core.audio().current().frame.devices.mixer.live.master
}

/// A host stopped at 97.25 BPM: not the song's tempo.
const STOPPED: HostTransport = HostTransport {
    playing: false,
    tempo: Some(97.25),
    pos_beats: Some(3.0),
};

#[test]
fn the_preview_plays_free_running_at_the_hosts_tempo() {
    for mode in [Mode::Latch, Mode::Gate, Mode::Note] {
        for t in [
            STOPPED,
            HostTransport {
                playing: true,
                tempo: Some(151.0),
                pos_beats: Some(17.5),
            },
            // a host that reports no tempo: the song's
            HostTransport {
                playing: true,
                tempo: None,
                pos_beats: Some(17.5),
            },
        ] {
            let p = with_song("full_mix");
            p.set_mode(mode);
            let song_tempo = p.view(&p.values()).0.tempo;
            let mut core = activate(&p, 44_100);
            call(&p, &mut core, 4096, &STOPPED, &[]);
            assert!(!core.audio().last_snapshot().playing, "{mode:?}: stopped");
            p.set_preview(true);
            let out = call(&p, &mut core, 40_000, &t, &[]);
            assert!(peak(&out) > 0.05, "{mode:?} {t:?}: the preview plays");
            let s = core.audio().last_snapshot();
            assert!(s.playing && !s.host_sync && !core.following());
            assert!(core.previewing());
            // the host's tempo (the user, 2026-11-16), also with the host stopped; the
            // song's when the host reports none
            let tempo = t.tempo.map_or(song_tempo, |b| (b * 1000.0).round() as u32);
            assert_eq!(s.tempo, tempo, "{mode:?} {t:?}");
            assert!(
                ![97_250, 151_000].contains(&song_tempo),
                "the tempos differ"
            );
            // free-running: the position is the engine's own (from the stopped song's
            // bar start), not the host's
            let ticks = 40_000.0 * tempo as f64 * orb_engine::seq::TEMPO_STEP / 64.0;
            assert!(
                (s.tick as f64 - ticks).abs() < 200.0,
                "{mode:?} {t:?}: tick {} for {ticks}",
                s.tick
            );
            // Stop leaves it: back to the mode
            p.set_preview(false);
            call(&p, &mut core, 512, &t, &[]);
            assert!(!core.previewing());
            let s = core.audio().last_snapshot();
            assert_eq!(s.playing, mode.patterns() && t.playing, "{mode:?} {t:?}");
        }
    }
}

/// Host input during the preview changes neither the sound nor what the panel shows (the
/// snapshot); the MIDI LED still counts it.
#[test]
fn the_preview_ignores_host_input() {
    let quiet = {
        let p = with_song("full_mix");
        let mut core = activate(&p, 44_100);
        p.set_preview(true);
        let mut out = Vec::new();
        for _ in 0..80 {
            out.extend(call(&p, &mut core, 512, &STOPPED, &[]));
        }
        (out, *core.audio().last_snapshot())
    };
    let p = with_song("full_mix");
    p.set_remote_enabled(true);
    let mut core = activate(&p, 44_100);
    p.set_preview(true);
    let note = |frame, channel, note| HostEvent {
        frame,
        message: Message::NoteOn {
            channel,
            note,
            velocity: 127,
        },
    };
    let mut out = Vec::new();
    for k in 0..80 {
        // the host plays and jumps, sends pattern-select and device notes, the Standard
        // Mapping's master volume, its Stop note and automation of a knob and a pattern
        let t = HostTransport {
            playing: k % 3 != 0,
            // the tempo is the host's in the preview too: the same as the quiet run's
            tempo: STOPPED.tempo,
            pos_beats: Some((k * 7 % 13) as f64),
        };
        if k == 10 {
            set(&p, "d1c3", 5);
            set(&p, "d1c1", 9);
            set(&p, "d0c0", 20);
        }
        let ev = [
            note(0, 0, (k % 32) as u8),
            note(5, 2, 36),
            note(9, 3, 40),
            HostEvent {
                frame: 20,
                message: Message::ControlChange {
                    channel: 15,
                    controller: 7,
                    value: 3,
                },
            },
            note(30, 15, 70),
        ];
        out.extend(call(&p, &mut core, 512, &t, &ev));
    }
    let first = quiet.0.iter().zip(&out).position(|(a, b)| a != b);
    assert_eq!(first, None, "the sound changed at sample {first:?}");
    let s = core.audio().last_snapshot();
    assert_eq!(s.live, quiet.1.live, "the devices' values (the panel's)");
    assert_eq!(s.mixer, quiet.1.mixer);
    assert_eq!(s.patterns, quiet.1.patterns);
    assert_eq!(s.tick, quiet.1.tick);
    assert_eq!(s.midi_in, 80 * 5, "the MIDI LED counts the messages");
    assert_eq!(quiet.1.midi_in, 0);
    // the host's values are kept: leaving the preview plays them
    p.set_preview(false);
    let t = HostTransport {
        playing: true,
        tempo: Some(120.0),
        pos_beats: Some(0.0),
    };
    call(&p, &mut core, 512, &t, &[]);
    assert_eq!(byte(&core, CUTOFF), 5);
    assert_eq!(master(&core), 20);
    assert_eq!(selected_303(&core), 9);
}

fn selected_303(core: &Core) -> u8 {
    byte(core, Target::Ctrl(Device::Tb303_1, 1))
}

#[test]
fn the_preview_set_is_kept_apart_from_the_host_values() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let t = STOPPED;
    call(&p, &mut core, 512, &t, &[]);
    let host_cutoff = p.param("d1c3").unwrap().value();
    let host_master = p.param("d0c0").unwrap().value();
    let host_pattern = p.param("d1c1").unwrap().value();
    assert_eq!(p.preview_values(), None, "no preview yet");
    assert!(
        p.set_preview_value(0, 1).is_err(),
        "only during the preview"
    );
    // the first entry starts from the host's values
    p.set_preview(true);
    assert_eq!(p.preview_values(), Some(p.values()));
    p.set_preview_value(index(&p, "d1c3"), 20).unwrap();
    p.set_preview_value(index(&p, "d1c1"), 7).unwrap();
    p.set_preview_value(index(&p, "d0c0"), 300).unwrap();
    call(&p, &mut core, 512, &t, &[]);
    assert_eq!(byte(&core, CUTOFF), 20);
    assert_eq!(master(&core), 127, "clamped to the range");
    assert_eq!(selected_303(&core), 7);
    assert_eq!(p.panel_values()[index(&p, "d1c3")], 20);
    // the host's parameters are untouched; the host changes one meanwhile
    assert_eq!(p.param("d1c3").unwrap().value(), host_cutoff);
    assert_eq!(p.param("d1c1").unwrap().value(), host_pattern);
    set(&p, "d0c0", 33);
    call(&p, &mut core, 512, &t, &[]);
    assert_eq!(master(&core), 127);
    // leaving: the engine plays the host's latest values
    p.set_preview(false);
    call(&p, &mut core, 512, &t, &[]);
    assert_eq!(byte(&core, CUTOFF), host_cutoff as u8);
    assert_eq!(master(&core), 33);
    assert_eq!(selected_303(&core), host_pattern as u8);
    assert_eq!(p.panel_values(), p.values());
    // the next entry restores the set
    p.set_preview(true);
    call(&p, &mut core, 512, &t, &[]);
    assert_eq!(byte(&core, CUTOFF), 20);
    assert_eq!(master(&core), 127);
    assert_eq!(selected_303(&core), 7);
    assert_ne!(host_master, 127);

    // the set and the mode survive a save and restore; the preview itself is not saved
    let saved: Vec<(String, i32)> = p.ids().iter().cloned().zip(p.values()).collect();
    let fields: BTreeMap<String, String> = p.serialize_fields();
    let q = OrbParams::new();
    assert_eq!(q.mode(), Mode::Note, "a new instance is in note mode");
    for (id, v) in &saved {
        set(&q, id, *v);
    }
    q.deserialize_fields(&fields);
    assert_eq!(q.preview_values(), p.preview_values());
    assert_eq!(q.mode(), Mode::Latch);
    assert!(!q.previewing());
    assert_eq!(q.main().doc, p.main().doc);
    let mut qc = activate(&q, 44_100);
    call(&q, &mut qc, 512, &t, &[]);
    assert_eq!(byte(&qc, CUTOFF), host_cutoff as u8, "the host's values");
    q.set_preview(true);
    call(&q, &mut qc, 512, &t, &[]);
    assert_eq!(byte(&qc, CUTOFF), 20, "the restored preview set");
    assert_eq!(selected_303(&qc), 7);
    // a restore ends a running preview
    q.deserialize_fields(&fields);
    assert!(!q.previewing());
}

#[test]
fn a_song_loaded_during_the_preview_is_previewed_with_its_own_values() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    p.set_preview(true);
    p.set_preview_value(index(&p, "d1c3"), 3).unwrap();
    call(&p, &mut core, 4096, &STOPPED, &[]);
    let values = p.load_song(common::song_bytes("303_accent127")).unwrap();
    for (id, v) in &values {
        set(&p, id, *v);
    }
    call(&p, &mut core, 4096, &STOPPED, &[]);
    assert!(core.audio().last_snapshot().playing, "the preview plays on");
    let want: Vec<i32> = values.iter().map(|x| x.1).collect();
    assert_eq!(p.preview_values().unwrap().to_vec(), want);
    let cutoff = values.iter().find(|x| x.0 == "d1c3").unwrap().1;
    assert_eq!(byte(&core, CUTOFF) as i32, cutoff);
    // not in the preview, a load forgets the set: the next entry starts from the new values
    p.set_preview(false);
    p.load_song(common::song_bytes("full_mix")).unwrap();
    assert_eq!(p.preview_values(), None);
}
