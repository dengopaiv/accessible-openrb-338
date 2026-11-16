//! task-197 AC #7: the plugin's process side allocates, frees and locks nothing.
//!
//! As orb-player's `no_alloc`: this binary's allocator counts what a thread does while it is
//! armed, and the test arms itself only around `Core::process`, the whole of what the plugin's
//! `process` does besides reading the parameters' atomics and draining the host's note events
//! (nice-plug's). Between the calls, unarmed, the main thread does what the editor and the host
//! do: parameter changes of every kind, the mode off and on, a song loaded over the playing one
//! (pending, then switched by the mode), a mod change, pattern edits, the Vintage switch and a
//! state restore. At 44.1 kHz, at 48 kHz (the resampler's table) and at 44101 Hz (its weights
//! computed per frame), with buffers of 1 to 4096 frames.
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
use orb_plugin::core::Core;
use orb_plugin::instance::OrbParams;

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

/// `frames` frames in calls of `buf`, armed; the events.
fn run(p: &OrbParams, core: &mut Core, frames: usize, buf: usize) -> usize {
    let (mut l, mut r) = (vec![0f32; buf], vec![0f32; buf]);
    let mut n = 0;
    let mut done = 0;
    while done < frames {
        let k = buf.min(frames - done);
        // what the plugin's `process` does: read the parameters, then the core
        n += armed(|| {
            let values = p.values();
            core.process(&mut l[..k], &mut r[..k], &values)
        });
        done += k;
    }
    n
}

fn session(rate: u32) {
    let p = with_song("full_mix");
    let mut core = activate(&p, rate);
    let mut events = run(&p, &mut core, 20_000, 512);
    for (k, buf) in [1, 63, 64, 4096, 333].into_iter().enumerate() {
        events += run(&p, &mut core, 5000, buf);
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
        events += run(&p, &mut core, 3000, buf);
    }
    // the mode off and on
    set(&p, "g1", 0);
    events += run(&p, &mut core, 5000, 256);
    set(&p, "g1", 1);
    events += run(&p, &mut core, 5000, 256);
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
    events += run(&p, &mut core, 5000, 100);
    p.set_mod(Some(test_mod())).unwrap();
    events += run(&p, &mut core, 5000, 100);
    for (id, v) in p.load_song(song_bytes("delay_automated")).unwrap() {
        set(&p, &id, v);
    }
    events += run(&p, &mut core, 10_000, 700);
    p.set_mod(None).unwrap();
    events += run(&p, &mut core, 5000, 100);
    // a state restore while running
    let saved = p.serialize_fields();
    p.deserialize_fields(&saved);
    events += run(&p, &mut core, 10_000, 4096);
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
