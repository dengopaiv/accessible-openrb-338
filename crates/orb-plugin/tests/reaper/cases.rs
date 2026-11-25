//! The REAPER render cases (task-244, `docs/plugin.md` section 7): each case is an instance of
//! the plugin (a diag song, controls, a mode) on a REAPER track, in note mode with the
//! drag-out clip (task-203) of a section as a MIDI item, or in Latch Pattern Mode, rendered
//! through REAPER's command line. This module makes the projects from `template.RPP` and the
//! plugin's state; it is shared by the generator (`examples/reaper_fixtures.rs`) and the tests
//! (`tests/reaper.rs`).

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use nice_plug::prelude::Params;
use orb_engine::seq::{TEMPO_STEP, trunc};
use orb_player::midi::Message;
use orb_plugin::core::AUX_OUTPUTS;
use orb_plugin::instance::{Mode, OrbParams, set_param};
pub use orb_plugin::routing::{self, OUTPUT_NAMES};
use orb_plugin::{clip, editor};

/// The REAPER version the fixtures and the test were made with (`docs/plugin.md` section 7).
pub const REAPER_VERSION: &str = "7.86";

/// The plugin formats a fixture is made for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Vst3,
}

impl Format {
    pub const ALL: [Format; 1] = [Format::Vst3];

    pub fn ext(self) -> &'static str {
        match self {
            Format::Vst3 => "vst3",
        }
    }
}

/// The output tracks of a routed case: the track name and the aux bus (0-based, [`OUTPUT_NAMES`]
/// order) it takes. The bus is the plugin's output `k + 1` after the Mix, REAPER's channels
/// `2k + 2` and `2k + 3` (0-based; outputs 3/4 .. 9/10 for the four device buses, `docs/plugin.md`
/// section 13). A routed case has the four device tracks; a case with [`Case::drums`] also the
/// 22 drum tracks.
pub fn tracks(c: &Case) -> Vec<(&'static str, usize)> {
    let n = if c.drums { AUX_OUTPUTS } else { 4 };
    (0..n).map(|k| (OUTPUT_NAMES[k], k)).collect()
}

/// Where the MIDI item puts a clip's notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timing {
    /// At the notes' own ticks, as the dragged-out clip plays them.
    Clip,
    /// Each note moved to the first frame of the engine block in which Latch Pattern Mode plays
    /// the step at its tick (up to a block earlier, `docs/plugin.md` 9.5), so that note mode
    /// plays it in the same block: the comparison with the pattern without task-227's gap. The
    /// 303's first step's note-off is at tick 25, where the pattern's clean start closes its
    /// gate ([`notes`]; the clip's tick 24 differs, task-265).
    Aligned,
}

/// One render case.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    /// The fixture's name: `tests/reaper/<name>.vst3.RPP`.
    pub name: &'static str,
    /// A diag song, `tests/songs/diag/songs/<song>.rbs`.
    pub song: &'static str,
    pub mode: Mode,
    /// The project's rate.
    pub rate: u32,
    /// The project's tempo, BPM ([`TEMPO`]).
    pub tempo: f64,
    /// The rendered length, seconds.
    pub seconds: f64,
    /// Parameters set after the song's load (id, value).
    pub edits: &'static [(&'static str, i32)],
    /// The section (0, 1 the 303s, 2 the 808, 3 the 909) whose selected pattern's drag-out clip
    /// the MIDI item holds [`LOOPS`] times, and where its notes are; `None`: no MIDI item.
    pub clip: Option<(usize, Timing)>,
    /// A looped time selection (start, end), seconds: REAPER's `LOOP 1` and `SELECTION`.
    pub loop_selection: Option<(f64, f64)>,
    /// The four device outputs routed to tracks of their own ([`DEVICE_TRACKS`]), rendered as
    /// stems with the master.
    pub routed: bool,
    /// All 27 outputs routed (task-305): the 22 drum outputs on tracks of their own as well
    /// ([`tracks`]), on a track of 54 channels; implies `routed`.
    pub drums: bool,
    /// The routing boxes unchecked (Options > Output Routing, the keys of
    /// [`routing::ROUTING_IDS`]), saved in the state's `routing`.
    pub off: &'static [&'static str],
    /// A key the MIDI item also presses at tick 0 on the clip's channel and never lets go (a
    /// note-off the host lost, task-258), the item then running [`TAIL`] past the clips.
    pub orphan: Option<u8>,
}

/// How long the MIDI item of a case with an orphan note runs past its clips, seconds: the 303's
/// release is over in the first half, and the rest is silent unless a held note keeps the gate
/// open.
pub const TAIL: f64 = 1.5;

/// The projects' tempo, BPM. A quarter note is then 22050 frames at 44.1 kHz and 24000 at
/// 48 kHz: the MIDI items count in frames ([`ppq`]), so a note plays at the frame the case
/// gives it, with no rounding of REAPER's.
///
/// Not 125: the host's tick at the start of engine block `m` is `m * tempo * 512 / 110250`,
/// at 125 BPM exactly 768 (bar 2) at block 1323, where the plugin's position, extrapolated
/// from the start of the host's call, comes out as 768.0 or 767.9999999999999 by the call's
/// size, and the output (Latch Pattern Mode at 48 kHz, the note-mode clock's PCF at 44.1 kHz)
/// differs from there on by the host's buffer size (task-244's finding, `docs/plugin.md`
/// section 7). At 120 BPM the first block that starts on a whole tick that is a multiple of 24
/// (a step or a 303 gate) is 11025 (16 s), after every render's end, so the renders do not
/// depend on how REAPER splits its calls.
pub const TEMPO: f64 = 120.0;

/// The times a clip plays one after the other.
pub const LOOPS: u32 = 2;

/// A note-mode case: the song's selected pattern of `row` dragged out, with the edits.
const fn note(
    name: &'static str,
    song: &'static str,
    edits: &'static [(&'static str, i32)],
    row: usize,
    timing: Timing,
) -> Case {
    Case {
        name,
        song,
        mode: Mode::Note,
        rate: 44_100,
        tempo: TEMPO,
        seconds: 4.5,
        edits,
        clip: Some((row, timing)),
        loop_selection: None,
        routed: false,
        drums: false,
        off: &[],
        orphan: None,
    }
}

/// A note-mode case of the 303's clip with an orphan note-on of `key` at tick 0 (task-258).
const fn stuck(name: &'static str, song: &'static str, key: u8) -> Case {
    Case {
        seconds: 4.0 + TAIL + 0.5,
        orphan: Some(key),
        ..note(name, song, &[], 0, Timing::Clip)
    }
}

/// A Latch Pattern Mode case of the full mix.
const fn latch(
    name: &'static str,
    rate: u32,
    seconds: f64,
    loop_selection: Option<(f64, f64)>,
    routed: bool,
) -> Case {
    Case {
        name,
        song: "full_mix",
        mode: Mode::Latch,
        rate,
        tempo: TEMPO,
        seconds,
        edits: &[],
        clip: None,
        loop_selection,
        routed,
        drums: false,
        off: &[],
        orphan: None,
    }
}

/// A Latch Pattern Mode case of the full mix with all 27 outputs routed (task-305), the routing
/// boxes `off` unchecked and `edits` applied.
const fn drums(
    name: &'static str,
    off: &'static [&'static str],
    edits: &'static [(&'static str, i32)],
) -> Case {
    Case {
        edits,
        drums: true,
        off,
        ..latch(name, 44_100, 4.0, None, true)
    }
}

/// The PCF on the 808's channel (`d0c2` 3) and on the 909's (4), on, wave 0 (a step every 48
/// ticks: `tests/note_clock.rs`).
const PCF_808: &[(&str, i32)] = &[("d0c2", 3), ("d7c0", 1), ("d7c4", 0)];
const PCF_909: &[(&str, i32)] = &[("d0c2", 4), ("d7c0", 1), ("d7c4", 0)];

/// The cases (`docs/plugin.md` section 7): the 303's slide pattern and the 808's and 909's
/// accent patterns with the PCF on their channel, dragged out and played in note mode, each at
/// the clip's times and aligned to the pattern's blocks; the full mix in Latch Pattern Mode at
/// 48 kHz, and with the device outputs routed (and the same project unrouted); a pin of
/// REAPER's render ignoring a looped selection; and the 303's clip with a note-on whose
/// note-off never comes (task-258: key 39, which the clip plays and lets go later in each loop).
pub const CASES: [Case; 14] = [
    note("tb303_slide_clip", "303_slide", &[], 0, Timing::Clip),
    note("tb303_slide_aligned", "303_slide", &[], 0, Timing::Aligned),
    note(
        "tr808_pcf_clip",
        "808_accent_level127",
        PCF_808,
        2,
        Timing::Clip,
    ),
    note(
        "tr808_pcf_aligned",
        "808_accent_level127",
        PCF_808,
        2,
        Timing::Aligned,
    ),
    note(
        "tr909_pcf_clip",
        "909_accent_level127",
        PCF_909,
        3,
        Timing::Clip,
    ),
    note(
        "tr909_pcf_aligned",
        "909_accent_level127",
        PCF_909,
        3,
        Timing::Aligned,
    ),
    // a limitation pin, not a matching case: REAPER 7.86's command-line render ignores the
    // loop, and the test fails once it loops (then task-264 compares it with a looped host)
    latch("loop_ignored", 44_100, 6.0, Some((2.0, 4.0)), false),
    latch("latch_48k", 48_000, 4.0, None, false),
    latch("outputs_routed", 44_100, 4.0, None, true),
    latch("outputs_unrouted", 44_100, 4.0, None, false),
    stuck("tb303_stuck_note", "303_slide", 39),
    // task-305: the 22 drum outputs and the routing state. The 808's send (channel 2's fourth
    // control, d0c20) is on where its device is taken out of the Mix, so that its delay send
    // is heard
    drums("outputs_drums", &[], &[]),
    drums("routing_device", &["mix.808"], &[("d0c20", 100)]),
    drums("routing_drums", &["808.bd", "909.cp"], &[]),
];

pub fn case(name: &str) -> &'static Case {
    CASES
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("no case {name}"))
}

/// The plugin crate's folder.
pub fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The fixtures' folder, `crates/orb-plugin/tests/reaper`.
pub fn fixtures_dir() -> PathBuf {
    crate_dir().join("tests/reaper")
}

pub fn fixture_path(c: &Case, f: Format) -> PathBuf {
    fixtures_dir().join(format!("{}.{}.RPP", c.name, f.ext()))
}

/// Sets the parameter `id` to `v`, as the host does.
pub fn set(p: &OrbParams, id: &str, v: i32) {
    let param = p.param(id).unwrap_or_else(|| panic!("no parameter {id}"));
    unsafe { set_param(param, v) };
    assert_eq!(param.value(), v, "{id} takes {v}");
}

/// The case's instance as its project restores it: the song loaded (its controls set as the
/// editor sets them), the edits, the mode.
pub fn instance(c: &Case) -> OrbParams {
    let path = crate_dir().join(format!("../../tests/songs/diag/songs/{}.rbs", c.song));
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let p = OrbParams::new();
    for (id, v) in p.load_song(bytes).unwrap() {
        set(&p, &id, v);
    }
    for &(id, v) in c.edits {
        set(&p, id, v);
    }
    p.set_mode(c.mode);
    p.set_routing(routing_of(c.off));
    p
}

/// The routing with the boxes `off` (keys of [`routing::ROUTING_IDS`]) unchecked.
pub fn routing_of(off: &[&str]) -> orb_engine::frame::OutputRouting {
    let mut m = std::collections::BTreeMap::new();
    for id in off {
        assert!(routing::ROUTING_IDS.contains(id), "no routing key {id}");
        m.insert(id.to_string(), false);
    }
    routing::from_saved(&m)
}

/// The frames of a quarter note at the case's rate and tempo: the MIDI item's division.
pub fn ppq(c: &Case) -> u32 {
    let q = c.rate as f64 * 60.0 / c.tempo;
    assert_eq!(
        q,
        q.round(),
        "{}: a quarter note is not whole frames",
        c.name
    );
    q as u32
}

/// The frame nearest to tick `t` (192 a quarter note) on the host's timeline: where the MIDI
/// item puts a note of the clip.
pub fn frame_of(c: &Case, t: u32) -> usize {
    (t as f64 * ppq(c) as f64 / 192.0).round() as usize
}

/// The first frame of the block in which Latch Pattern Mode, the host rolling from beat 0 at
/// the case's tempo at 44.1 kHz, plays the step at tick `t`: the first block whose position
/// after its advance (the host's tick at its start plus the clock's advance) reaches `t`, and
/// at least tick 1 (the clean start's locate to 0 makes the step at 0 the first block's after
/// it). As `tests/note_clock.rs`'s `pattern_frame`.
pub fn pattern_frame(c: &Case, t: u32) -> usize {
    assert_eq!(c.rate, 44_100, "{}: the engine's rate", c.name);
    let inc = (c.tempo * 1000.0) as u32 as f64 * TEMPO_STEP;
    let host_tick = |f: usize| f as f64 * c.tempo / 60.0 / 44_100.0 * 192.0;
    let mut m = 0;
    while trunc(host_tick(m * 64) + inc) < (t as i32).max(1) {
        m += 1;
    }
    m * 64
}

/// The MIDI item's events of the case, (frame, message): the drag-out notes (`clip::notes`)
/// of the section's selected pattern, [`LOOPS`] times one after the other, at their ticks'
/// frames or aligned ([`Timing`]). At one frame the note-offs come first, then the note-ons
/// (REAPER's order), each kind by pitch; with them a case's orphan note-on, and last the
/// all-notes-off REAPER writes at the item's end. Also the item's end: the frame of the clips'
/// end, [`TAIL`] later with an orphan.
pub fn notes(c: &Case, p: &OrbParams) -> (Vec<(usize, Message)>, usize) {
    let Some((row, timing)) = c.clip else {
        return (Vec::new(), 0);
    };
    let values = p.panel_values();
    let patterns = p.view(&values).1;
    let controls = editor::clip_controls(&values);
    let len = if row < 2 {
        clip::LEN_303
    } else {
        clip::LEN_DRUM
    };
    let pattern = p.param(&format!("d{}c1", row + 1)).unwrap().value() as usize;
    let one = &patterns[row][pattern * len..(pattern + 1) * len];
    let ticks = clip::length(one);
    let channel = row as u8;
    let at = |t: u32| match timing {
        Timing::Clip => frame_of(c, t),
        Timing::Aligned => pattern_frame(c, t),
    };
    let mut v = Vec::new();
    for k in 0..LOOPS {
        for n in clip::notes(row, one, &controls) {
            let on = Message::NoteOn {
                channel,
                note: n.key,
                velocity: n.velocity,
            };
            let off = Message::NoteOff {
                channel,
                note: n.key,
                velocity: 0,
            };
            // aligned, the 303's first step: the pattern's clean start (Stop, Locate 0, Play)
            // steps it at tick 1, so its gate closes at tick 1 + 24 (`clip::GATE`), not 24 as
            // the clip has it. A real discrepancy between the clip and the pattern (up to
            // 0.0045 without this move), task-265; moved here so the rest is compared exactly
            // (within one LSB)
            let first = row < 2 && k == 0 && n.start == 0 && n.end == clip::GATE;
            let end = if timing == Timing::Aligned && first {
                n.end + 1
            } else {
                n.end
            };
            v.push((at(k * ticks + n.start), on));
            v.push((at(k * ticks + end), off));
        }
    }
    if let Some(key) = c.orphan {
        let on = Message::NoteOn {
            channel,
            note: key,
            velocity: clip::VEL_NORMAL,
        };
        v.push((0, on));
    }
    let rank = |m: &Message| match *m {
        Message::NoteOff { note, .. } => (0, note),
        Message::NoteOn { note, .. } => (1, note),
        _ => unreachable!(),
    };
    v.sort_by_key(|(f, m)| (*f, rank(m)));
    let tail = match c.orphan {
        Some(_) => (TAIL * c.rate as f64).round() as usize,
        None => 0,
    };
    let end = frame_of(c, LOOPS * ticks) + tail;
    // the item's end: REAPER's all-notes-off on channel 1 (no section note before task-258;
    // since, note mode lets go of 303-1's held notes, none in a case)
    let all_off = Message::ControlChange {
        channel: 0,
        controller: 123,
        value: 0,
    };
    v.push((end, all_off));
    (v, end)
}

/// nice-plug's state of the instance: its parameters by id and its persisted fields, as JSON
/// (`docs/plugin.md` section 5; the VST3 component state).
pub fn state_json(p: &OrbParams) -> String {
    let params: std::collections::BTreeMap<String, serde_json::Value> = p
        .ids()
        .iter()
        .zip(p.values())
        .map(|(id, v)| (id.clone(), serde_json::json!({ "i32": v })))
        .collect();
    serde_json::to_string(&serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "params": params,
        "fields": p.serialize_fields(),
    }))
    .unwrap()
}

/// The plugin's outputs mapped in a project: the Mix and the four device buses, stereo each,
/// as REAPER saved the released plugin (v2026.1118.0919). task-300's 22 drum buses come after
/// them and are not mapped here, as in a project saved with the released plugin; a case with
/// [`Case::drums`] maps all of them ([`outputs`]).
const OUTPUTS: u32 = 10;

/// The channels a case maps (two a stereo output): the Mix and every aux bus with [`Case::drums`],
/// else [`OUTPUTS`].
fn outputs(c: &Case) -> u32 {
    if c.drums {
        2 * (1 + AUX_OUTPUTS as u32)
    } else {
        OUTPUTS
    }
}

/// `data` as base64 lines of 96 bytes (128 characters), REAPER's width, each indented.
fn b64_lines(out: &mut String, indent: &str, data: &[u8]) {
    for chunk in data.chunks(96) {
        out.push_str(indent);
        out.push_str(&B64.encode(chunk));
        out.push('\n');
    }
}

/// The plugin's block in a track's FX chain, as REAPER 7.86 saves it.
///
/// The VST3 form: the line `<VST "VST3i: ..." OpenRB-338.vst3 0 "" <id>{<class id>} ""`, then
/// base64 lines: a header (the id as a little-endian u32, 0xFEED5EEE, the input count 0 with no
/// pin masks, the output count 10 and each output's u64 pin mask, output n on track channel n),
/// one line of the data's length, 1 and 0xFFFF (u32 each), the data (the component state's
/// length and 1, the state, then the controller state's length 0 and a 0, u32 each), and a
/// trailer of six zero bytes.
pub fn fx_block(f: Format, json: &str, outputs: u32) -> String {
    let mut s = String::new();
    match f {
        Format::Vst3 => {
            // REAPER's id of the class (the decimal before the GUID), as it saved it
            let id: u32 = 1_476_801_187;
            s.push_str(&format!(
                "      <VST \"VST3i: OpenRB-338 (OpenRB-338) ({outputs} out)\" OpenRB-338.vst3 0 \
                 \"\" \
                 {id}{{EFAB08AFB39D43998A803DC1F84ED3D6}} \"\"\n"
            ));
            let mut head = Vec::new();
            head.extend(id.to_le_bytes());
            head.extend(0xFEED_5EEEu32.to_le_bytes());
            head.extend(0u32.to_le_bytes());
            head.extend(outputs.to_le_bytes());
            for n in 0..outputs {
                head.extend((1u64 << n).to_le_bytes());
            }
            let mut data = Vec::new();
            data.extend((json.len() as u32).to_le_bytes());
            data.extend(1u32.to_le_bytes());
            data.extend(json.as_bytes());
            data.extend(0u32.to_le_bytes());
            data.extend(0u32.to_le_bytes());
            let mut size = Vec::new();
            size.extend((data.len() as u32).to_le_bytes());
            size.extend(1u32.to_le_bytes());
            size.extend(0xFFFFu32.to_le_bytes());
            b64_lines(&mut s, "        ", &head);
            b64_lines(&mut s, "        ", &size);
            b64_lines(&mut s, "        ", &data);
            b64_lines(&mut s, "        ", &[0; 6]);
            s.push_str("      >\n");
        }
    }
    s
}

/// A number as REAPER writes it: no trailing zeros.
fn num(x: f64) -> String {
    let s = format!("{x:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// The instrument track: the plugin, and the MIDI item of [`notes`] (its division a quarter
/// note's frames, [`ppq`], so an event's position is its frame).
fn instrument_track(c: &Case, f: Format, p: &OrbParams) -> String {
    let mut s = String::from("  <TRACK\n    NAME \"OpenRB-338\"\n");
    // the outputs on the track's channels (two each) when the output tracks take them
    s.push_str(&format!(
        "    NCHAN {}\n",
        if c.routed { outputs(c) } else { 2 }
    ));
    s.push_str("    MAINSEND 1 0\n");
    s.push_str("    <FXCHAIN\n      SHOW 0\n      LASTSEL 0\n      DOCKED 0\n      BYPASS 0 0 0\n");
    s.push_str(&fx_block(f, &state_json(p), outputs(c)));
    s.push_str("      FLOATPOS 0 0 0 0\n      WAK 0 0\n    >\n");
    let (ev, end) = notes(c, p);
    if !ev.is_empty() {
        s.push_str("    <ITEM\n      POSITION 0\n");
        s.push_str(&format!(
            "      LENGTH {}\n",
            num(end as f64 / c.rate as f64)
        ));
        s.push_str("      LOOP 0\n      <SOURCE MIDI\n");
        s.push_str(&format!("        HASDATA 1 {} QN\n", ppq(c)));
        let mut at = 0;
        for (t, m) in &ev {
            let (status, d1, d2) = match *m {
                Message::NoteOn {
                    channel,
                    note,
                    velocity,
                } => (0x90 | channel, note, velocity),
                Message::NoteOff { channel, note, .. } => (0x80 | channel, note, 0),
                Message::ControlChange {
                    channel,
                    controller,
                    value,
                } => (0xb0 | channel, controller, value),
                _ => unreachable!(),
            };
            s.push_str(&format!(
                "        E {} {status:02x} {d1:02x} {d2:02x}\n",
                t - at
            ));
            at = *t;
        }
        // the last event is the item's end ([`notes`])
        assert_eq!(at, end, "{}: the item's end", c.name);
        s.push_str("      >\n    >\n");
    }
    s.push_str("  >\n");
    s
}

/// A device output's track: it receives the instrument track's channels `src`, `src + 1`
/// before its fader (post-FX), sends nothing to the master, and is selected (a stem).
fn device_track(name: &str, src: u32) -> String {
    format!(
        "  <TRACK\n    NAME \"{name}\"\n    SEL 1\n    NCHAN 2\n    MAINSEND 0 0\n    \
         AUXRECV 0 3 1 0 0 0 0 {src} 0 -1:U 0 -1 ''\n  >\n"
    )
}

/// The case's project for the format `f`, from the template.
pub fn project(c: &Case, f: Format, template: &str) -> String {
    let p = instance(c);
    let mut text = instrument_track(c, f, &p);
    if c.routed {
        for (name, bus) in tracks(c) {
            text.push_str(&device_track(name, 2 * (bus as u32 + 1)));
        }
    }
    let (looped, sel) = match c.loop_selection {
        Some((a, b)) => (1, format!("{} {}", num(a), num(b))),
        None => (0, "0 0".to_string()),
    };
    template
        .replace("@RATE@", &c.rate.to_string())
        .replace("@TEMPO@", &num(c.tempo))
        .replace("@LOOP@", &looped.to_string())
        .replace("@SELECTION@", &sel)
        .replace("@SECONDS@", &num(c.seconds))
        // 1: the master mix and a stem of each selected track; 0: the master mix
        .replace("@STEMS@", if c.routed { "1" } else { "0" })
        .replace("@TRACKS@", &text)
}

/// The template, `tests/reaper/template.RPP`.
pub fn template() -> String {
    let p = fixtures_dir().join("template.RPP");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}
