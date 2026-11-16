//! task-198: host sync and pattern mode. A simulated host transport (start, seek, loop jump,
//! tempo change, stop), pattern selection by notes on channels 1..4, the Standard Mapping (off by
//! default, channel 16) on its channel, deterministic renders, and the snapshot's Sync and MIDI
//! flags.

mod common;

use std::collections::BTreeMap;

use common::{Change, activate, peak, set, with_song};
use nice_plug::prelude::Params;
use orb_engine::seq::{TEMPO_STEP, shuffled};
use orb_engine::song::Device;
use orb_player::Command;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport, TICKS_PER_BEAT};
use orb_plugin::instance::{Mode, OrbParams};

/// A stretch of the host's timeline from host frame `at`: playing or not, its tempo, and its
/// position in beats at `at`.
#[derive(Clone, Copy, Debug)]
struct Seg {
    at: usize,
    playing: bool,
    tempo: Option<f64>,
    beats: f64,
}

/// The host's transport, as a list of stretches (a new one is a start, a stop, a seek, a loop
/// jump or a tempo change). The host splits its buffers at each stretch's start.
struct Timeline(Vec<Seg>);

impl Timeline {
    fn seg(&self, f: usize) -> Seg {
        *self.0.iter().rev().find(|s| s.at <= f).unwrap()
    }

    /// The transport at host frame `f` (44.1 kHz).
    fn at(&self, f: usize) -> HostTransport {
        let s = self.seg(f);
        let beats = match (s.playing, s.tempo) {
            (true, Some(t)) => s.beats + (f - s.at) as f64 * t / 60.0 / 44_100.0,
            _ => s.beats,
        };
        HostTransport {
            playing: s.playing,
            tempo: s.tempo,
            pos_beats: Some(beats),
        }
    }
}

/// A MIDI message at a host frame.
#[derive(Clone, Copy, Debug)]
struct Midi {
    at: usize,
    message: Message,
}

fn note_on(at: usize, channel: u8, note: u8) -> Midi {
    Midi {
        at,
        message: Message::NoteOn {
            channel,
            note,
            velocity: 100,
        },
    }
}

/// Runs `frames` host frames at 44.1 kHz in buffers of `buf`, split at the timeline's stretches
/// and the parameter changes; `check` runs after each call. Returns the output, interleaved.
#[allow(clippy::too_many_arguments)]
fn run(
    p: &OrbParams,
    core: &mut Core,
    tl: &Timeline,
    frames: usize,
    buf: usize,
    midi: &[Midi],
    changes: &[Change],
    mut check: impl FnMut(&Core, usize),
) -> Vec<f32> {
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mut pos = 0;
    while pos < frames {
        for c in changes.iter().filter(|c| c.at == pos) {
            set(p, c.id, c.value);
        }
        let next =
            tl.0.iter()
                .map(|s| s.at)
                .chain(changes.iter().map(|c| c.at))
                .filter(|&a| a > pos)
                .min()
                .unwrap_or(usize::MAX);
        let end = ((pos / buf + 1) * buf).min(next).min(frames);
        let events: Vec<HostEvent> = midi
            .iter()
            .filter(|m| (pos..end).contains(&m.at))
            .map(|m| HostEvent {
                frame: (m.at - pos) as u32,
                message: m.message,
            })
            .collect();
        core.process_with(
            &mut l[pos..end],
            &mut r[pos..end],
            &p.values(),
            &tl.at(pos),
            &events,
        );
        pos = end;
        check(core, pos);
    }
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

/// The engine's position in ticks.
fn pos(core: &Core) -> f64 {
    core.audio().current().frame.seq.pos
}

/// The host's position in ticks at the timeline frame of the last block rendered, and where
/// the block's clock puts it (one block's advance at the engine's tempo later).
fn host_ticks(core: &Core, tl: &Timeline) -> (f64, f64) {
    let h = 64 * (core.blocks() as usize - 1);
    let t = tl.at(h).pos_beats.unwrap() * TICKS_PER_BEAT;
    let tempo = core.audio().current().frame.seq.globals.tempo;
    (t, t + tempo as f64 * TEMPO_STEP)
}

/// The 303 #1's step check: when the position is well inside a step (26..44 ticks in, the
/// gate's 24 ticks past), the device's last step (live byte 9) is the step the position gives.
fn step_matches(p: &OrbParams, core: &Core) -> Option<bool> {
    let e = core.audio().current();
    let t = e.frame.seq.tick_pos();
    let pat = core.audio().last_snapshot().patterns[0]? as usize;
    let doc = &p.main().doc;
    let (shuffle, len) = (doc.patterns[0][98 * pat], doc.patterns[0][98 * pat + 1]);
    let tt = if shuffle != 0 {
        shuffled(t, e.frame.seq.globals.shuffle)
    } else {
        t
    };
    let v = tt % (48 * len as i32);
    if !(26..=44).contains(&(v % 48)) {
        return None;
    }
    let live = core.audio().last_snapshot().live[1];
    Some(live.as_slice()[9] as i32 == v / 48)
}

/// The host's timeline of the sync test: stopped at beat 0 for 20000 frames (orb free-runs),
/// started at frame 20000, a seek to beat 37.25 at 60000, the tempo to 133.5 at 80000, a loop
/// jump back to beat 4 at 100000, stopped at 130000.
fn timeline() -> Timeline {
    let seg = |at, playing, tempo, beats| Seg {
        at,
        playing,
        tempo: Some(tempo),
        beats,
    };
    let tl = Timeline(vec![
        seg(0, false, 120.0, 0.0),
        seg(20_000, true, 120.0, 0.0),
        seg(60_000, true, 120.0, 37.25),
        seg(80_000, true, 133.5, 0.0),
        seg(100_000, true, 133.5, 4.0),
        seg(130_000, false, 133.5, 0.0),
    ]);
    // the tempo change keeps the position: the stretch starts where the last one is
    let mut tl = tl;
    tl.0[3].beats = tl.at(80_000).pos_beats.unwrap();
    tl.0[5].beats = tl.at(130_000).pos_beats.unwrap();
    tl
}

#[test]
fn the_step_follows_the_host_position_and_tempo() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let tl = timeline();
    let mut followed = 0;
    let mut steps_checked = 0;
    let mut free_run = Vec::new();
    let out = run(&p, &mut core, &tl, 150_000, 512, &[], &[], |core, end| {
        let s = tl.seg(64 * (core.blocks() as usize).saturating_sub(1));
        if core.following() {
            assert!(s.playing, "follows only while the host plays (frame {end})");
            let (h, after) = host_ticks(core, &tl);
            // the position is the host's, advanced by the block's clock
            assert!(
                (pos(core) - after).abs() < 1e-6,
                "frame {end}: pos {} host {h}",
                pos(core)
            );
            let tempo = core.audio().current().frame.seq.globals.tempo;
            assert_eq!(tempo, (s.tempo.unwrap() * 1000.0).round() as u32);
            followed += 1;
            // two blocks after a jump the step has caught up
            if 64 * core.blocks() as usize >= s.at + 3 * 64
                && let Some(ok) = step_matches(&p, core)
            {
                assert!(ok, "frame {end}: the 303's step is not the position's");
                steps_checked += 1;
            }
        } else {
            assert!(
                !s.playing || 64 * core.blocks() as usize <= s.at + 64,
                "frame {end}: not following while the host plays"
            );
            free_run.push((end, pos(core)));
        }
    });
    assert!(followed > 200, "{followed} calls followed");
    assert!(steps_checked > 50, "{steps_checked} steps checked");
    // a stopped host (task-217): orb is stopped too, the position stands still
    let before: Vec<_> = free_run.iter().filter(|(e, _)| *e < 20_000).collect();
    let after: Vec<_> = free_run.iter().filter(|(e, _)| *e > 131_000).collect();
    for run in [before, after] {
        let (first, last) = (run[0].1, run[run.len() - 1].1);
        assert_eq!(first, last, "stopped from {first} to {last}");
    }
    assert!(!core.audio().last_snapshot().playing);
    assert_eq!(
        peak(&out[..2 * 19_000]),
        0.0,
        "silent before the host plays"
    );
    assert!(
        peak(&out[2 * 40_000..2 * 60_000]) > 0.05,
        "plays with the host"
    );
    // after the release (the delay's tail) silent once the host stops
    assert!(
        peak(&out[2 * 140_000..]) < 0.01,
        "quiet after the host stops: {}",
        peak(&out[2 * 140_000..])
    );
}

#[test]
fn the_host_start_restarts_the_patterns_at_its_position() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    // the host starts at bar 3 (beat 8) after orb free-ran mid-pattern
    let tl = Timeline(vec![
        Seg {
            at: 0,
            playing: false,
            tempo: Some(120.0),
            beats: 8.0,
        },
        Seg {
            at: 30_016,
            playing: true,
            tempo: Some(120.0),
            beats: 8.0,
        },
    ]);
    run(&p, &mut core, &tl, 30_016 + 64 * 3, 64, &[], &[], |_, _| {});
    // the block of frame 30016 located to tick 1536 and played on
    let seq = &core.audio().current().frame.seq;
    assert!(
        seq.pos > 1536.0 && seq.pos < 1536.0 + 3.0,
        "pos {}",
        seq.pos
    );
}

/// A seek or loop jump is a locate, as the original's TopHat reposition (`sub_1001C3C4`): the
/// devices are relocated, so a panel pattern waiting for the bar line plays from the jump on.
#[test]
fn a_jump_locates() {
    let tl = Timeline(vec![
        Seg {
            at: 0,
            playing: true,
            tempo: Some(120.0),
            beats: 0.0,
        },
        // a loop jump back to beat 1 in the middle of the first bar
        Seg {
            at: 40_000,
            playing: true,
            tempo: Some(120.0),
            beats: 1.0,
        },
    ]);
    let changes = [Change {
        at: 20_000,
        id: "d1c1",
        value: 12,
    }];
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let mut at = BTreeMap::new();
    run(
        &p,
        &mut core,
        &tl,
        45_000,
        64,
        &[],
        &changes,
        |core, end| {
            at.insert(end, playing(core)[0]);
        },
    );
    assert_ne!(at[&39_936], Some(12), "waits for the bar line");
    assert_eq!(at[&(40_000 / 64 * 64 + 128)], Some(12), "taken at the jump");
}

#[test]
fn a_host_tempo_change_changes_orbs_and_the_songs_comes_back() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let none = Timeline(vec![Seg {
        at: 0,
        playing: false,
        tempo: None,
        beats: 0.0,
    }]);
    run(&p, &mut core, &none, 2048, 512, &[], &[], |_, _| {});
    let song = core.audio().current().frame.seq.globals.tempo;
    let tl = Timeline(vec![
        Seg {
            at: 0,
            playing: false,
            tempo: Some(97.25),
            beats: 0.0,
        },
        Seg {
            at: 4096,
            playing: true,
            tempo: Some(151.0),
            beats: 0.0,
        },
        Seg {
            at: 8192,
            playing: true,
            tempo: None,
            beats: 3.0,
        },
    ]);
    let mut seen = Vec::new();
    run(&p, &mut core, &tl, 12_288, 512, &[], &[], |core, end| {
        seen.push((end, core.audio().current().frame.seq.globals.tempo));
    });
    let tempo_at = |f: usize| seen.iter().find(|(e, _)| *e == f).unwrap().1;
    assert_eq!(tempo_at(4096), 97_250);
    assert_eq!(tempo_at(8192), 151_000);
    assert_eq!(
        tempo_at(12_288),
        song,
        "the song's tempo when the host reports none"
    );
    assert_ne!(song, 151_000);
    // the snapshot shows the tempo orb plays at
    assert_eq!(p.snapshot().unwrap().tempo, song);
}

/// The pattern each section plays now.
fn playing(core: &Core) -> [Option<u8>; 4] {
    core.audio().last_snapshot().patterns
}

#[test]
fn notes_select_patterns_at_once_and_latch() {
    {
        let p = with_song("full_mix");
        let mut core = activate(&p, 44_100);
        let tl = Timeline(vec![Seg {
            at: 0,
            playing: true,
            tempo: Some(120.0),
            beats: 0.0,
        }]);
        // 120 BPM: a bar is 88200 frames; the notes fall inside bars
        let midi = [
            note_on(10_000, 0, 5),
            // a note-off and a velocity-0 note-on are ignored; a note above 31 too
            Midi {
                at: 20_000,
                message: Message::NoteOff {
                    channel: 0,
                    note: 7,
                    velocity: 64,
                },
            },
            Midi {
                at: 20_001,
                message: Message::NoteOn {
                    channel: 0,
                    note: 8,
                    velocity: 0,
                },
            },
            note_on(20_002, 0, 40),
            note_on(30_000, 1, 31),
            note_on(30_000, 2, 9),
            note_on(30_000, 3, 17),
            // channels 5..16 (0-based 4..15) do not select patterns
            note_on(35_000, 4, 20),
            note_on(35_000, 15, 21),
            note_on(130_000, 0, 2),
        ];
        // the panel selects 303 #1's pattern 12 at frame 40000: it waits for the bar line
        let changes = [Change {
            at: 40_000,
            id: "d1c1",
            value: 12,
        }];
        let start = playing(&{
            let p = with_song("full_mix");
            let mut c = activate(&p, 44_100);
            run(&p, &mut c, &tl, 1024, 512, &[], &[], |_, _| {});
            c
        });
        let mut at = BTreeMap::new();
        run(
            &p,
            &mut core,
            &tl,
            140_000,
            64,
            &midi,
            &changes,
            |core, end| {
                at.insert(end, playing(core));
            },
        );
        // a note at frame t is in the block rendered after it, one block (64 frames) later
        let after = |f: usize| at[&(f / 64 * 64 + 128)];
        assert_ne!(start[0], Some(5));
        assert_eq!(after(10_000)[0], Some(5), "at once, not at the bar line");
        assert_eq!(
            after(20_002)[0],
            Some(5),
            "note-offs and notes above 31 ignored"
        );
        assert_eq!(after(30_000)[1..], [Some(31), Some(9), Some(17)]);
        assert_eq!(after(35_000)[..], after(30_000)[..], "channels 5..16");
        // the panel's change waits for the bar line (frame 88200) and then wins
        assert_eq!(after(80_000)[0], Some(5));
        assert_eq!(after(90_000)[0], Some(12), "the panel wins after the note");
        // until the next note
        assert_eq!(after(130_000)[0], Some(2));
    }
}

/// The pattern each section has selected (the pattern controller's byte, what the panel shows).
fn selected(core: &Core) -> [u8; 4] {
    let s = core.audio().last_snapshot();
    // before the first block the snapshot is empty
    [1, 2, 3, 4].map(|d| s.live[d].as_slice().get(1).copied().unwrap_or(u8::MAX))
}

/// Pattern mode with the host stopped (task-217): no sound, and a pattern-select note only
/// changes the section's selected pattern, which plays when the host starts.
#[test]
fn with_the_host_stopped_notes_only_select() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let tl = Timeline(vec![
        Seg {
            at: 0,
            playing: false,
            tempo: Some(120.0),
            beats: 0.0,
        },
        Seg {
            at: 40_000,
            playing: true,
            tempo: Some(120.0),
            beats: 0.0,
        },
    ]);
    let midi = [note_on(10_000, 0, 5), note_on(20_000, 3, 17)];
    let mut at = BTreeMap::new();
    let out = run(&p, &mut core, &tl, 50_000, 64, &midi, &[], |core, end| {
        at.insert(end, (selected(core), core.audio().last_snapshot().playing));
    });
    assert_eq!(
        peak(&out[..2 * 40_000]),
        0.0,
        "no sound while the host is stopped"
    );
    let after = |f: usize| at[&(f / 64 * 64 + 128)];
    assert_ne!(at[&9_984].0[0], 5);
    assert_eq!(after(10_000).0[0], 5, "the selection changes");
    assert_eq!(after(20_000).0[3], 17);
    assert!(!after(30_000).1, "stopped");
    // the host starts: the selected patterns play from its position
    assert!(after(41_000).1);
    assert_eq!(playing(&core)[0], Some(5));
    assert_eq!(playing(&core)[3], Some(17));
    assert!(peak(&out[2 * 41_000..]) > 0.05, "plays with the host");
}

#[test]
fn notes_on_the_section_channels_do_nothing_in_note_mode() {
    let p = with_song("full_mix");
    p.set_mode(Mode::Note);
    let mut core = activate(&p, 44_100);
    let tl = Timeline(vec![Seg {
        at: 0,
        playing: true,
        tempo: Some(120.0),
        beats: 0.0,
    }]);
    let before = {
        let mut c = activate(&p, 44_100);
        run(&p, &mut c, &tl, 1024, 512, &[], &[], |_, _| {});
        c.audio().current().frame.devices.channels[0].live()[1]
    };
    run(
        &p,
        &mut core,
        &tl,
        4096,
        512,
        &[note_on(100, 0, 7)],
        &[],
        |c, _| {
            assert!(!c.following(), "no sync in note mode");
        },
    );
    let after = core.audio().current().frame.devices.channels[0].live()[1];
    assert_eq!(before, after);
}

/// The host timeline of the determinism tests: a tempo whose frame step is exact in binary
/// (161.4990234375 BPM is 1/16384 beat per frame at 44.1 kHz), so that the host's positions do
/// not depend on where its buffers start, and a start, a seek, a loop jump and a stop.
fn exact_timeline() -> Timeline {
    let t = Some(2_646_000.0 / 16_384.0);
    let seg = |at, playing, beats| Seg {
        at,
        playing,
        tempo: t,
        beats,
    };
    Timeline(vec![
        seg(0, false, 0.0),
        seg(10_000, true, 0.0),
        seg(50_000, true, 21.5),
        seg(90_000, true, 4.0),
        seg(110_000, false, 4.0 + 20_000.0 / 16_384.0),
    ])
}

fn exact_midi() -> Vec<Midi> {
    vec![
        note_on(15_000, 0, 3),
        note_on(33_333, 2, 6),
        note_on(70_001, 3, 30),
        note_on(95_000, 1, 1),
        Midi {
            at: 99_999,
            message: Message::ControlChange {
                channel: 15,
                controller: 7,
                value: 40,
            },
        },
    ]
}

fn render(buf: usize) -> Vec<f32> {
    let p = with_song("full_mix");
    p.set_remote_enabled(true);
    let mut core = activate(&p, 44_100);
    let changes = [Change {
        at: 60_000,
        id: "d4c1",
        value: 4,
    }];
    run(
        &p,
        &mut core,
        &exact_timeline(),
        130_000,
        buf,
        &exact_midi(),
        &changes,
        |_, _| {},
    )
}

#[test]
fn two_renders_of_the_same_host_timeline_are_identical() {
    let want = render(512);
    assert!(peak(&want) > 0.05);
    assert_eq!(render(512), want, "the same buffers");
    for buf in [1, 63, 64, 4096] {
        let got = render(buf);
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert_eq!(
            first, None,
            "buffer size {buf}: first difference at {first:?}"
        );
    }
}

fn master(core: &Core) -> u8 {
    core.audio().current().frame.devices.mixer.live.master
}

fn cc(at: usize, channel: u8, controller: u8, value: u8) -> Midi {
    Midi {
        at,
        message: Message::ControlChange {
            channel,
            controller,
            value,
        },
    }
}

#[test]
fn the_standard_mapping_works_on_its_channel() {
    let p = with_song("full_mix");
    let tl = Timeline(vec![Seg {
        at: 0,
        playing: true,
        tempo: Some(120.0),
        beats: 0.0,
    }]);
    // off by default, on channel 16 (0-based 15) when turned on
    assert!(!p.remote_enabled());
    assert_eq!(p.remote_channel(), 15);
    let mut core = activate(&p, 44_100);
    let before = master(&core);
    assert_ne!(before, 33);
    run(
        &p,
        &mut core,
        &tl,
        512,
        512,
        &[cc(100, 15, 7, 33), cc(101, 0, 7, 34)],
        &[],
        |_, _| {},
    );
    assert_eq!(master(&core), before, "off: nothing is mapped");
    p.set_remote_enabled(true);
    run(
        &p,
        &mut core,
        &tl,
        512,
        512,
        &[cc(100, 15, 7, 33)],
        &[],
        |_, _| {},
    );
    assert_eq!(master(&core), 33);
    run(
        &p,
        &mut core,
        &tl,
        512,
        512,
        &[cc(100, 6, 7, 44), cc(101, 0, 7, 45)],
        &[],
        |_, _| {},
    );
    assert_eq!(master(&core), 33, "not the remote channel");
    // the sections' channels (MIDI 1..4) are refused as the remote channel
    for c in 0..=3 {
        let e = p.set_remote_channel(c).unwrap_err();
        assert!(e.contains("section"), "{e}");
    }
    assert!(p.set_remote_channel(16).is_err());
    assert_eq!(p.remote_channel(), 15);
    p.set_remote_channel(4).unwrap();
    assert_eq!(p.remote_channel(), 4, "MIDI channel 5 is allowed");
    // channel 7 (0-based 6) takes over at once in the running instance
    p.set_remote_channel(6).unwrap();
    run(
        &p,
        &mut core,
        &tl,
        512,
        512,
        &[cc(100, 6, 7, 55)],
        &[],
        |_, _| {},
    );
    assert_eq!(master(&core), 55);
    run(
        &p,
        &mut core,
        &tl,
        512,
        512,
        &[cc(100, 0, 7, 66)],
        &[],
        |_, _| {},
    );
    assert_eq!(master(&core), 55);
    // the transport notes do nothing: Stop (note 70) does not stop the patterns
    run(
        &p,
        &mut core,
        &tl,
        1024,
        512,
        &[note_on(10, 6, 70)],
        &[],
        |_, _| {},
    );
    assert!(
        core.audio().current().playing(),
        "the transport is the host's"
    );
    // the channel is in the state; a saved device channel is refused on restore, the rest of
    // the state is kept
    p.edit(Command::StepField {
        device: Device::Tb303_1,
        pattern: 0,
        step: 2,
        field: 0,
        value: 11,
    })
    .unwrap();
    set(&p, "d1c3", 101);
    let saved = p.serialize_fields();
    let q = OrbParams::new();
    q.deserialize_fields(&saved);
    assert_eq!(q.remote_channel(), 6);
    assert!(q.remote_enabled());
    assert!(!OrbParams::new().remote_enabled());
    // a state without the two keys: off, channel 16
    let bare: BTreeMap<String, String> = saved
        .iter()
        .map(|(k, v)| {
            let v = v
                .replace(",\"remote_enabled\":true", "")
                .replace(",\"remote_channel\":6", "");
            (k.clone(), v)
        })
        .collect();
    assert!(
        !bare.values().any(|v| v.contains("remote_")),
        "keys removed"
    );
    let b = OrbParams::new();
    b.deserialize_fields(&bare);
    assert!(!b.remote_enabled());
    assert_eq!(b.remote_channel(), 15);
    assert!(b.main().last_error.is_none());
    // a device channel (MIDI 1, 3 and 4 too, the boundaries) resets to channel 16
    let mut r = None;
    for bad_channel in [0, 2, 3] {
        let bad: BTreeMap<String, String> = saved
            .iter()
            .map(|(k, v)| {
                let to = format!("\"remote_channel\":{bad_channel}");
                (k.clone(), v.replace("\"remote_channel\":6", &to))
            })
            .collect();
        assert_ne!(bad, saved);
        let x = OrbParams::new();
        // what the host does: the parameters, then the fields
        for (id, v) in p.ids().iter().zip(p.values()) {
            set(&x, id, v);
        }
        x.deserialize_fields(&bad);
        assert_eq!(x.remote_channel(), 15, "reset to channel 16");
        assert!(x.remote_enabled(), "the on/off is kept");
        assert!(x.main().last_error.as_deref().unwrap().contains("section"));
        r = Some(x);
    }
    let r = r.unwrap();
    {
        let (want, got) = (&p.main().doc, &r.main().doc);
        assert!(got.song.is_some(), "the song is kept");
        assert_eq!(got.song, want.song);
        assert_eq!(got.patterns, want.patterns, "the patterns are kept");
        assert_eq!(got.patterns[0][2 + 2 * 6], 11, "the edit is kept");
        assert_eq!(got.vintage, want.vintage);
    }
    assert_eq!(r.values(), p.values(), "the controls are kept");
    assert_eq!(r.param("d1c3").unwrap().value(), 101);
}

#[test]
fn the_snapshot_has_the_sync_led_and_midi_activity() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let tl = Timeline(vec![
        Seg {
            at: 0,
            playing: false,
            tempo: Some(120.0),
            beats: 0.0,
        },
        Seg {
            at: 4096,
            playing: true,
            tempo: Some(120.0),
            beats: 0.0,
        },
    ]);
    let snap = |p: &OrbParams| p.snapshot().unwrap();
    run(&p, &mut core, &tl, 4096, 512, &[], &[], |_, _| {});
    assert!(!snap(&p).host_sync, "the host is stopped: so is orb");
    assert_eq!(snap(&p).midi_in, 0);
    let midi = [note_on(4200, 0, 3), cc(4300, 0, 7, 9), note_on(4400, 9, 60)];
    run(&p, &mut core, &tl, 8192, 512, &midi, &[], |_, _| {});
    assert!(snap(&p).host_sync, "pattern mode, orb and the host play");
    assert_eq!(snap(&p).midi_in, 3, "every message counts");
    // note mode: no sync while the host plays
    p.set_mode(Mode::Note);
    let mut core2 = activate(&p, 44_100);
    run(&p, &mut core2, &tl, 12_288, 512, &[], &[], |_, _| {});
    assert!(!snap(&p).host_sync, "note mode");
    assert!(!core2.following());
}
