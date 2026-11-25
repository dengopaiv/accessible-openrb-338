//! task-214: a parameter change in the first host buffer after an activation. The activation's
//! song is put in place at the activation (`core::settle`, task-226), so a change the host makes
//! before or within the first call reaches the engine that plays, and is not lost to the
//! activation's load replacing the player's placeholder song at the first block.
//!
//! The reference is an instance activated with the new value. The output is not equal from the
//! start: the instance changed in its first call moves its 303 cutoff smoother (0x1012470C,
//! `docs/initial_investigation.md`) from the old value's fixed point, which it reached while
//! settling, to the new one, as any cutoff change does while playing. Once the smoother has
//! converged (about 21000 host frames at 44.1 and 48 kHz) the output is the reference's bit for
//! bit.

mod common;

use common::{Change, activate, host, peak, set, with_song};
use orb_engine::song::Device;
use orb_plugin::core::Core;
use orb_plugin::params::{self, Target};

/// The song: the 303-1 plays its pattern, so its cutoff is heard.
const SONG: &str = "303_accent0";
const ID: &str = "d1c3";
const CUTOFF: Target = Target::Ctrl(Device::Tb303_1, 3);
const BUF: usize = 512;
const FRAMES: usize = 64 * BUF;
/// The host frame from which the output equals the reference's: after the cutoff smoother has
/// converged (the last difference measured: frame 19016 at 44.1 kHz, 20767 at 48 kHz).
const CONVERGED: usize = 48 * BUF;

/// The engine's byte of the cutoff.
fn byte(core: &Core) -> u8 {
    params::engine_byte(core.audio().current(), CUTOFF).unwrap()
}

/// An instance of [`SONG`] with the cutoff at `v` at its activation, rendered for [`FRAMES`]
/// host frames with the `changes` (all in the first call); the output interleaved and the
/// engine's cutoff byte after the first call and at the end.
fn run(rate: u32, v: i32, changes: &[Change]) -> (Vec<f32>, u8, u8) {
    let p = with_song(SONG);
    set(&p, ID, v);
    let mut core = activate(&p, rate);
    let mut out = host(&p, &mut core, BUF, BUF, changes);
    let first = byte(&core);
    out.extend(host(&p, &mut core, FRAMES - BUF, BUF, &[]));
    (out, first, byte(&core))
}

#[test]
fn a_change_in_the_first_buffer_after_an_activation_is_played() {
    let p = with_song(SONG);
    let old = p.values()[p.ids().iter().position(|x| x == ID).unwrap()];
    let new = if old < 64 { old + 50 } else { old - 50 };
    let tail = |x: &[f32]| x[2 * CONVERGED..].to_vec();
    for rate in [44_100, 48_000] {
        let (reference, b, _) = run(rate, new, &[]);
        assert_eq!(b as i32, new, "{rate}: the reference plays the new cutoff");
        assert!(peak(&reference) > 0.1, "{rate}: the 303 sounds");
        let (unchanged, b, _) = run(rate, old, &[]);
        assert_eq!(b as i32, old);
        assert!(
            tail(&unchanged) != tail(&reference),
            "{rate}: the cutoff is heard"
        );
        // 0: the host sets the value before the first call; 10, 63: within the first 64 frames,
        // before the first engine block of the code before task-226
        for at in [0, 10, 63] {
            let changes = [Change {
                at,
                id: ID,
                value: new,
            }];
            let (out, first, last) = run(rate, old, &changes);
            assert_eq!(
                first as i32, new,
                "{rate} at {at}: the engine holds it after the call"
            );
            assert_eq!(last as i32, new, "{rate} at {at}: and keeps it");
            assert!(
                tail(&out) == tail(&reference),
                "{rate} at {at}: plays as if activated with it, once the cutoff has moved"
            );
        }
    }
}
