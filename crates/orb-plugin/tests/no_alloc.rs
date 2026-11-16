//! task-197 AC #7: the plugin's process side allocates, frees and locks nothing.
//!
//! As orb-player's `no_alloc`: this binary's allocator counts what a thread does while it is
//! armed, and the test arms itself only around `Core::process_with`, the whole of what the
//! plugin's `process` does besides reading the parameters' atomics and the transport and
//! draining the host's note events into a buffer reserved at construction (nice-plug's). The
//! simulated host plays, stops, jumps and changes its tempo, and sends pattern notes, note-offs
//! and Standard Mapping control changes (task-198). Between the calls, unarmed, the main thread
//! does what the editor and the host do: parameter changes of every kind, the mode off and on, a
//! song loaded over the playing one (pending, then switched by the mode), a mod change, pattern
//! edits, the Vintage switch and a state restore. At 44.1 kHz, at 48 kHz (the resampler's table)
//! and at 44101 Hz (its weights computed per frame), with buffers of 1 to 4096 frames.
//!
//! Locks: the process side's sources (`core.rs`, `resample.rs`, `params.rs`) name no lock type,
//! which `the_process_side_holds_no_lock` checks textually; orb-player's callback code is
//! checked by its own `no_alloc` test. The main-thread mutex (`instance.rs`) is never taken in
//! `process`.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{activate, set, song_bytes, test_mod, with_song};
use nice_plug::prelude::Params;
use orb_engine::song::Device;
use orb_player::Command;
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport};
use orb_plugin::instance::{Mode, OrbParams};

struct Counting;

static EVENTS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static ARMED: Cell<bool> = const { Cell::new(false) };
}

fn note() {
    if ARMED.try_with(|a| a.get()).unwrap_or(false) {
        EVENTS.fetch_add(1, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc(l) }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        note();
        unsafe { System.alloc_zeroed(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        note();
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        note();
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static A: Counting = Counting;

fn armed(f: impl FnOnce()) -> usize {
    let before = EVENTS.load(Ordering::Relaxed);
    ARMED.with(|a| a.set(true));
    f();
    ARMED.with(|a| a.set(false));
    EVENTS.load(Ordering::Relaxed) - before
}

#[test]
fn the_counter_counts() {
    assert_eq!(armed(|| drop(std::hint::black_box(vec![1u8; 10]))), 2);
}

/// The simulated host (task-198): its frames and calls so far. It plays from frame 20000 on,
/// jumps back four beats every 30000 frames, stops between frames 60000 and 70000, changes its
/// tempo every 25000 frames, and sends in every call a note 0..31 on one of channels 1..5 (a
/// pattern in Latch Pattern Mode, a gate in Gate Pattern Mode, a note in note mode), a note 30..89
/// on a section's channel (a 303 note or a drum hit in note mode, task-200), a note-off, and a
/// master-level control change.
#[derive(Default)]
struct Host {
    frames: u64,
    calls: u64,
}

impl Host {
    fn transport(&self, rate: u32) -> HostTransport {
        let f = self.frames;
        let tempo = 100.0 + (f / 25_000 % 4) as f64 * 17.5;
        let playing = f >= 20_000 && !(60_000..70_000).contains(&f);
        let beats = (f as f64 * tempo / 60.0 / rate as f64) - 4.0 * (f / 30_000) as f64;
        HostTransport {
            playing,
            tempo: Some(tempo),
            pos_beats: Some(beats),
        }
    }

    fn events(&self, k: usize) -> [HostEvent; 4] {
        let c = self.calls;
        let last = (k - 1) as u32;
        [
            HostEvent {
                frame: 0,
                message: Message::NoteOn {
                    channel: (c % 5) as u8,
                    note: (c % 32) as u8,
                    velocity: 100,
                },
            },
            // in note mode (task-200) a 303 note or a drum hit, overlapping the one before
            HostEvent {
                frame: last / 3,
                message: Message::NoteOn {
                    channel: (c % 4) as u8,
                    note: 30 + (c * 7 % 60) as u8,
                    velocity: 1 + (c % 127) as u8,
                },
            },
            HostEvent {
                frame: last / 2,
                message: Message::NoteOff {
                    channel: (c % 5) as u8,
                    note: (c % 32) as u8,
                    velocity: 0,
                },
            },
            HostEvent {
                frame: last,
                message: Message::ControlChange {
                    channel: 0,
                    controller: 7,
                    value: (c % 128) as u8,
                },
            },
        ]
    }
}

/// `frames` frames in calls of `buf`, armed; the events.
fn run(p: &OrbParams, core: &mut Core, h: &mut Host, frames: usize, buf: usize) -> usize {
    let (mut l, mut r) = (vec![0f32; buf], vec![0f32; buf]);
    let mut n = 0;
    let mut done = 0;
    while done < frames {
        let k = buf.min(frames - done);
        let t = h.transport(core.rate());
        let ev = h.events(k);
        // what the plugin's `process` does: read the parameters, then the core
        n += armed(|| {
            let values = p.values();
            core.process_with(&mut l[..k], &mut r[..k], &values, &t, &ev)
        });
        done += k;
        h.frames += k as u64;
        h.calls += 1;
    }
    n
}

fn session(rate: u32) {
    let p = with_song("full_mix");
    let mut core = activate(&p, rate);
    let mut h = Host::default();
    let mut events = run(&p, &mut core, &mut h, 20_000, 512);
    for (k, buf) in [1, 63, 64, 4096, 333].into_iter().enumerate() {
        events += run(&p, &mut core, &mut h, 5000, buf);
        // a change of every kind of target
        set(&p, "d1c3", 30 + k as i32);
        set(&p, "d3c11", (k % 2) as i32);
        set(&p, "d4c1", k as i32);
        set(&p, "d0c0", 90 - k as i32);
        set(&p, "d0c1", (k % 6) as i32);
        set(&p, "d0c2", (k % 5) as i32);
        set(&p, "d0c11", (k % 2) as i32);
        set(&p, "d0c12", 50 + k as i32);
        set(&p, "g20", 10 * k as i32);
        set(&p, "d5c1", 1 + k as i32);
        set(&p, "d7c6", (k % 2) as i32);
        events += run(&p, &mut core, &mut h, 3000, buf);
    }
    // Gate Pattern Mode (task-218) from frame 60000: the gates open, switch and close with the
    // host stopped (until frame 70000), then playing and jumping
    assert_eq!(h.frames, 60_000);
    p.set_mode(Mode::Gate);
    events += run(&p, &mut core, &mut h, 40_000, 256);
    // the other modes
    p.set_mode(Mode::Note);
    events += run(&p, &mut core, &mut h, 5000, 256);
    p.set_mode(Mode::Latch);
    events += run(&p, &mut core, &mut h, 5000, 256);
    // the preview (task-217): entered, changed, host changes ignored, left, entered again
    p.set_preview(true);
    events += run(&p, &mut core, &mut h, 5000, 256);
    p.set_preview_value(5, 40).unwrap();
    p.set_preview_value(20, 3).unwrap();
    set(&p, "d1c3", 77);
    events += run(&p, &mut core, &mut h, 5000, 333);
    p.set_preview(false);
    events += run(&p, &mut core, &mut h, 5000, 256);
    p.set_preview(true);
    events += run(&p, &mut core, &mut h, 5000, 64);
    p.set_preview(false);
    // the editor's calls: pattern edits, Vintage, a mod, a song over the playing one
    p.edit(Command::StepField {
        device: Device::Tb303_1,
        pattern: 0,
        step: 1,
        field: 0,
        value: 5,
    })
    .unwrap();
    p.edit(Command::Pattern {
        device: Device::Tr808,
        pattern: 0,
        bytes: vec![1u8; 194].into_boxed_slice(),
    })
    .unwrap();
    p.set_vintage(true).unwrap();
    events += run(&p, &mut core, &mut h, 5000, 100);
    p.set_mod(Some(test_mod())).unwrap();
    events += run(&p, &mut core, &mut h, 5000, 100);
    for (id, v) in p.load_song(song_bytes("delay_automated")).unwrap() {
        set(&p, &id, v);
    }
    events += run(&p, &mut core, &mut h, 10_000, 700);
    p.set_mod(None).unwrap();
    events += run(&p, &mut core, &mut h, 5000, 100);
    // a state restore while running
    let saved = p.serialize_fields();
    p.deserialize_fields(&saved);
    events += run(&p, &mut core, &mut h, 10_000, 4096);
    assert_eq!(core.dropped_events(), 0);
    assert_eq!(
        events, 0,
        "process allocated or freed {events} times at {rate} Hz"
    );
}

#[test]
fn process_does_not_allocate_at_44100() {
    session(44_100);
}

#[test]
fn process_does_not_allocate_when_resampling() {
    session(48_000);
    session(44_101);
}

#[test]
fn the_process_side_holds_no_lock() {
    let src = common::root().join("crates/orb-plugin/src");
    let mut bad = Vec::new();
    for f in ["core.rs", "resample.rs", "params.rs"] {
        let text = std::fs::read_to_string(src.join(f)).unwrap();
        for w in [
            "Mutex",
            "RwLock",
            "Condvar",
            "Barrier",
            "Once",
            "mpsc",
            "parking_lot",
        ] {
            if text.contains(w) {
                bad.push(format!("{f}: {w}"));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}
