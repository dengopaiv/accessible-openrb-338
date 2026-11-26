//! The plugin's outputs and Options > Output Routing (task-256, task-300; `docs/plugin.md`
//! section 13): the aux buses in the engine's tap order ([`orb_engine::frame::Taps`]), their
//! names, and the routing's keys in the state. Everything here follows the drum machines'
//! output tables ([`orb_engine::tr808::OUTPUTS`], [`orb_engine::tr909::OUTPUTS`]).

use std::collections::BTreeMap;

use orb_engine::drums::DrumOutput;
use orb_engine::frame::{OutputRouting, TAP_808, TAP_909, TAP_WIDTH};
use orb_engine::{tr808, tr909};

/// The aux buses: bus k (1-based in the hosts and the routing window: output k + 1) carries
/// tap k.
pub const AUX_OUTPUTS: usize = TAP_WIDTH;

/// The per-device outputs (task-256): 303-1, 303-2, 808, 909, the mixer's channels 0 to 3,
/// the first four aux buses.
pub const DEVICE_OUTPUTS: usize = 4;

/// The per-device outputs' names, the mixer's channels 0 to 3 in order.
pub const DEVICE_NAMES: [&str; DEVICE_OUTPUTS] = ["303-1", "303-2", "808", "909"];

/// The state's keys of the per-device outputs' Mix-L/R boxes; never changed once released.
pub const DEVICE_IDS: [&str; DEVICE_OUTPUTS] = ["mix.303-1", "mix.303-2", "mix.808", "mix.909"];

/// The aux buses' names: the four devices', then each drum output's ([`DrumOutput::bus`],
/// "808 BD" .. "909 RC"), in tap order.
pub const OUTPUT_NAMES: [&str; AUX_OUTPUTS] = table(DEVICE_NAMES, false);

/// The state's key of each box of the routing window, by bus: the devices' [`DEVICE_IDS`],
/// then each drum output's [`DrumOutput::id`] ("808.bd" .. "909.rc").
pub const ROUTING_IDS: [&str; AUX_OUTPUTS] = table(DEVICE_IDS, true);

const fn table(devices: [&'static str; DEVICE_OUTPUTS], ids: bool) -> [&'static str; AUX_OUTPUTS] {
    let mut out = [""; AUX_OUTPUTS];
    let mut k = 0;
    while k < DEVICE_OUTPUTS {
        out[k] = devices[k];
        k += 1;
    }
    const fn pick(o: &DrumOutput, ids: bool) -> &'static str {
        if ids { o.id } else { o.bus }
    }
    let mut i = 0;
    while i < tr808::OUTPUTS.len() {
        out[TAP_808 + i] = pick(&tr808::OUTPUTS[i], ids);
        i += 1;
    }
    let mut i = 0;
    while i < tr909::OUTPUTS.len() {
        out[TAP_909 + i] = pick(&tr909::OUTPUTS[i], ids);
        i += 1;
    }
    out
}

/// The routing as the state saves it: every box by its key, checked `true`.
pub fn to_saved(r: &OutputRouting) -> BTreeMap<String, bool> {
    let bits = r.to_bits();
    ROUTING_IDS
        .iter()
        .enumerate()
        .map(|(k, id)| (id.to_string(), bits & (1 << k) != 0))
        .collect()
}

/// The routing of a saved state: a key it lacks is checked (a state of the released build,
/// v2026.1118.0919, has none), a key it does not know is ignored.
pub fn from_saved(m: &BTreeMap<String, bool>) -> OutputRouting {
    let mut bits = 0u32;
    for (k, id) in ROUTING_IDS.iter().enumerate() {
        if m.get(*id).copied().unwrap_or(true) {
            bits |= 1 << k;
        }
    }
    OutputRouting::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_buses_are_the_devices_then_the_instruments() {
        assert_eq!(AUX_OUTPUTS, 26);
        assert_eq!(OUTPUT_NAMES[..4], DEVICE_NAMES);
        assert_eq!(
            OUTPUT_NAMES[4..],
            [
                "808 BD", "808 SD", "808 LT", "808 MT", "808 HT", "808 RS", "808 CP", "808 CB",
                "808 OH", "808 CH", "808 CY", "909 BD", "909 SD", "909 LT", "909 MT", "909 HT",
                "909 RS", "909 CP", "909 CH", "909 OH", "909 CC", "909 RC",
            ]
        );
        // no Roland names, no prefix
        for n in OUTPUT_NAMES {
            assert!(
                !n.contains("TR") && !n.contains("TB") && !n.contains("ORB"),
                "{n}"
            );
        }
    }

    #[test]
    fn the_keys_are_unique_and_stable() {
        let mut ids = ROUTING_IDS.to_vec();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), AUX_OUTPUTS);
        // released keys never change: these are the ones task-300 shipped
        assert_eq!(
            ROUTING_IDS.join(" "),
            "mix.303-1 mix.303-2 mix.808 mix.909 808.bd 808.sd 808.lt 808.mt 808.ht 808.rs \
             808.cp 808.cb 808.oh 808.ch 808.cy 909.bd 909.sd 909.lt 909.mt 909.ht 909.rs \
             909.cp 909.ch 909.oh 909.cc 909.rc"
        );
    }

    #[test]
    fn the_saved_routing_round_trips_and_an_absent_key_is_checked() {
        let d = OutputRouting::default();
        assert_eq!(from_saved(&BTreeMap::new()), d);
        assert_eq!(from_saved(&to_saved(&d)), d);
        for k in 0..AUX_OUTPUTS {
            let r = OutputRouting::from_bits(d.to_bits() & !(1 << k));
            let s = to_saved(&r);
            assert!(!s[ROUTING_IDS[k]]);
            assert_eq!(from_saved(&s), r);
        }
        let mut m = BTreeMap::new();
        m.insert("808.lt".to_string(), false);
        m.insert("unknown".to_string(), false);
        let r = from_saved(&m);
        assert!(!r.to_808[2]);
        assert_eq!(r.to_bits().count_ones() as usize, AUX_OUTPUTS - 1);
    }
}
