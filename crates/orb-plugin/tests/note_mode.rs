//! task-200: note mode. In note mode (task-217: a setting), host notes on channels 1..4 play the
//! devices (`docs/plugin.md` section 9), whether the host plays or not; in pattern mode they keep
//! selecting patterns; notes never change a parameter.

mod common;

use common::{activate, peak, with_song};
use orb_engine::tr808::sound as s808;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};

fn on(frame: u32, channel: u8, note: u8, velocity: u8) -> HostEvent {
    HostEvent {
        frame,
        message: Message::NoteOn {
            channel,
            note,
            velocity,
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

/// A new instance (note mode, the default).
fn instance() -> (OrbParams, Core) {
    let p = OrbParams::new();
    let core = activate(&p, 44_100);
    (p, core)
}

#[test]
fn a_303_note_sounds_for_its_length() {
    for transport in [
        HostTransport::default(),
        HostTransport {
            playing: true,
            tempo: Some(120.0),
            pos_beats: Some(0.0),
        },
    ] {
        let (p, mut core) = instance();
        let values = p.values();
        assert_eq!(peak(&call(&p, &mut core, 4096, &transport, &[])), 0.0);
        let held = call(&p, &mut core, 8192, &transport, &[on(0, 0, 36, 100)]);
        assert!(peak(&held) > 0.05, "{}", peak(&held));
        call(&p, &mut core, 512, &transport, &[off(0, 0, 36)]);
        call(&p, &mut core, 20_000, &transport, &[]);
        assert!(peak(&call(&p, &mut core, 1024, &transport, &[])) < 0.002);
        // a velocity-0 note-on is a note-off too
        call(&p, &mut core, 512, &transport, &[on(0, 1, 40, 127)]);
        call(&p, &mut core, 512, &transport, &[on(0, 1, 40, 0)]);
        call(&p, &mut core, 20_000, &transport, &[]);
        assert!(peak(&call(&p, &mut core, 1024, &transport, &[])) < 0.002);
        assert_eq!(p.values(), values, "notes change no parameter");
        assert!(!core.audio().last_snapshot().playing);
    }
}

#[test]
fn drum_notes_light_their_sounds() {
    let (p, mut core) = instance();
    let t = HostTransport::default();
    // the mixer's gains rise from 0 over the first blocks
    call(&p, &mut core, 4096, &t, &[]);
    // the 808 on channel 3: the conga of the LT/LC slot, whatever its switch says
    let out = call(
        &p,
        &mut core,
        256,
        &t,
        &[on(0, 2, 64, 127), on(0, 3, 51, 30)],
    );
    assert!(peak(&out) > 0.0);
    let s = core.audio().last_snapshot().sounding;
    assert_eq!((s[2].mask, s[2].ac), (1 << s808::LC, 1 << s808::LC));
    assert_eq!((s[3].mask, s[3].ac), (1 << orb_engine::tr909::sound::RC, 0));
    assert_eq!(p.values(), OrbParams::new().values());
}

#[test]
fn notes_below_32_play_in_note_mode_and_select_in_pattern_mode() {
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let t = HostTransport::default();
    call(&p, &mut core, 4096, &t, &[]);
    let before = core.audio().last_snapshot().live[1].as_slice()[1];
    // note mode: note 5 on 303-1's channel is a note (MIDI 5, 31 semitones below the C), not A6
    p.set_mode(Mode::Note);
    call(&p, &mut core, 4096, &t, &[on(10, 0, 5, 100)]);
    assert_eq!(core.audio().last_snapshot().live[1].as_slice()[1], before);
    assert_eq!(
        core.audio().current().frame.devices.channels[0].playing_pattern(),
        Some(before)
    );
    assert!(!core.audio().last_snapshot().playing);
    // pattern mode: it selects pattern 6 (the host is stopped: only the selection changes)
    p.set_mode(Mode::Latch);
    call(&p, &mut core, 4096, &t, &[on(10, 0, 5, 100)]);
    assert_eq!(core.audio().last_snapshot().live[1].as_slice()[1], 5);
    assert!(!core.audio().last_snapshot().playing);
}

#[test]
fn note_mode_plays_notes_whatever_the_host_transport_does() {
    // the user, 2026-11-16: the no-sound-while-stopped rule is pattern mode's only
    for t in [HostTransport::default(), common::PLAYING] {
        let p = with_song("full_mix");
        p.set_mode(Mode::Note);
        let mut core = activate(&p, 44_100);
        assert_eq!(peak(&call(&p, &mut core, 8192, &t, &[])), 0.0, "{t:?}");
        let out = call(&p, &mut core, 8192, &t, &[on(0, 2, 36, 100)]);
        assert!(peak(&out) > 0.01, "{t:?}: the 808's bass drum");
        assert!(!core.audio().last_snapshot().playing);
    }
}

#[test]
fn a_note_right_after_the_mode_turns_to_note_is_not_silenced() {
    // the mode turns to note mode while the patterns play and a drum note follows in the same
    // block: the engine is stopped before the note, so the stop's silence does not cut it
    let p = with_song("full_mix");
    let mut core = activate(&p, 44_100);
    let t = common::PLAYING;
    call(&p, &mut core, 4096, &t, &[]);
    assert!(core.audio().last_snapshot().playing);
    p.set_mode(Mode::Note);
    call(&p, &mut core, 70, &t, &[on(3, 2, 56, 100)]);
    call(&p, &mut core, 64, &t, &[]);
    let s = core.audio().last_snapshot();
    assert!(!s.playing);
    assert_eq!(s.sounding[2].mask & 1 << s808::CB, 1 << s808::CB);
}
