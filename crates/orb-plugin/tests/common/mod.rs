//! The plugin tests' helpers: a corpus song, an instance with it loaded, and a host that calls
//! `process` with its buffer size, split at the parameter changes as nice-plug's
//! sample-accurate automation splits it.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use orb_plugin::core::{Core, HostTransport};
use orb_plugin::instance::{self, Mode, OrbParams, set_param};

/// The transport of [`host`]: playing, without a tempo or a position (a host that reports
/// neither), so that pattern mode plays the song free-running at its own tempo.
pub const PLAYING: HostTransport = HostTransport {
    playing: true,
    tempo: None,
    pos_beats: None,
};

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The bytes of a diag corpus song, `tests/songs/diag/songs/<name>.rbs`.
pub fn song_bytes(name: &str) -> Vec<u8> {
    let p = root().join(format!("tests/songs/diag/songs/{name}.rbs"));
    std::fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Sets the parameter `id` to `v`, as the host does before a call.
pub fn set(p: &OrbParams, id: &str, v: i32) {
    let param = p.param(id).unwrap_or_else(|| panic!("no parameter {id}"));
    unsafe { set_param(param, v) };
    assert_eq!(param.value(), v, "{id} takes {v}");
}

/// An instance with the song `name` loaded (its controls set as the editor would) in pattern
/// mode, as File > Open leaves it.
pub fn with_song(name: &str) -> OrbParams {
    let p = OrbParams::new();
    for (id, v) in p.load_song(song_bytes(name)).unwrap() {
        set(&p, &id, v);
    }
    p.set_mode(Mode::Pattern);
    p
}

/// An activation at `rate`.
pub fn activate(p: &OrbParams, rate: u32) -> Core {
    instance::core(p, rate)
}

/// A parameter change at a host frame.
#[derive(Clone, Copy, Debug)]
pub struct Change {
    pub at: usize,
    pub id: &'static str,
    pub value: i32,
}

/// Runs `frames` host frames in buffers of `buf` frames, each split at the changes, which are
/// set before the call that starts at their frame; the host plays ([`PLAYING`]). Returns the
/// frames interleaved.
pub fn host(
    p: &OrbParams,
    core: &mut Core,
    frames: usize,
    buf: usize,
    changes: &[Change],
) -> Vec<f32> {
    host_with(p, core, frames, buf, changes, &PLAYING)
}

/// [`host`] with the transport `t` in every call.
pub fn host_with(
    p: &OrbParams,
    core: &mut Core,
    frames: usize,
    buf: usize,
    changes: &[Change],
    t: &HostTransport,
) -> Vec<f32> {
    let (mut l, mut r) = (vec![0f32; frames], vec![0f32; frames]);
    let mut pos = 0;
    while pos < frames {
        for c in changes.iter().filter(|c| c.at == pos) {
            set(p, c.id, c.value);
        }
        let block_end = (pos / buf + 1) * buf;
        let next_change = changes
            .iter()
            .map(|c| c.at)
            .filter(|&a| a > pos)
            .min()
            .unwrap_or(usize::MAX);
        let end = block_end.min(next_change).min(frames);
        core.process_with(&mut l[pos..end], &mut r[pos..end], &p.values(), t, &[]);
        pos = end;
    }
    l.into_iter().zip(r).flat_map(|(a, b)| [a, b]).collect()
}

/// A third-party mod for the tests: the standard mod renamed "Test Kit", with the 808's bass
/// drum and first snare swapped, and the 909's.
pub fn test_mod() -> Vec<u8> {
    let r = rbm::Rbm::parse(rbm::standard::EMBEDDED.to_vec()).unwrap();
    let t = rbm::KeyTable::embedded(&r).unwrap();
    let mut u = rbm::pack::unpack(&r, &t).unwrap();
    u.manifest.info.name = "Test Kit".into();
    u.manifest.info_raw = None;
    let at = |name: &str| {
        u.manifest
            .files
            .iter()
            .position(|e| e.name.eq_ignore_ascii_case(name))
            .unwrap_or_else(|| panic!("no {name}"))
    };
    let pairs = [
        (at("TR808BD.aif"), at("TR808SD1.aif")),
        (at("TR909bd1.aif"), at("TR909sd1.aif")),
    ];
    for (a, b) in pairs {
        u.files.swap(a, b);
    }
    rbm::pack::pack(&u.manifest, &u.files).unwrap()
}

/// The peak of interleaved frames.
pub fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |m, v| m.max(v.abs()))
}
