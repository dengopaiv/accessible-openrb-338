//! task-258: note mode's held 303 notes when the host loses or adds note-offs (the user's
//! report: a REAPER send removed while playing left the 303 sliding until the mode was
//! switched). A key is held once, so a lost note-off heals at the key's next note-off; an
//! all-notes-off or all-sound-off on the section's channel and the host's stop let go of every
//! held note; a stray note-off changes nothing (`docs/plugin.md` 9.1).

mod common;

use common::{activate, peak};
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::OrbParams;

fn ev(frame: u32, message: Message) -> HostEvent {
    HostEvent { frame, message }
}

fn on(frame: u32, channel: u8, note: u8) -> HostEvent {
    ev(
        frame,
        Message::NoteOn {
            channel,
            note,
            velocity: 100,
        },
    )
}

fn off(frame: u32, channel: u8, note: u8) -> HostEvent {
    ev(
        frame,
        Message::NoteOff {
            channel,
            note,
            velocity: 64,
        },
    )
}

fn cc(frame: u32, channel: u8, controller: u8) -> HostEvent {
    ev(
        frame,
        Message::ControlChange {
            channel,
            controller,
            value: 0,
        },
    )
}

const STOPPED: HostTransport = HostTransport {
    playing: false,
    tempo: None,
    pos_beats: None,
};

/// The host rolling at 120 BPM, at frame `f` of 44.1 kHz.
fn rolling(f: usize) -> HostTransport {
    HostTransport {
        playing: true,
        tempo: Some(120.0),
        pos_beats: Some(f as f64 * 2.0 / 44_100.0),
    }
}

/// A new instance (note mode, the default), past the mixer's rise from 0.
fn instance() -> (OrbParams, Core) {
    let p = OrbParams::new();
    let mut core = activate(&p, 44_100);
    let (mut l, mut r) = (vec![0f32; 4096], vec![0f32; 4096]);
    core.process_with(&mut l, &mut r, &p.values(), &STOPPED, &[]);
    (p, core)
}

/// One call of `n` frames with `events`, the host stopped; the output interleaved.
fn call(p: &OrbParams, core: &mut Core, n: usize, ev: &[HostEvent]) -> Vec<f32> {
    call_at(p, core, n, &STOPPED, ev)
}

fn call_at(
    p: &OrbParams,
    core: &mut Core,
    n: usize,
    t: &HostTransport,
    ev: &[HostEvent],
) -> Vec<f32> {
    let (mut l, mut r) = (vec![0f32; n], vec![0f32; n]);
    core.process_with(&mut l, &mut r, &p.values(), t, ev);
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

/// Whether the 303 still sounds after its last note-off: the release (about 0.4 s) is over
/// after 20000 frames, so a gate left open is the only sound left.
fn sounds_on(p: &OrbParams, core: &mut Core) -> bool {
    call(p, core, 20_000, &[]);
    peak(&call(p, core, 1024, &[])) > 0.002
}

#[test]
fn a_lost_note_off_keeps_the_303_sliding_until_the_key_is_let_go() {
    let (p, mut core) = instance();
    // 36's note-off never comes (a REAPER send removed during the note)
    call(&p, &mut core, 4096, &[on(0, 0, 36)]);
    // the next notes slide from it, and the last one's release slides back to it: stuck
    call(&p, &mut core, 4096, &[on(0, 0, 40)]);
    call(&p, &mut core, 4096, &[off(0, 0, 40), on(100, 0, 43)]);
    call(&p, &mut core, 4096, &[off(0, 0, 43)]);
    assert!(sounds_on(&p, &mut core), "the lost note-off holds the gate");
    // the key comes again (the looped clip) and is let go: nothing is held. Before task-258 a
    // key was held as often as it was pressed, so the lost note-off held it for good
    call(&p, &mut core, 4096, &[on(0, 0, 36)]);
    call(&p, &mut core, 4096, &[off(0, 0, 36)]);
    assert!(
        !sounds_on(&p, &mut core),
        "the key's next note-off lets it go"
    );
}

#[test]
fn all_notes_off_and_all_sound_off_let_the_held_notes_go() {
    for controller in [123, 120] {
        // with notes 36 and 40 held, the message on 303-1's channel is both note-offs
        let run = |release: &[HostEvent]| {
            let (p, mut core) = instance();
            let mut out = call(&p, &mut core, 4096, &[on(0, 0, 36), on(2000, 0, 40)]);
            out.extend(call(&p, &mut core, 4096, release));
            out.extend(call(&p, &mut core, 20_000, &[]));
            out
        };
        let by_cc = run(&[cc(300, 0, controller)]);
        let by_offs = run(&[off(300, 0, 36), off(300, 0, 40)]);
        assert!(peak(&by_cc) > 0.05);
        let first = by_cc
            .iter()
            .zip(&by_offs)
            .position(|(a, b)| a.to_bits() != b.to_bits());
        assert_eq!(first, None, "CC {controller}: as the note-offs");
        // the stuck state lets go too
        let (p, mut core) = instance();
        call(&p, &mut core, 4096, &[on(0, 0, 36), on(100, 0, 40)]);
        call(&p, &mut core, 4096, &[off(0, 0, 40)]);
        assert!(sounds_on(&p, &mut core));
        // on another section's channel it does not
        call(&p, &mut core, 64, &[cc(0, 1, controller)]);
        assert!(
            sounds_on(&p, &mut core),
            "CC {controller} on 303-2's channel"
        );
        call(&p, &mut core, 64, &[cc(0, 0, controller)]);
        assert!(!sounds_on(&p, &mut core), "CC {controller} lets go");
    }
}

#[test]
fn the_hosts_stop_lets_the_held_notes_go() {
    let (p, mut core) = instance();
    let mut f = 0;
    let mut roll = |core: &mut Core, n: usize, ev: &[HostEvent]| {
        let out = call_at(&p, core, n, &rolling(f), ev);
        f += n;
        out
    };
    roll(&mut core, 4096, &[on(0, 0, 36)]);
    roll(&mut core, 4096, &[on(0, 0, 40)]);
    roll(&mut core, 4096, &[off(0, 0, 40)]);
    for _ in 0..6 {
        roll(&mut core, 4096, &[]);
    }
    assert!(
        peak(&roll(&mut core, 1024, &[])) > 0.002,
        "held while it plays"
    );
    // the host stops: the held note is let go
    assert!(!sounds_on(&p, &mut core), "the host's stop lets go");
    // a note played with the host stopped is held until its note-off
    call(&p, &mut core, 4096, &[on(0, 0, 36)]);
    assert!(peak(&call(&p, &mut core, 20_000, &[])) > 0.05);
    call(&p, &mut core, 64, &[off(0, 0, 36)]);
    assert!(!sounds_on(&p, &mut core));
}

#[test]
fn a_stray_note_off_changes_nothing() {
    let run = |stray: bool| {
        let (p, mut core) = instance();
        let mut first = vec![on(500, 0, 36)];
        if stray {
            // a note-off of a key not held (a REAPER send added during a note), before and
            // after the note-on, and a second note-off of the key
            first.insert(0, off(0, 0, 40));
            first.push(off(800, 0, 40));
        }
        let mut out = call(&p, &mut core, 4096, &first);
        let mut second = vec![off(100, 0, 36)];
        if stray {
            second.push(off(200, 0, 36));
        }
        out.extend(call(&p, &mut core, 4096, &second));
        out.extend(call(&p, &mut core, 4096, &[on(0, 0, 43), off(3000, 0, 43)]));
        out
    };
    let (a, b) = (run(false), run(true));
    assert!(peak(&a) > 0.05);
    let first = a
        .iter()
        .zip(&b)
        .position(|(x, y)| x.to_bits() != y.to_bits());
    assert_eq!(first, None);
}
