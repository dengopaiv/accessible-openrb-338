//! task-242: the note-mode clock. In note mode the song is stopped, and the PCF takes its step
//! from the note-mode clock (`docs/plugin.md` 9.5): the host's position while the host plays,
//! free-running at the tempo from where it was while the host is stopped. A pattern dragged out
//! (task-203) and played back as notes steps the PCF as the pattern does in pattern mode.

mod common;

use common::{activate, set, with_song};
use nice_plug::prelude::Params;
use orb_engine::pcf;
use orb_engine::seq::{TEMPO_STEP, trunc};
use orb_player::midi::Message;
use orb_plugin::core::{Core, HostEvent, HostTransport, TICKS_PER_BEAT};
use orb_plugin::instance::{Mode, OrbParams, STATE_KEY};
use orb_plugin::{clip, editor};

/// The host's tempo, BPM (the diag songs' own).
const TEMPO: f64 = 120.0;
/// The engine frames of a block.
const BLOCK: usize = 64;
/// The PCF's wave (0: 16 steps of resolution 16, a step every 48 ticks).
const WAVE: u8 = 0;
/// The PCF's place for the 808 and the 909 (`d0c2`: channel 3 and 4).
const PCF_808: i32 = 3;
const PCF_909: i32 = 4;

/// The host plays from beat 0 at [`TEMPO`]: its transport at host frame `f`.
fn rolling(f: usize) -> HostTransport {
    HostTransport {
        playing: true,
        tempo: Some(TEMPO),
        pos_beats: Some(f as f64 * TEMPO / 60.0 / 44_100.0),
    }
}

/// The host stopped at beat `beats`.
fn stopped(beats: f64) -> HostTransport {
    HostTransport {
        playing: false,
        tempo: Some(TEMPO),
        pos_beats: Some(beats),
    }
}

/// The host's tick at host frame `f` while it rolls, as the plugin computes it.
fn host_tick(f: usize) -> f64 {
    f as f64 * TEMPO / 60.0 / 44_100.0 * TICKS_PER_BEAT
}

/// The clock's advance of one block at [`TEMPO`].
fn inc() -> f64 {
    (TEMPO * 1000.0) as u32 as f64 * TEMPO_STEP
}

/// The PCF's step quotient at tick `t` (`t / 48` for [`WAVE`], 0x10016792..0x100167AE).
fn q(t: f64) -> i32 {
    pcf::quotient(trunc(t), pcf::pattern(WAVE).1).unwrap()
}

/// The PCF's step quotient as its last block left it.
fn quotient(core: &Core) -> i32 {
    core.audio()
        .current()
        .frame
        .devices
        .pcf
        .stack_word(None)
        .expect("the PCF is on, at a resolution of 8, 16 or 32")
}

/// Runs `frames` host frames in calls of one frame, so that every engine block is seen, with
/// the transport `tr(f)` and the MIDI messages `ev` (host frame, message). Returns the output,
/// interleaved, and the PCF's quotient after each block.
fn by_frame(
    p: &OrbParams,
    core: &mut Core,
    frames: usize,
    tr: impl Fn(usize) -> HostTransport,
    ev: &[(usize, Message)],
) -> (Vec<f32>, Vec<i32>) {
    let mut out = Vec::with_capacity(2 * frames);
    let mut steps = Vec::new();
    let first = core.blocks();
    for f in 0..frames {
        let events: Vec<HostEvent> = ev
            .iter()
            .filter(|e| e.0 == f)
            .map(|e| HostEvent {
                frame: 0,
                message: e.1,
            })
            .collect();
        let (mut l, mut r) = ([0f32], [0f32]);
        core.process_with(&mut l, &mut r, &p.values(), &tr(f), &events);
        out.extend([l[0], r[0]]);
        let n = (core.blocks() - first) as usize;
        assert!(n <= steps.len() + 1, "one block a call");
        if n > steps.len() {
            steps.push(quotient(core));
        }
    }
    (out, steps)
}

/// An instance of the diag song `song` in `mode` with the PCF at `pcf` (`d0c2`), on, at
/// [`WAVE`].
fn instance(song: &str, pcf: i32, mode: Mode) -> (OrbParams, Core) {
    instance_with(song, pcf, mode, |_| {})
}

/// [`instance`] with `edit` applied to the instance after the song's load.
fn instance_with(song: &str, pcf: i32, mode: Mode, edit: fn(&OrbParams)) -> (OrbParams, Core) {
    let p = with_song(song);
    edit(&p);
    set(&p, "d0c2", pcf);
    set(&p, "d7c0", 1);
    set(&p, "d7c4", WAVE as i32);
    p.set_mode(mode);
    let core = activate(&p, 44_100);
    (p, core)
}

/// The drag-out's notes (`clip::notes`, task-203) of the selected patterns of the drum
/// sections `rows` (2 the 808, 3 the 909), `loops` times: (tick, message).
fn clip_notes(p: &OrbParams, rows: &[usize], loops: u32) -> Vec<(u32, Message)> {
    let values = p.panel_values();
    let patterns = p.view(&values).1;
    let mut v = Vec::new();
    for &row in rows {
        let pattern = p.param(&format!("d{}c1", row + 1)).unwrap().value() as usize;
        let one = &patterns[row][pattern * clip::LEN_DRUM..(pattern + 1) * clip::LEN_DRUM];
        let len = clip::length(one);
        let notes = clip::notes(row, one, &editor::clip_controls(&values));
        assert!(!notes.is_empty());
        let channel = row as u8;
        for k in 0..loops {
            for n in &notes {
                let on = Message::NoteOn {
                    channel,
                    note: n.key,
                    velocity: n.velocity,
                };
                let off = Message::NoteOff {
                    channel,
                    note: n.key,
                    velocity: 64,
                };
                v.push((k * len + n.start, on));
                v.push((k * len + n.end, off));
            }
        }
    }
    v.sort_by_key(|e| e.0);
    v
}

/// The first host frame whose position reaches tick `t`: where the host's MIDI item plays a
/// note at `t`.
fn frame_of(t: u32) -> usize {
    (t as f64 * 44_100.0 * 60.0 / (TEMPO * TICKS_PER_BEAT)).ceil() as usize
}

/// The first frame of the block in which pattern mode plays the step at tick `t`: the first
/// block whose position after its advance (the host's tick at its start plus the clock's
/// advance) reaches `t`, and at least tick 1 (the clean start's locate to 0 makes the step
/// at 0 the first block's after it: its clock steps the devices from tick 1).
fn pattern_frame(t: u32) -> usize {
    let mut m = 0;
    while trunc(host_tick(m * BLOCK) + inc()) < (t as i32).max(1) {
        m += 1;
    }
    m * BLOCK
}

/// The first frame where `a` and `b` differ, the frames that differ, and the largest
/// difference.
fn diff(a: &[f32], b: &[f32]) -> (Option<usize>, usize, f32) {
    let frames: Vec<usize> = (0..a.len().min(b.len()))
        .filter(|&i| a[i] != b[i])
        .map(|i| i / 2)
        .collect();
    let mut n = frames.clone();
    n.dedup();
    let max = a.iter().zip(b).fold(0f32, |m, (x, y)| m.max((x - y).abs()));
    (frames.first().copied(), n.len(), max)
}

/// One LSB of the int16 output.
const LSB: f32 = 1.0 / 32768.0;

/// The pattern of `song` played in Latch Pattern Mode with the host rolling from bar 1 against
/// its drag-out played into note mode (two loops, 4 s at 120 BPM): the notes at the host's
/// frames and the notes in the blocks where the pattern plays its steps. The PCF's steps are
/// the same, block for block, in all three. Returns the output differences of both.
fn compare(song: &str, rows: &[usize], pcf: i32) -> [(Option<usize>, usize, f32); 2] {
    compare_with(song, rows, pcf, |_| {})
}

/// [`compare`] with `edit` applied to both instances after the song's load.
fn compare_with(
    song: &str,
    rows: &[usize],
    pcf: i32,
    edit: fn(&OrbParams),
) -> [(Option<usize>, usize, f32); 2] {
    let frames = 4 * 44_100;
    let (pp, mut pc) = instance_with(song, pcf, Mode::Latch, edit);
    let (pattern_out, pattern_steps) = by_frame(&pp, &mut pc, frames, rolling, &[]);
    // 2 bars of 16 steps, and the first of the third (the blocks run one ahead)
    assert_eq!(
        *pattern_steps.last().unwrap(),
        32,
        "{song}: the PCF steps through 2 bars"
    );
    let notes = clip_notes(&pp, rows, 2);
    let mut d = [(None, 0, 0.0); 2];
    for (k, at) in [frame_of as fn(u32) -> usize, pattern_frame]
        .iter()
        .enumerate()
    {
        let ev: Vec<(usize, Message)> = notes.iter().map(|&(t, m)| (at(t), m)).collect();
        let (np, mut nc) = instance_with(song, pcf, Mode::Note, edit);
        let (note_out, note_steps) = by_frame(&np, &mut nc, frames, rolling, &ev);
        assert!(
            !nc.audio().last_snapshot().playing,
            "note mode keeps the song stopped"
        );
        assert_eq!(
            note_steps, pattern_steps,
            "{song}: the PCF's steps, block by block"
        );
        d[k] = diff(&pattern_out, &note_out);
        eprintln!(
            "{song} {}: {:?}",
            ["host frames", "pattern blocks"][k],
            d[k]
        );
    }
    d
}

#[test]
fn the_dragged_808_pattern_steps_the_pcf_as_the_pattern_does() {
    let [host, aligned] = compare("808_accent_level127", &[2], PCF_808);
    // at the host's frames the notes come up to a block later than the steps: task-227's
    // timing rule (an event reaches the first block at or after its frame; `docs/plugin.md`
    // 9.5), so the same hits are shifted; the measured largest difference
    assert!(host.2 <= 1.5874329, "{host:?}");
    // in the steps' blocks: one LSB now and then (also without the PCF; 9.5, task-243)
    assert!(aligned.0.unwrap() > 3000 && aligned.2 <= LSB, "{aligned:?}");
}

#[test]
fn the_dragged_909_pattern_steps_the_pcf_as_the_pattern_does() {
    let [host, aligned] = compare("909_accent_level127", &[3], PCF_909);
    // task-227's timing rule, as for the 808; the measured largest difference
    assert!(host.2 <= 1.7430725, "{host:?}");
    assert!(aligned.2 <= LSB, "{aligned:?}");
}

#[test]
fn dragged_shuffled_patterns_step_the_pcf_as_the_patterns_do() {
    let [host, aligned] = compare("shuffle_drums_g64", &[2, 3], PCF_808);
    // task-227's timing rule, as for the 808; the measured largest difference
    assert!(host.2 <= 0.21432495, "{host:?}");
    assert!(aligned.2 <= LSB, "{aligned:?}");
}

/// The user's 909 pattern A6 (task-245, `909_a6_clap.RPP`): AC 2 on every step, BD 2, CH and OH
/// 1, the crash 1 on step 1 and the hand clap 3 (low + flam) on steps 5 and 13. Columns AC, BD,
/// SD, LT, MT, HT, RS, CP, CH, OH, CC, RC.
const A6: [[u8; 12]; 16] = {
    let mut s = [[0u8; 12]; 16];
    let mut k = 0;
    while k < 16 {
        s[k][0] = 2;
        if k % 4 == 0 {
            s[k][1] = 2;
        }
        if k % 4 < 2 {
            s[k][8] = 1;
        } else if k % 4 == 2 {
            s[k][9] = 1;
        }
        k += 1;
    }
    s[0][10] = 1;
    s[4][7] = 3;
    s[12][7] = 3;
    s
};

/// The largest difference of [`A6`]'s notes at the host's frames (measured).
const HOST_A6: f32 = 1.9999695;

/// The 909's selected pattern becomes [`A6`], with the user's Flam knob (54), Accent (127) and
/// clap level (121).
fn user_a6(p: &OrbParams) {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;
    let values = p.panel_values();
    let mut patterns = p.view(&values).1;
    let at = p.param("d4c1").unwrap().value() as usize * clip::LEN_DRUM;
    let one = &mut patterns[3][at..at + clip::LEN_DRUM];
    one[..2].copy_from_slice(&[0, 16]);
    one[2..].copy_from_slice(A6.as_flattened());
    let mut fields = p.serialize_fields();
    let mut doc: serde_json::Value = serde_json::from_str(&fields[STATE_KEY]).unwrap();
    doc["patterns"] = patterns.iter().map(|a| b64.encode(a)).collect();
    fields.insert(STATE_KEY.to_string(), doc.to_string());
    p.deserialize_fields(&fields);
    set(p, "d4c29", 54);
    set(p, "d4c2", 127);
    set(p, "d4c21", 121);
}

/// task-245: the clap's flam (value 3) is carried by the drag-out's flam band and played by
/// note mode's flam as the pattern's flam pass plays it, so the dragged A6 sounds as the
/// pattern (with the flam bands written as the plain ones, the aligned notes differ by 0.51,
/// in 111324 frames, from frame 3080).
#[test]
fn the_dragged_909_flam_pattern_plays_as_the_pattern() {
    let [host, aligned] = compare_with("909_accent_level127", &[3], PCF_909, user_a6);
    // task-227's timing rule, as above; the measured largest difference
    assert!(host.2 <= HOST_A6, "{host:?}");
    // in the steps' blocks: within one LSB (9.5, task-243)
    assert!(aligned.2 <= LSB, "{aligned:?}");
}

/// task-245 review: a note-on of velocity 0 on the 909's channel (`0x93 39 0`, which the MIDI
/// parser keeps as a note-on) is the note-off, through the core and the engine: no hit, no flam
/// retrigger, silence.
#[test]
fn a_velocity_0_note_on_plays_no_909_hit_and_no_flam() {
    let (p, mut core) = instance("909_accent_level127", PCF_909, Mode::Note);
    let ev: Vec<(usize, Message)> = [39, 36, 42]
        .into_iter()
        .map(|note| {
            (
                100,
                Message::NoteOn {
                    channel: 3,
                    note,
                    velocity: 0,
                },
            )
        })
        .collect();
    let (out, _) = by_frame(&p, &mut core, 22_050, rolling, &ev);
    assert!(out.iter().all(|&x| x == 0.0), "silence");
    let hits = core.audio().last_snapshot().sounding[3].hits;
    assert_eq!(hits, [0; 16], "no hit, no flam retrigger");
}

#[test]
fn with_the_host_stopped_the_pcf_runs_on_from_the_last_host_position() {
    let (p, mut core) = instance("808_accent_level127", PCF_808, Mode::Note);
    let clock = |c: &Core| c.audio().current().frame.note_clock.unwrap();
    // the host plays two beats: the clock is the host's tick plus the block's advance
    let (_, steps) = by_frame(&p, &mut core, 2 * 22_050, rolling, &[]);
    let blocks = core.blocks();
    let last = host_tick((blocks - 1) as usize * BLOCK) + inc();
    assert_eq!(clock(&core), last);
    assert!(steps.windows(2).all(|w| w[1] >= w[0]));
    assert_eq!(*steps.last().unwrap(), q(last));
    // the host stops (its position stays at beat 2): the clock runs on from where it was
    let (_, steps) = by_frame(&p, &mut core, 44_100, |_| stopped(2.0), &[]);
    let n = (core.blocks() - blocks) as usize;
    assert_eq!(steps.len(), n);
    let mut c = last;
    let mut want = Vec::new();
    for _ in 0..n {
        c += inc();
        want.push(q(c));
    }
    assert_eq!(steps, want, "free-running from the last host position");
    assert_eq!(clock(&core), c);
    // a second at 120 BPM is 384 ticks, 8 steps
    assert_eq!(
        want.last().unwrap() - want[0],
        8,
        "the step advances across blocks"
    );
    assert!(!core.audio().last_snapshot().playing);
}

#[test]
fn a_stopped_host_from_the_start_runs_the_clock_from_the_songs_position() {
    let (p, mut core) = instance("808_accent_level127", PCF_808, Mode::Note);
    let (_, steps) = by_frame(&p, &mut core, 44_100, |_| stopped(0.0), &[]);
    let mut c = 0.0;
    let want: Vec<i32> = (0..steps.len())
        .map(|_| {
            c += inc();
            q(c)
        })
        .collect();
    assert_eq!(steps, want);
    // a second (the blocks run one ahead of the host's frames): 384 ticks and a block
    assert_eq!(*want.last().unwrap(), 8);
}

#[test]
fn the_pattern_modes_and_the_preview_run_no_note_clock() {
    let song = "808_accent_level127";
    for mode in [Mode::Latch, Mode::Gate] {
        let (p, mut core) = instance(song, PCF_808, mode);
        by_frame(&p, &mut core, 8192, rolling, &[]);
        assert_eq!(core.audio().current().frame.note_clock, None, "{mode:?}");
    }
    // note mode, then the preview: the clock is off while it runs
    let (p, mut core) = instance(song, PCF_808, Mode::Note);
    by_frame(&p, &mut core, 4096, rolling, &[]);
    assert!(core.audio().current().frame.note_clock.is_some());
    p.set_preview(true);
    by_frame(&p, &mut core, 4096, rolling, &[]);
    assert!(core.previewing());
    assert_eq!(core.audio().current().frame.note_clock, None);
    // back to note mode: the host's position again
    p.set_preview(false);
    let (_, steps) = by_frame(&p, &mut core, 4096, |f| rolling(f + 8192), &[]);
    let last = host_tick((core.blocks() - 1) as usize * BLOCK) + inc();
    assert_eq!(core.audio().current().frame.note_clock, Some(last));
    assert_eq!(*steps.last().unwrap(), q(last));
    // note mode to Latch: off
    p.set_mode(Mode::Latch);
    by_frame(&p, &mut core, 4096, rolling, &[]);
    assert_eq!(core.audio().current().frame.note_clock, None);
}

#[test]
fn leaving_the_preview_with_the_host_stopped_continues_the_pcf_from_the_previews_tick() {
    let (p, mut core) = instance("808_accent_level127", PCF_808, Mode::Note);
    let stop = |_| stopped(0.0);
    by_frame(&p, &mut core, 4096, stop, &[]);
    // the preview plays the song free-running from its bar start for a second
    p.set_preview(true);
    let (_, steps) = by_frame(&p, &mut core, 44_100, stop, &[]);
    assert!(core.previewing());
    let seq = &core.audio().current().frame.seq;
    let reached = seq.pos;
    assert!(reached > 384.0, "{reached}");
    assert_eq!(*steps.last().unwrap(), q(reached), "the preview's PCF step");
    // leaving it: the song stops (at its bar start), the clock runs on from the preview's tick
    p.set_preview(false);
    let before = core.blocks();
    let (_, steps) = by_frame(&p, &mut core, 22_050, stop, &[]);
    let n = (core.blocks() - before) as usize;
    let mut c = reached;
    let want: Vec<i32> = (0..n)
        .map(|_| {
            c += inc();
            q(c)
        })
        .collect();
    assert_eq!(
        steps, want,
        "continued from the preview's tick, not from the bar start"
    );
    assert_eq!(core.audio().current().frame.note_clock, Some(c));
    let s = core.audio().current();
    assert!(!s.playing());
    assert_eq!(
        s.frame.seq.tick_pos(),
        0,
        "the song itself is at its bar start"
    );
}
