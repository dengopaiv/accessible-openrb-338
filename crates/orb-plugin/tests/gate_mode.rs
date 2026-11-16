//! task-218: Gate Pattern Mode. A section plays only while a note 0..31 on its channel is held:
//! the note-on starts its pattern and the note-off stops it at the event's block (no bar-line
//! wait), from the host-position step while the host plays and from step 1 while it is stopped;
//! the last note pressed wins; the four sections gate on their own; the Standard Mapping only
//! selects; the On/off switch and the parameters are never touched (`docs/plugin.md` section 11).

mod common;

use common::{activate, peak, set};
use orb_engine::seq::TEMPO_STEP;
use orb_engine::song::Device;
use orb_player::Command;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};

const RATE: u32 = 44_100;

fn on(frame: u32, channel: u8, note: u8) -> HostEvent {
    HostEvent {
        frame,
        message: Message::NoteOn {
            channel,
            note,
            velocity: 100,
        },
    }
}

fn off(frame: u32, channel: u8, note: u8) -> HostEvent {
    HostEvent {
        frame,
        message: Message::NoteOff {
            channel,
            note,
            velocity: 64,
        },
    }
}

/// One call of `n` frames with `events`; the output interleaved.
fn call(p: &OrbParams, core: &mut Core, n: usize, t: &HostTransport, ev: &[HostEvent]) -> Vec<f32> {
    let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
    core.process_with(&mut l, &mut r, &p.values(), t, ev);
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

/// A host that plays at `bpm` from beat `start`, its position following its frames.
struct Host {
    bpm: f64,
    beats: f64,
}

impl Host {
    fn new(bpm: f64, start: f64) -> Host {
        Host { bpm, beats: start }
    }

    fn transport(&self) -> HostTransport {
        HostTransport {
            playing: true,
            tempo: Some(self.bpm),
            pos_beats: Some(self.beats),
        }
    }

    /// One call of `n` frames; the host's position advances by them.
    fn call(&mut self, p: &OrbParams, core: &mut Core, n: usize, ev: &[HostEvent]) -> Vec<f32> {
        let out = call(p, core, n, &self.transport(), ev);
        self.beats += n as f64 * self.bpm / 60.0 / RATE as f64;
        out
    }

    /// The host's tick now.
    fn tick(&self) -> f64 {
        self.beats * 192.0
    }
}

/// A stopped host at 97.25 BPM (not a song's tempo).
const STOPPED: HostTransport = HostTransport {
    playing: false,
    tempo: Some(97.25),
    pos_beats: Some(3.0),
};

/// A 303 pattern of 16 steps, every step note `note` with no rest.
fn pattern_303(note: u8) -> Box<[u8]> {
    let mut p = vec![0u8; 98];
    p[1] = 16;
    for s in 0..16 {
        p[2 + 6 * s] = note;
    }
    p.into_boxed_slice()
}

/// A drum pattern of 16 steps, instrument 1 on every step.
fn pattern_drums() -> Box<[u8]> {
    let mut p = vec![0u8; 194];
    p[1] = 16;
    for s in 0..16 {
        p[2 + 12 * s + 1] = 1;
    }
    p.into_boxed_slice()
}

/// A new instance in Gate Pattern Mode whose four sections play on every step of every
/// pattern (the 303s each pattern on its own note), activated at 44.1 kHz.
fn instance() -> (OrbParams, Core) {
    let p = OrbParams::new();
    p.set_mode(Mode::Gate);
    // the edits reach the engine of the activation, at its first block
    let core = activate(&p, RATE);
    for pattern in 0..32 {
        for (device, bytes) in [
            (Device::Tb303_1, pattern_303(pattern % 12)),
            (Device::Tb303_2, pattern_303(pattern % 12)),
            (Device::Tr808, pattern_drums()),
            (Device::Tr909, pattern_drums()),
        ] {
            p.edit(Command::Pattern {
                device,
                pattern,
                bytes,
            })
            .unwrap();
        }
    }
    // the blocks that apply them, with the main thread taking back the garbage, as the editor
    // does every frame
    let mut core = core;
    for _ in 0..16 {
        call(&p, &mut core, 64, &HostTransport::default(), &[]);
        p.collect_garbage();
    }
    // the gates the first blocks could not send (the garbage queue full of the edits) were
    // sent again
    assert!(core.dropped_events() > 0);
    assert_eq!(closed(&core), [true; 4]);
    (p, core)
}

fn closed(core: &Core) -> [bool; 4] {
    core.audio().last_snapshot().gate_closed
}

/// The pattern section `row` plays.
fn playing(core: &Core, row: usize) -> Option<u8> {
    core.audio().current().frame.devices.channels[row].playing_pattern()
}

/// The pattern section `row` shows (its pattern controller, which the panel follows).
fn shown(core: &Core, row: usize) -> u8 {
    core.audio().last_snapshot().live[row + 1].as_slice()[1]
}

/// The last step section `row` played (the 303's live byte 9, the 808's 29, the 909's 30).
fn last_step(core: &Core, row: usize) -> u8 {
    let at = [9, 9, 29, 30][row];
    core.audio().last_snapshot().live[row + 1].as_slice()[at]
}

/// The first and the last interleaved frame index of `x` that is not silent.
fn sounding_range(x: &[f32]) -> Option<(usize, usize)> {
    let first = x.iter().position(|&v| v != 0.0)?;
    let last = x.iter().rposition(|&v| v != 0.0)?;
    Some((first / 2, last / 2))
}

#[test]
fn a_note_starts_and_its_note_off_stops_the_section_at_its_block() {
    for buf in [64, 1000, 8192] {
        let (p, mut core) = instance();
        let mut host = Host::new(120.0, 0.0);
        // no note held: silence, whatever the patterns
        let mut out = Vec::new();
        for _ in 0..(16_384 / buf) {
            out.extend(host.call(&p, &mut core, buf, &[]));
        }
        assert_eq!(peak(&out), 0.0, "buf {buf}: no gate open");
        assert_eq!(closed(&core), [true; 4]);
        assert!(
            core.audio().last_snapshot().playing,
            "the song follows the host"
        );
        // a note-on at host frame 16384 + 1000: the section sounds from the first block that
        // starts at or after the event's frame (16384 + 1024, heard 64 frames later; the
        // plugin's 64-frame blocks, `docs/plugin.md` section 4)
        let mut out = Vec::new();
        let ev = [on(1000, 0, 0)];
        let mut done = 0;
        while done < 8192 {
            let k = buf.min(8192 - done);
            let e: Vec<HostEvent> = ev
                .iter()
                .filter(|e| (done..done + k).contains(&(e.frame as usize)))
                .map(|e| HostEvent {
                    frame: e.frame - done as u32,
                    ..*e
                })
                .collect();
            out.extend(host.call(&p, &mut core, k, &e));
            done += k;
        }
        let (first, _) = sounding_range(&out).expect("the section sounds");
        assert!(
            (1088..1152).contains(&first),
            "buf {buf}: the first sound at {first}"
        );
        assert_eq!(closed(&core), [false, true, true, true]);
        // the note-off at 1000 of the next 8192 frames: silent from the next block on
        let mut out = Vec::new();
        let ev = [off(1000, 0, 0)];
        let mut done = 0;
        while done < 8192 {
            let k = buf.min(8192 - done);
            let e: Vec<HostEvent> = ev
                .iter()
                .filter(|e| (done..done + k).contains(&(e.frame as usize)))
                .map(|e| HostEvent {
                    frame: e.frame - done as u32,
                    ..*e
                })
                .collect();
            out.extend(host.call(&p, &mut core, k, &e));
            done += k;
        }
        let (_, last) = sounding_range(&out).expect("it sounded until the note-off");
        // the first block at or after the event's frame fades the section out (task-222), then
        // silence
        assert!(
            (1088..1152).contains(&last),
            "buf {buf}: the last sound at {last}"
        );
        assert!(out[2 * 1152..].iter().all(|&v| v == 0.0));
        assert_eq!(closed(&core), [true; 4]);
        assert!(
            core.audio().last_snapshot().playing,
            "the song still follows"
        );
    }
}

#[test]
fn the_step_follows_the_host_position_its_loops_and_jumps() {
    let (p, mut core) = instance();
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 33_075, &[]);
    // beat 1.5: a note opens 303 #2 mid-pattern, in step with the song
    let out = host.call(&p, &mut core, 512, &[on(0, 1, 4)]);
    assert!(peak(&out) > 0.0);
    let check = |core: &Core, host: &Host| {
        let s = core.audio().last_snapshot();
        assert!(s.host_sync && core.following());
        // the engine's position is the host's (one block behind the call's end at most)
        assert!(
            (s.tick as f64 - host.tick()).abs() < 2.0,
            "tick {} for the host's {}",
            s.tick,
            host.tick()
        );
        assert_eq!(
            last_step(core, 1) as i32,
            s.tick / 48 % 16,
            "tick {}",
            s.tick
        );
        assert_eq!(playing(core, 1), Some(4));
    };
    check(&core, &host);
    for _ in 0..40 {
        host.call(&p, &mut core, 512, &[]);
    }
    check(&core, &host);
    // a loop back to beat 0.25 and a seek forward to beat 37.7: the section plays on there
    for to in [0.25, 37.7] {
        host.beats = to;
        let out = host.call(&p, &mut core, 512, &[]);
        assert!(peak(&out) > 0.0, "beat {to}: the gate stays open");
        host.call(&p, &mut core, 512, &[]);
        check(&core, &host);
    }
}

#[test]
fn with_the_host_stopped_a_note_plays_from_step_1_at_the_hosts_tempo() {
    let no_tempo = HostTransport {
        tempo: None,
        ..STOPPED
    };
    for t in [STOPPED, no_tempo] {
        let (p, mut core) = instance();
        let song_tempo = p.view(&p.values()).0.tempo;
        assert_ne!(song_tempo, 97_250);
        assert_eq!(peak(&call(&p, &mut core, 8192, &t, &[])), 0.0);
        assert!(!core.audio().last_snapshot().playing);
        // the note-on in the last frame before a block: the next block starts at step 1 (it
        // plays from the call after the next)
        call(&p, &mut core, 64, &t, &[on(63, 2, 9)]);
        call(&p, &mut core, 128, &t, &[]);
        let s = core.audio().last_snapshot();
        assert!(s.playing && !s.host_sync);
        assert_eq!(s.tick, 0, "{t:?}");
        assert_eq!(last_step(&core, 2), 0);
        assert_eq!(playing(&core, 2), Some(9));
        // free-running at the host's tempo, the song's when it reports none
        let tempo = t.tempo.map_or(song_tempo, |b| (b * 1000.0).round() as u32);
        let out = call(&p, &mut core, 44_100, &t, &[]);
        assert!(peak(&out) > 0.01, "{t:?}: the 808 plays");
        let s = core.audio().last_snapshot();
        assert_eq!(s.tempo, tempo);
        let ticks = 44_100.0 * tempo as f64 * TEMPO_STEP / 64.0;
        assert!(
            (s.tick as f64 - ticks).abs() < 2.0,
            "{t:?}: tick {} for {ticks}",
            s.tick
        );
        // the note-off stops the section, and the song with it
        let out = call(&p, &mut core, 4096, &t, &[off(0, 2, 9)]);
        assert!(out[2 * 192..].iter().all(|&v| v == 0.0));
        assert!(!core.audio().last_snapshot().playing);
        // the next note starts at step 1 again
        call(&p, &mut core, 128, &t, &[on(63, 2, 9)]);
        // its block is the first at or after the note, rendered up to 127 frames later
        call(&p, &mut core, 64, &t, &[]);
        let s = core.audio().last_snapshot();
        assert!(s.playing && s.tick < 2, "tick {}", s.tick);
    }
}

#[test]
fn the_last_note_pressed_wins() {
    let (p, mut core) = instance();
    let mut host = Host::new(133.0, 2.0);
    host.call(&p, &mut core, 4096, &[]);
    let mut step = |ev: &[HostEvent], want: Option<u8>| {
        let out = host.call(&p, &mut core, 2048, ev);
        let row = 0;
        match want {
            Some(n) => {
                assert_eq!(playing(&core, row), Some(n), "{ev:?}");
                assert_eq!(shown(&core, row), n, "{ev:?}: the panel shows it");
                assert!(!closed(&core)[row]);
                assert!(peak(&out[2 * 128..]) > 0.0);
            }
            None => {
                assert!(closed(&core)[row], "{ev:?}");
                // an event in the frames 1..63 reaches the block of frame 64, which fades out
                assert!(out[2 * 192..].iter().all(|&v| v == 0.0), "{ev:?}");
            }
        }
    };
    step(&[on(0, 0, 2)], Some(2));
    step(&[on(10, 0, 17)], Some(17));
    // releasing the last note falls back to the one still held
    step(&[off(10, 0, 17)], Some(2));
    step(&[on(10, 0, 30)], Some(30));
    // releasing an older note changes nothing
    step(&[off(10, 0, 2)], Some(30));
    // a velocity-0 note-on is a note-off
    step(
        &[HostEvent {
            frame: 3,
            message: Message::NoteOn {
                channel: 0,
                note: 30,
                velocity: 0,
            },
        }],
        None,
    );
    // three notes, released newest first, then a note above 31 (nothing)
    step(&[on(0, 0, 1), on(1, 0, 5), on(2, 0, 6)], Some(6));
    step(&[off(0, 0, 6)], Some(5));
    step(&[on(0, 0, 40), off(1, 0, 5)], Some(1));
    step(&[off(0, 0, 1)], None);
}

#[test]
fn the_sections_gate_on_their_own_channels() {
    let (p, mut core) = instance();
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 4096, &[]);
    host.call(&p, &mut core, 512, &[on(0, 0, 3), on(0, 2, 5), on(0, 3, 7)]);
    assert_eq!(closed(&core), [false, true, false, false]);
    host.call(&p, &mut core, 512, &[off(0, 2, 5)]);
    assert_eq!(closed(&core), [false, true, true, false]);
    // a closed section does not step; the open ones do
    let before = [
        last_step(&core, 0),
        last_step(&core, 2),
        last_step(&core, 3),
    ];
    host.call(&p, &mut core, 22_050, &[]);
    let after = [
        last_step(&core, 0),
        last_step(&core, 2),
        last_step(&core, 3),
    ];
    assert_ne!(before[0], after[0]);
    assert_eq!(before[1], after[1]);
    assert_ne!(before[2], after[2]);
    // the panel's step lights: none for a closed section (the snapshot says which)
    host.call(
        &p,
        &mut core,
        512,
        &[off(0, 0, 3), off(0, 3, 7), on(0, 1, 11)],
    );
    assert_eq!(closed(&core), [true, false, true, true]);
    assert_eq!(playing(&core, 1), Some(11));
}

#[test]
fn remote_pattern_notes_select_but_do_not_open_the_gate() {
    let (p, mut core) = instance();
    p.set_remote_enabled(true);
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 4096, &[]);
    // the Standard Mapping on channel 16: note 19 selects 303 #1's pattern 3 (A3)
    let remote = HostEvent {
        frame: 0,
        message: Message::NoteOn {
            channel: 15,
            note: 19,
            velocity: 100,
        },
    };
    let out = host.call(&p, &mut core, 8192, &[remote]);
    assert_eq!(shown(&core, 0), 2, "it selects");
    assert_eq!(closed(&core), [true; 4], "it opens no gate");
    assert_eq!(peak(&out), 0.0);
    // with a gate open on pattern 6 a remote selection shows, but the held note's pattern
    // plays on, also past the bar lines
    host.call(&p, &mut core, 512, &[on(0, 0, 6)]);
    assert_eq!((playing(&core, 0), shown(&core, 0)), (Some(6), 6));
    host.call(&p, &mut core, 512, &[remote]);
    host.call(&p, &mut core, 3 * 88_200, &[]);
    assert_eq!((playing(&core, 0), shown(&core, 0)), (Some(6), 2));
}

#[test]
fn the_gate_never_touches_the_on_off_switch_or_a_parameter() {
    let (p, mut core) = instance();
    let values = p.values();
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 4096, &[]);
    let enabled = |core: &Core| core.audio().last_snapshot().live[1].as_slice()[0];
    assert_eq!(enabled(&core), 1);
    host.call(&p, &mut core, 4096, &[on(0, 0, 5)]);
    host.call(&p, &mut core, 4096, &[off(0, 0, 5)]);
    assert_eq!(p.values(), values, "notes change no parameter");
    assert_eq!(enabled(&core), 1, "the panel's On button stays on");
    // switched off, the section stays silent with its gate open
    set(&p, "d1c0", 0);
    host.call(&p, &mut core, 4096, &[]);
    let out = host.call(&p, &mut core, 88_200, &[on(0, 0, 5)]);
    assert_eq!(peak(&out), 0.0, "off and open: silent");
    assert!(!closed(&core)[0]);
    assert_eq!(enabled(&core), 0);
    // on again: it sounds from the next bar line, the switch's own latch
    set(&p, "d1c0", 1);
    let out = host.call(&p, &mut core, 88_200, &[]);
    assert!(peak(&out) > 0.0, "on and open: it sounds");
    assert_eq!(enabled(&core), 1);
}

#[test]
fn a_held_note_shows_its_pattern_and_the_panel_keeps_the_last_one() {
    let (p, mut core) = instance();
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 4096, &[]);
    assert_eq!(p.param("d4c1").unwrap().value(), 0);
    host.call(&p, &mut core, 4096, &[on(0, 3, 13)]);
    assert_eq!((shown(&core, 3), playing(&core, 3)), (13, Some(13)));
    host.call(&p, &mut core, 4096, &[off(0, 3, 13)]);
    assert_eq!(
        shown(&core, 3),
        13,
        "the panel keeps the last gated pattern"
    );
    assert_eq!(
        p.param("d4c1").unwrap().value(),
        0,
        "the parameter is the host's"
    );
    // the pattern parameter changes what the panel shows, but only a gate note plays
    host.call(&p, &mut core, 4096, &[on(0, 3, 13)]);
    set(&p, "d4c1", 21);
    host.call(&p, &mut core, 3 * 88_200, &[]);
    assert_eq!((shown(&core, 3), playing(&core, 3)), (21, Some(13)));
    host.call(&p, &mut core, 4096, &[on(0, 3, 22)]);
    assert_eq!((shown(&core, 3), playing(&core, 3)), (22, Some(22)));
    assert_eq!(p.param("d4c1").unwrap().value(), 21);
}

#[test]
fn leaving_gate_mode_opens_the_sections_and_a_new_song_is_gated() {
    let (p, mut core) = instance();
    let mut host = Host::new(120.0, 0.0);
    host.call(&p, &mut core, 4096, &[on(0, 0, 3)]);
    // Latch: every section plays again, the held notes forgotten
    p.set_mode(Mode::Latch);
    let out = host.call(&p, &mut core, 8192, &[]);
    assert_eq!(closed(&core), [false; 4]);
    assert!(peak(&out) > 0.0);
    // Gate again: no note is held
    p.set_mode(Mode::Gate);
    host.call(&p, &mut core, 512, &[]);
    assert_eq!(closed(&core), [true; 4]);
    let out = host.call(&p, &mut core, 8192, &[off(0, 0, 3)]);
    assert_eq!(peak(&out), 0.0);
    // the preview plays every section; leaving it closes the gates again
    p.set_preview(true);
    host.call(&p, &mut core, 512, &[]);
    assert_eq!(closed(&core), [false; 4]);
    p.set_preview(false);
    host.call(&p, &mut core, 512, &[]);
    assert_eq!(closed(&core), [true; 4]);
    // File > New while gating: the new song's gates are closed too
    host.call(&p, &mut core, 512, &[on(0, 2, 1)]);
    for (id, v) in p.new_song().unwrap() {
        set(&p, &id, v);
    }
    host.call(&p, &mut core, 4096, &[]);
    assert_eq!(
        closed(&core),
        [true, true, false, true],
        "the held note stays held"
    );
    host.call(&p, &mut core, 512, &[off(0, 2, 1)]);
    assert_eq!(closed(&core), [true; 4]);
}

/// The stop fades out over the first block at or after the note-off (task-222): over 40
/// note-off positions the step from the last sample to silence stays below 0.002 of full scale,
/// and the section is silent after that block. Without the fade (the first build's hard
/// cut) it reached 0.036 (the 303s, peak 0.125), 0.047 (the 808, peak 0.077) and 0.037 (the
/// 909, peak 0.069) at the default levels; `--nocapture` prints the values.
#[test]
fn the_stop_fades_out_over_one_block() {
    for ch in 0..4u8 {
        let mut worst = 0f32;
        let mut pk = 0f32;
        for k in 0..40 {
            let (p, mut core) = instance();
            let mut host = Host::new(120.0, 0.0);
            host.call(&p, &mut core, 4096, &[on(0, ch, 0)]);
            let a = host.call(&p, &mut core, 1000 + 997 * k, &[]);
            pk = pk.max(peak(&a));
            let out = host.call(&p, &mut core, 4096, &[off(0, ch, 0)]);
            if let Some((_, last)) = sounding_range(&out) {
                // the fade is the first block at or after the note-off's frame, 64 to 127 frames
                // into the call with the latency
                assert!(
                    last < 192,
                    "channel {ch}: silent after the fade block, {last}"
                );
                let j = out[2 * last].abs().max(out[2 * last + 1].abs());
                worst = worst.max(j);
            }
        }
        eprintln!("channel {ch}: peak {pk}, the largest step to silence {worst}");
        assert!(worst < 0.002, "channel {ch}: {worst}");
    }
}

#[test]
fn a_mid_step_note_starts_at_once() {
    // 60 BPM: a tick is 230 frames, a step 0.25 s; notes at 10 % and 80 % into step 6 (beat
    // 1.25..1.5), far from its boundaries, on the 808 (channel 3) and 303 #1 (channel 1)
    for frac in [0.1, 0.8] {
        let (p, mut core) = instance();
        // the host plays for a second first (its start triggers the step there, slot 3)
        let beat = 0.25 + 0.25 * frac;
        let mut host = Host::new(60.0, beat);
        host.call(&p, &mut core, RATE as usize, &[]);
        let out = host.call(&p, &mut core, 512, &[on(0, 2, 0), on(0, 0, 0)]);
        let (first, _) = sounding_range(&out).expect("it sounds");
        assert!(first < 128, "{frac}: at once, {first}");
        assert_eq!((last_step(&core, 0), last_step(&core, 2)), (5, 5), "{frac}");
        // the 303's note ends by the step's end: step 7 comes on time
        host.call(
            &p,
            &mut core,
            ((1.0 - frac) * 0.25 * RATE as f64) as usize + 256,
            &[],
        );
        assert_eq!((last_step(&core, 0), last_step(&core, 2)), (6, 6), "{frac}");
    }
}

#[test]
fn an_on_beat_clip_note_lands_on_its_step() {
    // a clip note exactly on beat 1.5 (step 7, tick 288) at 60 BPM, for engine blocks at any
    // offset from it: the engine's position at the event is up to a block (0.28 tick, 1.45 ms)
    // before the step, inside the 5 ms snap window, so step 7 plays, on time, and the 303's
    // step 7 is not swallowed by a late step 6 (whose note would last 24 ticks)
    for k in 0..16 {
        let (p, mut core) = instance();
        let n = RATE as usize + 4 * k;
        let mut host = Host::new(60.0, 1.5 - n as f64 / RATE as f64);
        host.call(&p, &mut core, n, &[]);
        let out = host.call(&p, &mut core, 512, &[on(0, 0, 0), on(0, 2, 0)]);
        let (first, _) = sounding_range(&out).expect("it sounds");
        assert!(first < 128, "offset {k}: {first}");
        assert_eq!(
            (last_step(&core, 0), last_step(&core, 2)),
            (6, 6),
            "offset {k}"
        );
        // ten ticks on, the 303 still plays step 7's note (no late step 6)
        host.call(&p, &mut core, 2300, &[]);
        assert_eq!(last_step(&core, 0), 6, "offset {k}");
    }
}

#[test]
fn with_the_host_stopped_a_second_gate_joins_in_step() {
    let (p, mut core) = instance();
    call(&p, &mut core, 64, &STOPPED, &[on(63, 2, 9)]);
    call(&p, &mut core, 30_000, &STOPPED, &[]);
    let before = core.audio().last_snapshot().tick;
    assert!(before > 200);
    // the second section opens at the running position: no locate to 0
    call(&p, &mut core, 128, &STOPPED, &[on(0, 0, 4)]);
    let s = core.audio().last_snapshot();
    assert!(
        s.playing && s.tick >= before,
        "tick {} after {before}",
        s.tick
    );
    assert_eq!(last_step(&core, 0) as i32, s.tick / 48 % 16);
    assert_eq!(last_step(&core, 0), last_step(&core, 2));
    // the first one's release leaves the second playing on
    call(&p, &mut core, 4096, &STOPPED, &[off(0, 2, 9)]);
    let s = core.audio().last_snapshot();
    assert!(s.playing && s.tick > before);
    assert_eq!(closed(&core), [false, true, true, true]);
}
