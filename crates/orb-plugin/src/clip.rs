//! The pattern drag-out's clip (task-203, `docs/plugin.md` section 12, `docs/midi.md` 5.6): one
//! pattern of a section as a Standard MIDI File in the plugin's note mapping, so that the host's
//! MIDI item played back into the plugin in note mode sounds as the pattern.
//!
//! - [`notes`]: the pattern's notes, in engine ticks (192 a quarter note, 48 a step).
//! - [`smf`]: a type-0 file of one track with them, at 192 ticks a quarter note, so a MIDI tick is
//!   an engine tick.
//! - [`write_temp`] and [`sweep`]: the temporary files the drag hands to the host.
//!
//! The 303's notes follow its step machinery (step 0x100253A0, note-on 0x10023CD4, note-off
//! 0x10023C74; `orb_engine::tb303`): the gate opens at a played step and closes 24 ticks later
//! unless the step slides. A step after a sliding one is legato (no new attack, the pitch glides):
//! a note that overlaps the sounding one by [`OVERLAP`] ticks, or the same note made longer when
//! the pitch is the same (a tie). A rest step after a slide is legato too: the original's rest
//! keeps the gate as it is and still moves the pitch (and the accent). Shuffle is baked in: with
//! the pattern's Shuffle on, the steps and the gate's ends move as the engine's shuffled tick
//! moves them (`orb_engine::seq::shuffled`, 0x1001F3D0).

use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use orb_engine::{tr808, tr909};

/// The engine's ticks of a step (a 16th note).
pub const STEP: u32 = 48;
/// The ticks of a quarter note: the file's division, so a MIDI tick is an engine tick.
pub const PPQ: u16 = 192;
/// The 303's gate: a step that does not slide closes it this many ticks after it opened
/// (`gate_off = v + 24`, 0x100254D3).
pub const GATE: u32 = 24;
/// How long a legato note overlaps the note before it (the earlier one ends this many ticks
/// after the later one starts): shorter than [`GATE`], so the later note sounds on alone.
pub const OVERLAP: u32 = 12;
/// The length of a drum note (a drum ignores the note-off).
pub const DRUM_LEN: u32 = 24;

/// The 303's key of pattern note 0 (C) with no Up or Down (`docs/midi.md` 5.3).
pub const TB303_C: i32 = 36;
/// The export velocities (`docs/midi.md` 5.6).
pub const VEL_NORMAL: u8 = 80;
pub const VEL_ACCENT: u8 = 127;
/// The 909's, by flam (values 1 and 2, then 3 and 4: `tr909::velocity_band`'s flam slices),
/// then low (value 1 or 3) and high (2 or 4), each without and with AC.
pub const VEL_909: [[[u8; 2]; 2]; 2] = [[[40, 104], [80, 127]], [[4, 98], [52, 114]]];

/// The bytes of one pattern: the shuffle flag, the length, 16 steps.
pub const LEN_303: usize = 2 + 16 * 6;
pub const LEN_DRUM: usize = 2 + 16 * 12;

/// The values the clip takes from the controls besides the pattern.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Controls {
    /// The shuffle amount (song msg 35 id 20, 0..127): a shuffled step moves `shuffle / 6` ticks.
    pub shuffle: u8,
    /// The 808's shared-slot switches in the order LT/LC, MT/MC, HT/HC, RS/CL, CP/MA
    /// (controllers 11, 14, 17, 19, 21): `true` is the tom, rim shot or clap (a non-zero byte).
    pub switches_808: [bool; 5],
}

impl Default for Controls {
    fn default() -> Self {
        Controls {
            shuffle: 0,
            switches_808: [true; 5],
        }
    }
}

/// One note of the clip, in engine ticks from the pattern's start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note {
    pub start: u32,
    pub end: u32,
    pub key: u8,
    pub velocity: u8,
}

/// The section names: in the track name and the file name.
pub const SECTIONS: [&str; 4] = ["303-1", "303-2", "808", "909"];

/// The pattern's name on the panel: bank A..D and button 1..8 (`pattern` 0..31).
pub fn pattern_name(pattern: u8) -> String {
    format!(
        "{}{}",
        (b'A' + (pattern / 8).min(3)) as char,
        pattern % 8 + 1
    )
}

/// The clip file's name: `303-1 A1.mid`, `808 B3.mid`.
pub fn file_name(row: usize, pattern: u8) -> String {
    format!("{} {}.mid", SECTIONS[row.min(3)], pattern_name(pattern))
}

/// The steps a pattern plays: its length byte, 1..16 (a hand-made 0 or more than 16 is cut; the
/// engine would divide by zero or read the next pattern's bytes).
fn steps(pattern: &[u8]) -> u32 {
    pattern.get(1).copied().unwrap_or(16).clamp(1, 16) as u32
}

/// The clip's length in ticks: the pattern's steps.
pub fn length(pattern: &[u8]) -> u32 {
    STEP * steps(pattern)
}

/// The ticks a shuffled step moves: `shuffle / 6` when the pattern's Shuffle is on (the
/// engine's shuffled tick `96*(t/96) + max(0, t%96 - shuffle/6)`, `orb_engine::seq::shuffled`).
fn shift(pattern: &[u8], c: &Controls) -> u32 {
    if pattern.first().copied().unwrap_or(0) != 0 {
        c.shuffle as u32 / 6
    } else {
        0
    }
}

/// When the engine's step `k` starts (the shuffled tick reaches `48k`): the second step of each
/// 8th moves `s` ticks later.
fn step_on(k: u32, s: u32) -> u32 {
    STEP * k + if k % 2 == 1 { s } else { 0 }
}

/// When the 303's gate of step `k` closes (the shuffled tick reaches `48k + 24`): every step's
/// moves `s` ticks later (the shuffled tick lags the tick by `s` in the whole 8th but its start).
fn gate_off(k: u32, s: u32) -> u32 {
    STEP * k + GATE + s
}

/// The notes of `pattern`, the bytes of one pattern of section `row` (0, 1 a 303, 2 the 808, 3
/// the 909), sorted by start.
pub fn notes(row: usize, pattern: &[u8], c: &Controls) -> Vec<Note> {
    let mut v = match row {
        0 | 1 => notes_303(pattern, c),
        2 => notes_drums(pattern, c, key_808, |r, _| {
            if r[tr808::AC] != 0 {
                VEL_ACCENT
            } else {
                VEL_NORMAL
            }
        }),
        _ => notes_drums(
            pattern,
            c,
            |i, _| key_909(i),
            |r, v| {
                // values 1 and 3 are low; 2, 4 and the flam-only 5..255 high; 3 and 4 flam (the
                // flam-only hit has no band: the step's high hit; `docs/midi.md` 5.6)
                let high = !matches!(v, 1 | 3);
                let flam = matches!(v, 3 | 4);
                VEL_909[flam as usize][high as usize][(r[tr909::AC] != 0) as usize]
            },
        ),
    };
    v.sort_by_key(|n| (n.start, n.key));
    v
}

/// One step of a pattern: its bytes, zero past the end.
fn step_bytes<const N: usize>(pattern: &[u8], k: u32) -> [u8; N] {
    let at = 2 + N * k as usize;
    std::array::from_fn(|i| pattern.get(at + i).copied().unwrap_or(0))
}

/// The 303's notes, from its gate (see the module's documentation).
fn notes_303(pattern: &[u8], c: &Controls) -> Vec<Note> {
    let s = shift(pattern, c);
    let len = length(pattern);
    // keys outside 0..127 (only a hand-made note byte gives one) are kept until the end, so the
    // gate stays right, and then dropped
    let mut out: Vec<(Note, bool)> = Vec::new();
    let mut sounding: Option<usize> = None;
    let mut slide = false;
    for k in 0..steps(pattern) {
        let [note, step_slide, accent, up, down, rest] = step_bytes::<6>(pattern, k);
        // 0x10023E61: (int16)(note + 12*up - 12*down)
        let semis = (note as i32 + 12 * up as i32 - 12 * down as i32) as i16 as i32;
        let key = TB303_C + semis;
        let valid = (0..=127).contains(&key);
        let key = key.clamp(0, 127) as u8;
        let velocity = if accent != 0 { VEL_ACCENT } else { VEL_NORMAL };
        let on = step_on(k, s);
        match sounding {
            // legato: the gate is open and the step before slid; a rest moves the pitch too
            Some(i) if slide => {
                if out[i].0.key != key || out[i].1 != valid {
                    out[i].0.end = on + OVERLAP;
                    out.push((
                        Note {
                            start: on,
                            end: len,
                            key,
                            velocity,
                        },
                        valid,
                    ));
                    sounding = Some(out.len() - 1);
                }
            }
            _ => {
                if let Some(i) = sounding {
                    // not reached: a step that does not slide has closed the gate
                    out[i].0.end = on;
                }
                sounding = None;
                // With `slide` set here (a rest that slid, the gate closed), the engine opens the
                // gate without a new attack and glides from the rest's pitch, which never
                // sounded; note mode cannot (a note with nothing held attacks), so the clip
                // writes an attacking note: a limit of the clip (`docs/midi.md` 5.6).
                if rest == 0 {
                    out.push((
                        Note {
                            start: on,
                            end: len,
                            key,
                            velocity,
                        },
                        valid,
                    ));
                    sounding = Some(out.len() - 1);
                }
            }
        }
        slide = step_slide != 0;
        if !slide && let Some(i) = sounding.take() {
            out[i].0.end = gate_off(k, s).min(len);
        }
    }
    // a slide out of the last step goes into the next loop's first step: the note ends with the
    // clip (it stays at `len`)
    out.into_iter().filter(|n| n.1).map(|n| n.0).collect()
}

/// The 808's key of instrument column `i` with the shared slots' switches (`docs/midi.md` 5.4).
fn key_808(i: usize, c: &Controls) -> Option<u8> {
    let sw = |n: usize, first: u8, second: u8| {
        if c.switches_808[n] { first } else { second }
    };
    Some(match i {
        tr808::BD => 36,
        tr808::SD => 38,
        tr808::LT => sw(0, 43, 64),
        tr808::MT => sw(1, 47, 63),
        tr808::HT => sw(2, 50, 62),
        tr808::RS => sw(3, 37, 75),
        tr808::CP => sw(4, 39, 70),
        tr808::CB => 56,
        tr808::CY => 49,
        tr808::OH => 46,
        tr808::CH => 42,
        _ => return None,
    })
}

/// The 909's key of instrument column `i` (`docs/midi.md` 5.4).
fn key_909(i: usize) -> Option<u8> {
    Some(match i {
        tr909::BD => 36,
        tr909::SD => 38,
        tr909::LT => 43,
        tr909::MT => 47,
        tr909::HT => 50,
        tr909::RS => 37,
        tr909::CP => 39,
        tr909::CH => 42,
        tr909::OH => 46,
        tr909::CC => 49,
        tr909::RC => 51,
        _ => return None,
    })
}

/// A drum machine's notes: every non-zero instrument byte of a step is a hit at the step's start.
fn notes_drums(
    pattern: &[u8],
    c: &Controls,
    key: impl Fn(usize, &Controls) -> Option<u8>,
    velocity: impl Fn(&[u8; 12], u8) -> u8,
) -> Vec<Note> {
    let s = shift(pattern, c);
    let mut out = Vec::new();
    for k in 0..steps(pattern) {
        let row = step_bytes::<12>(pattern, k);
        let start = step_on(k, s);
        for (i, &v) in row.iter().enumerate().skip(1) {
            if v == 0 {
                continue;
            }
            if let Some(key) = key(i, c) {
                out.push(Note {
                    start,
                    end: start + DRUM_LEN,
                    key,
                    velocity: velocity(&row, v),
                });
            }
        }
    }
    out
}

/// A variable-length quantity of a Standard MIDI File.
fn vlq(out: &mut Vec<u8>, mut v: u32) {
    let mut b = [0u8; 5];
    let mut n = 0;
    loop {
        b[n] = (v & 0x7F) as u8;
        n += 1;
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    for i in (0..n).rev() {
        out.push(b[i] | if i > 0 { 0x80 } else { 0 });
    }
}

/// A type-0 Standard MIDI File of `notes` on MIDI channel `channel` (0-based), named `name`, that
/// ends at tick `length`: 192 ticks a quarter note, a 4/4 time signature, no tempo (the host's
/// tempo plays it). At one tick note-offs come before note-ons.
pub fn smf(channel: u8, name: &str, notes: &[Note], length: u32) -> Vec<u8> {
    // (tick, order, bytes): order 0 the meta events, 1 note-offs, 2 note-ons
    let mut ev: Vec<(u32, u8, [u8; 3])> = Vec::new();
    for n in notes {
        ev.push((
            n.start,
            2,
            [0x90 | (channel & 15), n.key & 127, n.velocity.clamp(1, 127)],
        ));
        ev.push((
            n.end.max(n.start + 1),
            1,
            [0x80 | (channel & 15), n.key & 127, 0],
        ));
    }
    ev.sort_by_key(|e| (e.0, e.1));
    let mut t = Vec::new();
    // the track's name
    t.extend([0x00, 0xFF, 0x03]);
    vlq(&mut t, name.len() as u32);
    t.extend(name.as_bytes());
    // 4/4, 24 clocks a click, 8 32nds a quarter
    t.extend([0x00, 0xFF, 0x58, 0x04, 0x04, 0x02, 0x18, 0x08]);
    let mut now = 0;
    for (tick, _, b) in ev {
        vlq(&mut t, tick - now);
        now = tick;
        t.extend(b);
    }
    vlq(&mut t, length.max(now) - now);
    t.extend([0xFF, 0x2F, 0x00]);
    let mut f = Vec::with_capacity(22 + t.len());
    f.extend(b"MThd");
    f.extend(6u32.to_be_bytes());
    f.extend(0u16.to_be_bytes());
    f.extend(1u16.to_be_bytes());
    f.extend(PPQ.to_be_bytes());
    f.extend(b"MTrk");
    f.extend((t.len() as u32).to_be_bytes());
    f.extend(t);
    f
}

/// The clip of pattern `pattern` (0..31) of section `row`, whose bytes are `bytes`: a Standard
/// MIDI File on the section's channel (1..4).
pub fn clip(row: usize, pattern: u8, bytes: &[u8], c: &Controls) -> Vec<u8> {
    let row = row.min(3);
    let name = format!("{} {}", SECTIONS[row], pattern_name(pattern));
    smf(row as u8, &name, &notes(row, bytes, c), length(bytes))
}

/// The folder of the clips: `OpenRB-338/clips` in the temporary folder.
pub fn clips_dir() -> PathBuf {
    std::env::temp_dir().join("OpenRB-338").join("clips")
}

/// How long a clip's file is kept at least: the host reads it at the drop, a few seconds at most.
pub const KEEP: Duration = Duration::from_secs(600);

/// Writes a clip into a new folder of its own in `base` (so that its name is the clip's), after
/// sweeping `base` ([`sweep`]); returns the file's path.
pub fn write_temp(base: &Path, name: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    static N: AtomicU32 = AtomicU32::new(0);
    sweep(base, KEEP);
    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = base.join(format!("{}-{stamp}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(name);
    std::fs::write(&path, bytes)?;
    Ok(path)
}

/// Removes the clip folders of `base` older than `keep` (by their time of change), of any
/// process: a folder whose host still holds its file open (Windows) stays for the next sweep.
pub fn sweep(base: &Path, keep: Duration) {
    let Ok(dir) = std::fs::read_dir(base) else {
        return;
    };
    let now = SystemTime::now();
    for e in dir.flatten() {
        let old = e
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| now.duration_since(t).ok())
            .is_some_and(|age| age >= keep);
        if old && e.path().is_dir() {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

#[cfg(test)]
mod tests;
