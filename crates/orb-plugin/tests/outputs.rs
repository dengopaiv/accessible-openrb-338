//! task-256: the per-device outputs 303-1, 303-2, 808 and 909, tapped where ReWire taps them
//! (task-259): after the channel's inserts, before its level, pan and on switch, mono on both
//! sides.
//!
//! - AC 3: the main Mix output is bit-identical whether the host connects none, some or all of the
//!   per-device buses, at 44.1 kHz and at a resampled rate, for any buffer size.
//! - AC 2: a device playing alone with an insert on its channel (the distortion on a 303, the
//!   compressor on the 808, the PCF on the 909) reproduces the main mix from its own bus: the Mix
//!   is the bus times the channel's gain and the master gain, as int16, and the other buses are
//!   silent; the channel's level, pan and on switch change the Mix but not the bus.
//!
//! task-300: the 22 drum outputs after them, and Options > Output Routing (the engine's exact
//! semantics are `orb-engine`'s `frame::tests::routing`):
//! - with every box checked, the Mix is bit-identical with none, some or all 26 buses connected,
//!   and the drum outputs sound;
//! - a device out of Mix-L/R leaves the Mix (its delay send stays) and its own output as it was;
//! - a drum instrument out of its machine's bus changes the Mix and the machine's output, and its
//!   own output and the other drum outputs stay what they were.

mod common;

use common::{PLAYING, activate, peak, set, with_song};
use orb_engine::frame::to_i16;
use orb_engine::frame::{OutputRouting, TAP_808};
use orb_plugin::core::{AUX_OUTPUTS, AuxOut};
use orb_plugin::instance::OrbParams;

/// The host's output: the Mix and the 26 buses, each interleaved stereo.
struct Out {
    mix: Vec<f32>,
    buses: Vec<Vec<f32>>,
}

/// The first four buses (the devices') as `four` says, and the drum buses connected or not.
fn first(four: [bool; 4], drums: bool) -> [bool; AUX_OUTPUTS] {
    std::array::from_fn(|k| if k < 4 { four[k] } else { drums })
}

/// Plays `frames` host frames of `p` at `rate` in buffers of `buf` frames, with the buses of
/// `connected` given buffers (a `[bool; 4]` is the first four, the drum buses unconnected).
fn play(
    p: &OrbParams,
    rate: u32,
    frames: usize,
    buf: usize,
    connected: impl Into<Connected>,
) -> Out {
    let connected = connected.into().0;
    let mut core = activate(p, rate);
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mut buses = vec![vec![0f32; frames]; 2 * AUX_OUTPUTS];
    let values = p.values();
    let mut pos = 0;
    while pos < frames {
        let end = (pos + buf).min(frames);
        let mut it = buses.iter_mut();
        let mut aux: AuxOut = std::array::from_fn(|c| {
            let (bl, br) = (it.next().unwrap(), it.next().unwrap());
            connected[c].then_some([&mut bl[pos..end], &mut br[pos..end]])
        });
        core.process_aux(
            &mut l[pos..end],
            &mut r[pos..end],
            &mut aux,
            &values,
            &PLAYING,
            &[],
        );
        pos = end;
    }
    let inter = |a: &[f32], b: &[f32]| a.iter().zip(b).flat_map(|(&x, &y)| [x, y]).collect();
    Out {
        mix: inter(&l, &r),
        buses: buses.chunks(2).map(|p| inter(&p[0], &p[1])).collect(),
    }
}

/// Which buses get buffers.
struct Connected([bool; AUX_OUTPUTS]);

impl From<[bool; 4]> for Connected {
    fn from(four: [bool; 4]) -> Connected {
        Connected(first(four, false))
    }
}

impl From<[bool; AUX_OUTPUTS]> for Connected {
    fn from(all: [bool; AUX_OUTPUTS]) -> Connected {
        Connected(all)
    }
}

fn bits(x: &[f32]) -> Vec<u32> {
    x.iter().map(|v| v.to_bits()).collect()
}

#[test]
fn the_mix_does_not_depend_on_the_buses() {
    // every device, the delay, the PCF on the 909 and the compressor on the master
    let p = with_song("full_mix");
    for rate in [44_100, 48_000] {
        let frames = rate as usize * 2;
        let none = play(&p, rate, frames, 512, [false; 4]);
        assert!(peak(&none.mix) > 0.05, "the song sounds");
        for (buf, connected) in [
            (512, first([true; 4], false)),
            (100, first([true, false, true, false], false)),
            (1, first([false, false, false, true], false)),
            (4096, first([true; 4], false)),
            // task-300: the drum buses too
            (333, [true; AUX_OUTPUTS]),
            (64, std::array::from_fn(|k| k % 3 == 1)),
        ] {
            let with = play(&p, rate, frames, buf, connected);
            assert_eq!(
                bits(&none.mix),
                bits(&with.mix),
                "rate {rate}, buffer {buf}, {connected:?}"
            );
        }
        // the buses do not depend on the buffer size either, and an unconnected one is left
        // as the host had it
        let a = play(&p, rate, frames, 512, [true; 4]);
        let b = play(&p, rate, frames, 100, [true, false, true, false]);
        assert_eq!(bits(&a.buses[0]), bits(&b.buses[0]));
        assert_eq!(bits(&a.buses[2]), bits(&b.buses[2]));
        assert!(b.buses[1].iter().all(|&x| x == 0.0));
        for (c, bus) in a.buses[..4].iter().enumerate() {
            assert!(peak(bus) > 0.01, "rate {rate}: bus {c} sounds");
        }
        // the drum outputs: some of each machine sound
        let all = play(&p, rate, frames, 512, [true; AUX_OUTPUTS]);
        assert_eq!(bits(&all.buses[0]), bits(&a.buses[0]));
        for (m, range) in [("808", TAP_808..15), ("909", 15..AUX_OUTPUTS)] {
            let sounding = all.buses[range].iter().filter(|b| peak(b) > 0.01).count();
            assert!(sounding > 0, "rate {rate}: some {m} outputs sound");
        }
    }
}

#[test]
fn a_device_alone_with_an_insert_is_its_share_of_the_mix() {
    // the 303-1 through the distortion, the 909 through the PCF, the 808 through the compressor
    for (song, c) in [("dist_a100_s64", 0), ("pcf_on_909", 3), ("comp_ch_808", 2)] {
        let p = with_song(song);
        let frames = 44_100 * 3;
        let out = play(&p, 44_100, frames, 256, [true; 4]);
        // the master gain, steady: `f32(2 T[master] + 1e-6)` smoothed to its fixed point, and
        // the channel's gain (level 100, centered: left and right alike) likewise
        let core = activate(&p, 44_100);
        let state = &core.audio().current().frame.state;
        let m = state.master as f64;
        let g = state.channels[c].gain_l as f64;
        assert_eq!(state.channels[c].gain_l, state.channels[c].gain_r);
        let bus = &out.buses[c];
        assert!(peak(bus) > 0.05, "{song}: the device sounds");
        // mono: the same signal on both sides
        for f in bus.chunks(2) {
            assert_eq!(f[0].to_bits(), f[1].to_bits());
        }
        for (o, other) in out.buses[..4].iter().enumerate() {
            if o != c {
                // an idle 303 outputs about 3e-6 (its amp adds 1e-6 every block, 0x1002494C)
                assert!(
                    peak(other) < 1e-5,
                    "{song}: bus {o} is silent: {}",
                    peak(other)
                );
            }
        }
        // the Mix is the guard plus the bus times the channel's gain, times the master gain,
        // truncated to int16 (the send is off, so the delay returns at most 1e-6 of it)
        let mut worst = 0;
        for (j, (&x, &mix)) in bus.iter().zip(&out.mix).enumerate() {
            let pre = (1e-6f32 as f64 + x as f64 * g) as f32;
            let want = to_i16((pre as f64 * m) as f32) as i32;
            let got = (mix * 32768.0) as i32;
            let d = (got - want).abs();
            worst = worst.max(d);
            assert!(
                d <= 1,
                "{song}: sample {j}: Mix {got}, bus times master {want}"
            );
        }
        eprintln!("{song}: master {m}, worst difference {worst} LSB");
    }
}

#[test]
fn a_device_output_is_before_its_level_pan_and_on_switch() {
    // the 303-1 through the distortion; its mixer channel 0: on d0c5, level d0c6, pan d0c7
    let frames = 44_100 * 2;
    let base = play(&with_song("dist_a100_s64"), 44_100, frames, 512, [true; 4]);
    assert!(peak(&base.buses[0]) > 0.05);
    for changes in [
        &[("d0c6", 40)][..],
        &[("d0c7", 0)],
        &[("d0c6", 127), ("d0c7", 127)],
        &[("d0c5", 0)],
    ] {
        let p = with_song("dist_a100_s64");
        for &(id, v) in changes {
            set(&p, id, v);
        }
        let out = play(&p, 44_100, frames, 512, [true; 4]);
        assert_eq!(
            bits(&base.buses[0]),
            bits(&out.buses[0]),
            "{changes:?}: the device output does not change"
        );
        assert_ne!(bits(&base.mix), bits(&out.mix), "{changes:?}: the Mix does");
        if changes == [("d0c5", 0)] {
            // the channel off: out of the Mix, still on its own output
            assert!(peak(&out.mix) < 1e-3, "the Mix is silent");
        }
    }
}

/// task-300: a device out of Mix-L/R (the 303-1, through the distortion, alone in its song, the
/// send off) leaves the Mix silent, and its own output as it was.
#[test]
fn a_device_out_of_the_mix_keeps_its_own_output() {
    let frames = 44_100 * 2;
    let base = play(&with_song("dist_a100_s64"), 44_100, frames, 512, [true; 4]);
    assert!(peak(&base.mix) > 0.05);
    let p = with_song("dist_a100_s64");
    let mut r = OutputRouting::default();
    r.to_mix[0] = false;
    p.set_routing(r);
    let out = play(&p, 44_100, frames, 512, [true; 4]);
    assert_eq!(bits(&base.buses[0]), bits(&out.buses[0]));
    assert!(
        peak(&out.mix) < 1e-3,
        "the Mix is silent: {}",
        peak(&out.mix)
    );
    // back in: the Mix again
    p.set_routing(OutputRouting::default());
    let back = play(&p, 44_100, frames, 512, [true; 4]);
    assert_eq!(bits(&base.mix), bits(&back.mix));
}

/// task-300: each sounding drum instrument out of its machine's bus: its own output and the
/// other drum outputs stay as they were, the machine's output and the Mix change.
#[test]
fn a_drum_instrument_out_of_its_bus_keeps_its_own_output() {
    let frames = 44_100;
    let p = with_song("full_mix");
    let base = play(&p, 44_100, frames, 512, [true; AUX_OUTPUTS]);
    let mut tried = 0;
    for k in TAP_808..AUX_OUTPUTS {
        if peak(&base.buses[k]) < 0.01 {
            continue;
        }
        tried += 1;
        let mut bits_on = OutputRouting::default().to_bits();
        bits_on &= !(1 << k);
        let p = with_song("full_mix");
        p.set_routing(OutputRouting::from_bits(bits_on));
        let out = play(&p, 44_100, frames, 512, [true; AUX_OUTPUTS]);
        for j in TAP_808..AUX_OUTPUTS {
            assert_eq!(
                bits(&base.buses[j]),
                bits(&out.buses[j]),
                "bus {k} off: bus {j}"
            );
        }
        let machine = if k < 15 { 2 } else { 3 };
        assert_ne!(
            bits(&base.buses[machine]),
            bits(&out.buses[machine]),
            "bus {k} off"
        );
        assert_ne!(bits(&base.mix), bits(&out.mix), "bus {k} off");
        // the routing is in the state
        assert_eq!(p.routing(), OutputRouting::from_bits(bits_on));
    }
    assert!(tried >= 4, "{tried} drum outputs sound");
}
