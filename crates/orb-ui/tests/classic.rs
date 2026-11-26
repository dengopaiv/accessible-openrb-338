//! The classic skin (task-267, docs/ui_spec.md section 21): the panel composed from a kit's 58
//! bitmaps at the original's geometry, driven by the one controller.
//! - It draws what the Theme Viewer of task-187 drew, pixel for pixel, for many panel states (the
//!   viewer's own list of blits is kept here as the reference, `viewer`).
//! - Its hit areas are the original's (read in IDA: the step views, the instrument selection, the
//!   pattern grid, the PCF faders, the knob spans).
//! - The same input gives the same panel state in the modern and the classic skin, one test per
//!   control kind.
//! - A mod's bitmaps fall back one by one; a set at 2x composes the 1x image upscaled.
//! - View > Classic / Modern switches the main window's skin and is kept in the settings.
//!
//! Everything uses the embedded standard mod; the third-party collection's test is gated on
//! `ORB_THIRDPARTY_MODS_DIR`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use eframe::egui::{
    self, Color32, ColorImage, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2,
};
use orb_cli::mods::Kit;
use orb_ui::app::App;
use orb_ui::behavior;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::menu::Cmd;
use orb_ui::model::{Panel, SELECTOR_808};
use orb_ui::settings::{Settings, Theme};
use orb_ui::skin::bitmaps::{self, Bitmaps, IDS, W, compose, upscale};
use orb_ui::skin::classic::ClassicSkin;
use orb_ui::skin::{Ctl, Skin};

/// The Theme Viewer's list of blits (task-187, `theme_view::ops`, removed by task-267), kept
/// as the reference the classic skin is compared with: the same bitmap, frame and place for
/// every piece of the panel, with the 1x sizes of the built-in kit.
mod viewer {
    use orb_ui::behavior;
    use orb_ui::model::{Panel, peak_meter_frame};
    use orb_ui::skin::bitmaps::{Op, digits, handle_top, knob_frame};

    fn knob(o: &mut Vec<Op>, p: &Panel, id: &str, strip: (u16, u16, i32, i32), x: i32, y: i32) {
        let (bmp, n, w, h) = strip;
        o.push(Op::h(bmp, n, knob_frame(n, p.knob(id)), x, y, w, h));
    }

    fn fader(o: &mut Vec<Op>, p: &Panel, id: &str, track: (u16, i32), x: i32, y: i32) {
        let (bmp, h) = track;
        o.push(Op::pic(bmp, x, y, 15, h));
        o.push(Op::pic(
            12031,
            x,
            y + handle_top(p.fader(id), h, 11),
            15,
            11,
        ));
    }

    fn display(o: &mut Vec<Op>, value: Option<i32>, n: usize, x: i32, y: i32) {
        for (i, f) in digits(value, n).into_iter().enumerate() {
            o.push(Op::h(12015, 11, f, x + 9 * i as i32, y, 9, 16));
        }
    }

    fn toggle(o: &mut Vec<Op>, bmp: u16, on: bool, x: i32, y: i32, w: i32, h: i32) {
        o.push(Op::h(bmp, 2, on as u16, x, y, w, h));
    }

    fn arrows(o: &mut Vec<Op>, x: i32, y: i32) {
        o.push(Op::h(12019, 2, 0, x, y, 11, 9));
        o.push(Op::h(12020, 2, 0, x, y + 9, 11, 9));
    }

    const KNOB_SMALL: (u16, u16, i32, i32) = (12552, 32, 18, 19);
    const KNOB_303: (u16, u16, i32, i32) = (12507, 54, 30, 30);
    const KNOB_808_RED: (u16, u16, i32, i32) = (12521, 36, 13, 13);
    const KNOB_808_WHITE: (u16, u16, i32, i32) = (12522, 36, 13, 13);
    const KNOB_909: (u16, u16, i32, i32) = (12565, 41, 12, 12);
    const TR909_SELECTED_LED: [(i32, i32); 12] = [
        (7, 52),
        (29, 76),
        (71, 76),
        (113, 76),
        (156, 76),
        (198, 76),
        (240, 76),
        (267, 76),
        (282, 76),
        (309, 76),
        (324, 76),
        (351, 76),
    ];

    pub fn ops(p: &Panel, now: f64) -> Vec<Op> {
        let mut o = Vec::new();
        o.push(Op::pic(12062, 0, 0, 16, 510));
        o.push(Op::pic(12062, 608, 0, 16, 510));
        o.push(Op::pic(12057, 16, 0, 592, 50));
        for r in 0..4 {
            let y = 50 + 115 * r as i32;
            o.push(Op::pic(12058, 16, y, 70, 115));
            o.push(Op::h(12068, 2, (p.focus == r) as u16, 86, y, 4, 115));
            o.push(Op::pic([12500, 12500, 12520, 12564][r], 90, y, 366, 115));
            o.push(Op::pic(12060, 456, y, 70, 115));
        }
        o.push(Op::pic(12061, 526, 50, 82, 98));
        o.push(Op::pic(12064, 526, 148, 82, 111));
        o.push(Op::pic(12063, 526, 259, 82, 93));
        o.push(Op::pic(12065, 526, 352, 82, 65));
        o.push(Op::pic(12569, 526, 417, 82, 93));
        // the transport
        let t = &p.transport;
        knob(&mut o, p, "transport.shuffle", KNOB_SMALL, 27, 13);
        o.push(Op::h(12041, 3, 0, 55, 2, 11, 11));
        o.push(Op::h(12041, 3, 0, 90, 2, 11, 11));
        display(&mut o, p.display("transport.tempo"), 3, 60, 27);
        arrows(&mut o, 89, 26);
        o.push(Op::h(12016, 2, t.song_mode as u16, 177, 2, 26, 19));
        toggle(&mut o, 12559, t.playing, 112, 21, 32, 25);
        toggle(&mut o, 12560, false, 144, 21, 32, 25);
        toggle(&mut o, 12561, false, 176, 21, 32, 25);
        toggle(&mut o, 12562, false, 208, 21, 32, 25);
        toggle(&mut o, 12563, t.recording, 240, 21, 32, 25);
        display(&mut o, p.display("transport.bar"), 3, 287, 27);
        arrows(&mut o, 316, 26);
        toggle(&mut o, 12551, t.looping, 331, 2, 17, 21);
        display(&mut o, p.display("transport.loop_start"), 3, 345, 27);
        arrows(&mut o, 374, 26);
        display(&mut o, p.display("transport.loop_length"), 3, 388, 27);
        arrows(&mut o, 417, 26);
        // the pattern sections
        for r in 0..4 {
            let y = 50 + 115 * r as i32;
            let row = &p.rows[r];
            let s = format!("pattern{}", r + 1);
            toggle(&mut o, 12556, row.enabled, 20, y + 4, 13, 14);
            let lit = row.lit_button().map_or(0, |b| b as u16 + 1);
            o.push(Op::v(12554, 9, lit, 17, y + 19, 68, 40));
            o.push(Op::v(12555, 4, row.bank.min(3) as u16, 17, y + 58, 68, 30));
            let sh = p.toggle_on(&format!("{s}.shuffle"));
            toggle(&mut o, 12571, sh, 19, y + 93, 18, 20);
            display(&mut o, p.display(&format!("{s}.steps")), 2, 52, y + 95);
            arrows(&mut o, 72, y + 94);
        }
        // the 303s
        for k in 0..2 {
            let y = 50 + 115 * k;
            let s = format!("tb303_{}", k + 1);
            let id = |n: &str| format!("{s}.{n}");
            let wave = p.switch_position(&id("wave")) as u16;
            o.push(Op::h(12504, 2, wave, 114, y + 10, 16, 12));
            for (i, n) in ["tune", "cutoff", "reso", "envmod", "decay", "accent"]
                .into_iter()
                .enumerate()
            {
                knob(&mut o, p, &id(n), KNOB_303, 162 + 33 * i as i32, y + 12);
            }
            display(&mut o, p.display(&id("edit_step")), 2, 420, y + 25);
            o.push(Op::h(12510, 2, 0, 110, y + 74, 15, 8));
            let led = |o: &mut Vec<Op>, bmp: u16, name: &str, x: i32, ly: i32| {
                let lit = p.led_lit(&id(name), now) as u16;
                o.push(Op::h(bmp, 2, lit, x, y + ly, 8, 8));
            };
            led(&mut o, 12506, "pitch_led", 113, 66);
            o.push(Op::h(12502, 2, 0, 108, y + 96, 17, 10));
            for (note, x) in [
                (0, 142),
                (2, 163),
                (4, 184),
                (5, 205),
                (7, 226),
                (9, 247),
                (11, 268),
                (12, 289),
            ] {
                o.push(Op::h(12501, 2, 0, x, y + 89, 9, 16));
                led(&mut o, 12505, &format!("key{note}_led"), x, 81);
            }
            for (note, x) in [(1, 153), (3, 174), (6, 216), (8, 237), (10, 258)] {
                o.push(Op::h(12509, 2, 0, x, y + 63, 8, 15));
                led(&mut o, 12506, &format!("key{note}_led"), x - 1, 55);
            }
            for (name, x) in [("down", 314), ("up", 339), ("accent", 364), ("slide", 389)] {
                o.push(Op::h(12501, 2, 0, x, y + 89, 9, 16));
                led(&mut o, 12505, &format!("{name}_led"), x, 81);
            }
            led(&mut o, 12505, "note_led", 336, 59);
            led(&mut o, 12505, "rest_led", 374, 59);
            o.push(Op::h(12502, 2, 0, 384, y + 58, 17, 10));
            o.push(Op::h(12502, 2, 0, 418, y + 62, 17, 10));
            o.push(Op::h(12503, 2, 0, 415, y + 90, 25, 17));
        }
        // the 808
        let (x0, y0) = (90, 280);
        let red = [
            "ac", "bd", "sd", "lt", "mt", "ht", "rs", "cp", "cb", "cy", "oh", "ch",
        ];
        for (i, n) in red.into_iter().enumerate() {
            let id = format!("tr808.{n}_level");
            knob(
                &mut o,
                p,
                &id,
                KNOB_808_RED,
                x0 + 14 + 22 * i as i32,
                y0 + 9,
            );
        }
        for (n, x, y) in [
            ("bd_tone", 36, 31),
            ("bd_decay", 36, 54),
            ("sd_tone", 58, 31),
            ("sd_snappy", 58, 54),
            ("lt_tuning", 80, 31),
            ("mt_tuning", 102, 31),
            ("ht_tuning", 124, 31),
            ("cy_tone", 212, 31),
            ("cy_decay", 212, 54),
            ("oh_decay", 234, 54),
        ] {
            let id = format!("tr808.{n}");
            knob(&mut o, p, &id, KNOB_808_WHITE, x0 + x, y0 + y);
        }
        for (slot, x) in [(3, 81), (4, 103), (5, 125), (6, 146), (7, 169)] {
            let f = p.switch_position(&format!("tr808.slot{slot}_switch")) as u16;
            o.push(Op::h(12523, 2, f, x0 + x, y0 + 55, 11, 17));
        }
        let v = (127 * p.inst808 as i32 + 5) / 11;
        let f = knob_frame(12, behavior::knob_from_engine(v));
        o.push(Op::h(12525, 12, f, x0 + 306, y0 + 41, 29, 31));
        for i in 0..16 {
            if p.steps808[p.inst808.min(11)][i] != (p.playing_step808 == Some(i)) {
                let x = x0 + 16 + 16 * i as i32;
                o.push(Op::h(12524, 4, (i / 4) as u16, x, y0 + 91, 11, 5));
            }
        }
        // the 909
        let (x0, y0) = (90, 395);
        for (n, x, y) in [
            ("ac_level", 9, 28),
            ("bd_tune", 30, 28),
            ("bd_attack", 30, 54),
            ("bd_level", 50, 28),
            ("bd_decay", 50, 54),
            ("sd_tune", 72, 28),
            ("sd_tone", 72, 54),
            ("sd_level", 92, 28),
            ("sd_snappy", 92, 54),
            ("lt_tune", 114, 28),
            ("lt_decay", 114, 54),
            ("lt_level", 134, 28),
            ("mt_tune", 156, 28),
            ("mt_decay", 156, 54),
            ("mt_level", 176, 28),
            ("ht_tune", 198, 28),
            ("ht_decay", 198, 54),
            ("ht_level", 218, 28),
            ("rs_level", 240, 28),
            ("cp_level", 260, 28),
            ("ch_decay", 282, 54),
            ("hh_level", 302, 28),
            ("oh_decay", 302, 54),
            ("cc_level", 324, 28),
            ("cc_tune", 324, 54),
            ("rc_level", 344, 28),
            ("rc_tune", 344, 54),
            ("flam", 9, 76),
        ] {
            knob(&mut o, p, &format!("tr909.{n}"), KNOB_909, x0 + x, y0 + y);
        }
        let fl = p.toggle_on("tr909.flam_button");
        toggle(&mut o, 12568, fl, x0 + 8, y0 + 93, 14, 14);
        let (lx, ly) = TR909_SELECTED_LED[p.inst909.min(11)];
        o.push(Op::pic(12567, x0 + lx, y0 + ly, 6, 6));
        for i in 0..16 {
            let mut v = p.steps909[p.inst909.min(11)][i];
            if p.playing_step909 == Some(i) {
                v = if v == 0 { 2 } else { 0 };
            }
            if v > 0 {
                let f = if v >= 3 { 3 } else { v as u16 - 1 };
                o.push(Op::h(12566, 4, f, x0 + 30 + 21 * i as i32, y0 + 86, 14, 8));
            }
        }
        // the mixer channels
        for c in 0..4 {
            let (x0, y0) = (456, 50 + 115 * c);
            let id = |n: &str| format!("mixer{}.{n}", c + 1);
            toggle(
                &mut o,
                12550,
                p.toggle_on(&id("mute")),
                x0 + 3,
                y0 + 4,
                17,
                15,
            );
            let m = p.meters.get(&id("meter")).copied().unwrap_or(0);
            let f = peak_meter_frame(m, 4);
            o.push(Op::h(12558, 4, f, x0 + 51, y0 + 4, 14, 15));
            toggle(
                &mut o,
                12040,
                p.toggle_on(&id("dist")),
                x0 + 42,
                y0 + 19,
                24,
                11,
            );
            toggle(
                &mut o,
                12040,
                p.toggle_on(&id("pcf")),
                x0 + 42,
                y0 + 37,
                24,
                11,
            );
            toggle(
                &mut o,
                12040,
                p.toggle_on(&id("comp")),
                x0 + 42,
                y0 + 55,
                24,
                11,
            );
            knob(&mut o, p, &id("pan"), KNOB_SMALL, x0 + 10, y0 + 25);
            fader(&mut o, p, &id("level"), (12032, 54), x0 + 10, y0 + 59);
            knob(&mut o, p, &id("delay"), KNOB_SMALL, x0 + 42, y0 + 79);
        }
        // the master and the effects
        let meter = |id: &str, n: u16| peak_meter_frame(p.meters.get(id).copied().unwrap_or(0), n);
        for (id, x) in [("master.meter_l", 542), ("master.meter_r", 576)] {
            o.push(Op::v(12557, 11, meter(id, 11), x, 74, 15, 56));
        }
        fader(&mut o, p, "master.fader", (12032, 54), 559, 75);
        toggle(&mut o, 12040, p.toggle_on("master.comp"), 568, 131, 24, 11);
        for (s, y) in [("pcf", 148), ("delay", 259), ("dist", 352), ("comp", 417)] {
            let dy = if s == "pcf" { 2 } else { 3 };
            let on = p.toggle_on(&format!("{s}.on"));
            toggle(&mut o, 12066, on, 529, y + dy, 16, 16);
            let f = meter(&format!("{s}.meter"), 4);
            o.push(Op::h(12067, 4, f, 590, y + dy, 13, 16));
        }
        display(&mut o, p.display("pcf.pattern"), 2, 536, 171);
        arrows(&mut o, 555, 170);
        let mode = p.switch_position("pcf.mode") as u16;
        o.push(Op::h(12017, 2, mode, 575, 169, 9, 18));
        for (i, n) in ["freq", "q", "amt", "dec"].into_iter().enumerate() {
            fader(
                &mut o,
                p,
                &format!("pcf.{n}"),
                (12030, 45),
                530 + 19 * i as i32,
                203,
            );
        }
        display(&mut o, p.display("delay.steps"), 2, 535, 282);
        arrows(&mut o, 555, 281);
        let tri = p.switch_position("delay.triplet") as u16;
        o.push(Op::h(12017, 2, tri, 575, 281, 9, 18));
        knob(&mut o, p, "delay.pan", KNOB_SMALL, 538, 317);
        knob(&mut o, p, "delay.feedback", KNOB_SMALL, 579, 317);
        knob(&mut o, p, "dist.amount", KNOB_SMALL, 538, 380);
        knob(&mut o, p, "dist.shape", KNOB_SMALL, 579, 380);
        o.push(Op::v(12570, 2, 0, 534, 442, 63, 10));
        for i in 0..orb_ui::model::gr_lit(p.gr_byte) {
            let x = 7 * i as i32;
            o.push(Op {
                sx: x,
                ..Op::v(12570, 2, 1, 534 + x, 442, 7, 10)
            });
        }
        knob(&mut o, p, "comp.ratio", KNOB_SMALL, 538, 473);
        knob(&mut o, p, "comp.threshold", KNOB_SMALL, 579, 473);
        o
    }
}

/// A small deterministic generator for the panel states.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }

    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
}

/// A panel in a random state: knobs, faders, switches, toggles, steps with a running light,
/// meters, the compressor's gain, the selections, the focus, the banks and patterns, the song
/// mode's displays.
fn random_panel(seed: u64) -> Panel {
    let mut g = Lcg(seed);
    let mut p = Panel::new();
    let skin = ClassicSkin::builtin();
    for c in &skin.layout().controls {
        match c.kind.as_str() {
            "knob" if c.id != SELECTOR_808 => p.set_knob(&c.id, g.below(1001) as i32),
            "fader" => p.set_fader(&c.id, g.below(1000) as i32),
            "hswitch" | "vswitch" => p.set_switch(&c.id, g.below(2) as u8),
            "toggle" | "radio" if g.below(2) == 1 && !c.id.starts_with("transport") => {
                p.press(&c.id, 0.0)
            }
            "meter" if !c.name().starts_with("gr") => {
                p.meters.insert(c.id.clone(), g.below(128) as u8);
            }
            _ => {}
        }
    }
    for i in 0..12 {
        for s in 0..16 {
            p.steps808[i][s] = g.below(2) == 1;
            p.steps909[i][s] = g.below(6) as u8;
        }
    }
    p.inst808 = g.below(12) as usize;
    p.inst909 = g.below(12) as usize;
    p.playing_step808 = (g.below(2) == 1).then(|| g.below(16) as usize);
    p.playing_step909 = (g.below(2) == 1).then(|| g.below(16) as usize);
    p.gr_byte = g.below(128) as u8;
    p.focus = g.below(4) as usize;
    for r in 0..4 {
        p.rows[r].bank = g.below(4) as u8;
        p.rows[r].pattern = g.below(32) as u8;
        p.rows[r].enabled = g.below(2) == 1;
    }
    p.transport.song_mode = g.below(2) == 1;
    p.transport.playing = g.below(2) == 1;
    p.transport.recording = g.below(2) == 1;
    p.transport.looping = g.below(2) == 1;
    p
}

/// The classic skin draws what the Theme Viewer drew, pixel for pixel: the new panel, and 40
/// random states.
#[test]
fn the_classic_skin_draws_what_the_viewer_drew() {
    let skin = ClassicSkin::builtin();
    let b = Bitmaps::builtin();
    let mut states = vec![Panel::new()];
    states.extend((1..=40).map(random_panel));
    for (i, p) in states.iter().enumerate() {
        let want = compose(b, &viewer::ops(p, 0.0));
        let got = skin.compose_panel(p, 0.0);
        assert_eq!(got.size, [624, 510]);
        let diff = got
            .pixels
            .iter()
            .zip(&want.pixels)
            .position(|(a, b)| a != b);
        assert_eq!(
            diff.map(|k| (k % W, k / W)),
            None,
            "state {i}: the first differing pixel"
        );
    }
}

/// Task-267, criterion 7: a set at 2x (the built-in bitmaps upscaled by nearest) composes the
/// 1x image upscaled 2x, for several states: places, frame cells, digit pitches, the fader
/// handle and the gain-reduction LEDs all scale by the set's N.
#[test]
fn a_2x_set_composes_the_1x_image_upscaled() {
    let one = Bitmaps::builtin();
    let images: HashMap<u16, ColorImage> = IDS
        .iter()
        .map(|&id| (id, upscale(one.get(id).unwrap(), 2)))
        .collect();
    let two = Bitmaps::from_images(images, Vec::new());
    assert_eq!(two.error, None);
    assert_eq!(two.scale(), 2);
    let skin2 = ClassicSkin::new(orb_ui::skin::classic::CLASSIC_TSV, two).unwrap();
    assert_eq!(skin2.scale(), 2);
    let skin1 = ClassicSkin::builtin();
    for seed in [0, 3, 7, 11] {
        let p = if seed == 0 {
            Panel::new()
        } else {
            random_panel(seed)
        };
        let want = upscale(&skin1.compose_panel(&p, 0.0), 2);
        let got = skin2.compose_panel(&p, 0.0);
        assert_eq!(got.size, [1248, 1020]);
        assert!(got.pixels == want.pixels, "state {seed}");
    }
    // the control rectangles and hit areas stay in panel units
    let l1 = skin1.layout();
    let l2 = skin2.layout();
    assert_eq!(l2.size, l1.size);
    assert_eq!(l2.controls.len(), l1.controls.len());
}

/// A set whose sizes are no whole scale is refused with a clear error, and the built-in set
/// is drawn instead; a missing bitmap of a 2x set is the built-in one upscaled.
#[test]
fn a_set_of_no_whole_scale_is_refused() {
    let one = Bitmaps::builtin();
    let mut images: HashMap<u16, ColorImage> = IDS
        .iter()
        .map(|&id| (id, upscale(one.get(id).unwrap(), 2)))
        .collect();
    let missing = images
        .remove(&12031)
        .map(|_| vec![(12031, "missing".to_string())]);
    let b = Bitmaps::from_images(images.clone(), missing.unwrap());
    assert_eq!(b.scale(), 2);
    assert_eq!(b.get(12031).unwrap().size, [30, 22]);
    // one bitmap at 1x in a 2x set: its odd side breaks the rule
    images.insert(12031, one.get(12031).unwrap().clone());
    let b = Bitmaps::from_images(images, Vec::new());
    let e = b.error.clone().expect("an error");
    assert!(e.contains("12031") && e.contains("15 x 11"), "{e}");
    assert_eq!(b.scale(), 1);
    assert_eq!(b.get(12057).unwrap().size, [592, 50]);
}

#[test]
fn the_builtin_kit_has_all_58_bitmaps_at_their_sizes() {
    let b = Bitmaps::builtin();
    assert!(b.fallbacks.is_empty(), "{:?}", b.fallbacks);
    for (id, w, h) in [
        (12015, 109, 16),
        (12057, 592, 50),
        (12062, 16, 510),
        (12507, 1674, 30),
        (12552, 608, 19),
        (12554, 68, 504),
        (12557, 15, 792),
        (12571, 37, 20),
    ] {
        assert_eq!(b.get(id).unwrap().size, [w, h], "{id}");
    }
    assert_eq!(IDS.len(), 58);
    assert_eq!(b.scale(), 1);
}

/// A JPEG of `w` x `h` in one color.
fn jpeg(w: u32, h: u32, rgb: [u8; 3]) -> Vec<u8> {
    let img = image::RgbImage::from_pixel(w, h, image::Rgb(rgb));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
    out.into_inner()
}

fn compose_with(b: Bitmaps, p: &Panel) -> ColorImage {
    ClassicSkin::new(orb_ui::skin::classic::CLASSIC_TSV, b)
        .unwrap()
        .compose_panel(p, 0.0)
}

#[test]
fn a_mods_bitmaps_fall_back_one_by_one() {
    let red = jpeg(1674, 30, [255, 0, 0]);
    // 32 frames of 4 x 6 (the built-in knobs are 18 x 19)
    let odd = jpeg(32 * 5 - 1, 6, [0, 0, 255]);
    let b = Bitmaps::from_files(|name| match name {
        "12507.jpg" => Some(red.clone()),
        "12552.jpg" => Some(odd.clone()),
        "12015.jpg" => Some(b"\xFF\xD8not a jpeg".to_vec()),
        "12057.jpg" => None,
        n => rbm::standard::builtin_file(n),
    });
    let ids: Vec<u16> = b.fallbacks.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, [12015, 12057]);
    let builtin = Bitmaps::builtin();
    assert_eq!(
        b.get(12015).unwrap().pixels,
        builtin.get(12015).unwrap().pixels
    );
    let img = compose_with(b, &Panel::new());
    // the mod's knob strip where the 303's knobs are
    let px = img.pixels[(62 + 15) * W + 195 + 15];
    assert!(px.r() > 200 && px.g() < 60, "{px:?}");
    // the odd strip draws its own frame size, 4 x 6, at the knob's corner
    let px = img.pixels[(13 + 2) * W + 27 + 1];
    assert!(px.b() > 200, "{px:?}");
    let bg = builtin.get(12057).unwrap();
    let (x, y) = (27 + 10, 13 + 10);
    assert_eq!(img.pixels[y * W + x], bg.pixels[y * bg.size[0] + (x - 16)]);
}

/// Review round 4 of task-187, kept: a mod's strip of another size than the built-in one is
/// drawn at its own frame size (ReBirthday's Loop switch 81 x 15: two frames of 40 x 15).
#[test]
fn a_strip_of_another_size_draws_its_own_frames() {
    let img = image::RgbImage::from_fn(81, 15, |x, _| {
        if x == 40 {
            image::Rgb([255, 255, 255])
        } else if x < 40 {
            image::Rgb([255, 0, 0])
        } else {
            image::Rgb([0, 255, 0])
        }
    });
    let mut j = std::io::Cursor::new(Vec::new());
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut j, 100)
        .encode_image(&image::DynamicImage::ImageRgb8(img))
        .unwrap();
    let looping = j.into_inner();
    let b = Bitmaps::from_files(|name| match name {
        "12551.jpg" => Some(looping.clone()),
        n => rbm::standard::builtin_file(n),
    });
    let mut p = Panel::new();
    p.transport.looping = true;
    let img = compose_with(b, &p);
    let px = |x: usize, y: usize| img.pixels[y * W + x];
    let green = |c: Color32| c.g() > 200 && c.r() < 60 && c.b() < 60;
    assert!(green(px(331, 2)) && green(px(370, 16)), "{:?}", px(370, 16));
}

/// The TR-909 lights the selected instrument's LED (12567) at its dot: BD's at (119, 471), as
/// the original does, and it moves with the selection.
#[test]
fn the_909_lights_its_selected_instrument() {
    let b = Bitmaps::builtin();
    let led = b.get(12567).unwrap();
    let skin = ClassicSkin::builtin();
    let shows = |img: &ColorImage, x: usize, y: usize| {
        (0..6).all(|j| (0..6).all(|i| img.pixels[(y + j) * W + x + i] == led.pixels[j * 6 + i]))
    };
    let mut p = Panel::new();
    p.inst909 = 1;
    let img = skin.compose_panel(&p, 0.0);
    assert!(shows(&img, 119, 471));
    p.inst909 = 0;
    let img = skin.compose_panel(&p, 0.0);
    assert!(shows(&img, 97, 447) && !shows(&img, 119, 471));
}

/// The 808's instrument selector of the original (a knob, span 40, docs/ui_spec.md section 6):
/// it shows `(127 * i + 5) / 11` for the instrument i and selects `(11 * v + 63) / 127` of its
/// value v; it sends nothing to the engine.
#[test]
fn the_808_selector_knob_selects_the_instrument() {
    let mut p = Panel::new();
    for i in 0..12 {
        p.inst808 = i;
        let v = behavior::knob_send(p.knob(SELECTOR_808));
        assert_eq!(v, (127 * i as i32 + 5) / 11, "{i}");
        p.set_knob(SELECTOR_808, p.knob(SELECTOR_808));
        assert_eq!(p.inst808, i);
    }
    p.events.clear();
    p.set_knob(SELECTOR_808, 0);
    assert_eq!(p.inst808, 0);
    p.set_knob(SELECTOR_808, 1000);
    assert_eq!(p.inst808, 11);
    assert!(p.events.is_empty());
}

fn controls(skin: &dyn Skin) -> Vec<Ctl> {
    skin.layout().controls.clone()
}

fn ctl<'a>(cs: &'a [Ctl], id: &str, kind: &str) -> &'a Ctl {
    cs.iter()
        .find(|c| c.id == id && c.kind == kind)
        .unwrap_or_else(|| panic!("{id} {kind}"))
}

/// The classic skin has every control of the modern skin but the modern-only ones (the 808's
/// mod window and the modern art's two areas), and the original's 808 selector besides.
#[test]
fn the_classic_skin_has_the_modern_controls() {
    let modern = controls(&*orb_ui::skin::svg::modern());
    let classic = controls(&*ClassicSkin::builtin());
    let key = |c: &Ctl| (c.id.clone(), c.kind.clone());
    let m: std::collections::BTreeSet<_> = modern
        .iter()
        .filter(|c| c.kind != "modview" && c.kind != "area")
        .map(key)
        .collect();
    let mut k: std::collections::BTreeSet<_> = classic.iter().map(key).collect();
    assert!(k.remove(&(SELECTOR_808.to_string(), "knob".to_string())));
    assert_eq!(m, k);
}

/// The 808's step strip, `(x - 16) / 16` of the step view (16, 91, 256, 17) of the section
/// (`sub_442CDA`, `sub_44111F`); the 909's, `(x - 29) / 21` of (29, 86, 336, 16)
/// (`sub_446E4E`, `sub_444FB3`).
fn original_step(machine808: bool, x: i32, y: i32) -> Option<usize> {
    let (vx, vy, w, h, cell) = if machine808 {
        (16, 91, 256, 17, 16)
    } else {
        (29, 86, 336, 16, 21)
    };
    if !(vx..vx + w).contains(&x) || !(vy..vy + h).contains(&y) {
        return None;
    }
    let i = (x - vx) / cell;
    (i < 16).then_some(i as usize)
}

/// The 808's selection (`sub_4433EE`): the legend (10, 71)..(366, 85), `(x - 10) / 22` under
/// 12; the upper labels (78, 43)..(184, 55), `(x - 78) / 22 + 3`.
fn original_808(x: i32, y: i32) -> Option<usize> {
    if (10..366).contains(&x) && (71..85).contains(&y) {
        let i = (x - 10) / 22;
        return (i < 12).then_some(i as usize);
    }
    if (78..184).contains(&x) && (43..55).contains(&y) {
        let i = (x - 78) / 22 + 3;
        return (i < 12).then_some(i as usize);
    }
    None
}

/// The 909's selection (`sub_447500`): the legend (4, 4)..(366, 18) and the name row
/// (26, 70)..(366, 85) by `dx`, and the AC box (4, 44)..(26, 62).
fn original_909(x: i32, y: i32) -> Option<usize> {
    let legend = (4..366).contains(&x) && (4..18).contains(&y);
    let names = (26..366).contains(&x) && (70..85).contains(&y);
    if legend || names {
        let i = if x < 26 {
            0
        } else if x < 236 {
            (x - 26) / 42 + 1
        } else {
            (x - 236) / 21 + 6
        };
        return (i <= 11).then_some(i as usize);
    }
    ((4..26).contains(&x) && (44..62).contains(&y)).then_some(0)
}

/// Task-267, criterion 4: the classic skin's hit areas are the original's, read in IDA
/// (docs/ui_spec.md sections 2, 4, 6, 7, 9): every whole pixel of both drum machines and 30
/// around them for the step strips and the instrument selection, the pattern grid's buttons,
/// the PCF's 45-high faders, the knob spans.
#[test]
fn the_classic_hit_areas_are_the_originals() {
    let skin = ClassicSkin::builtin();
    let l = skin.layout();
    for (section, ox, oy) in [("tr808", 90, 280), ("tr909", 90, 395)] {
        let strip = l.strips.iter().find(|s| s.section == section).unwrap();
        for y in -30..145 {
            for x in -30..396 {
                let p = Pos2::new((ox + x) as f32 + 0.5, (oy + y) as f32 + 0.5);
                let step = strip
                    .contains(p)
                    .then(|| behavior::strip_step(strip.x, strip.cell(), p.x))
                    .flatten();
                assert_eq!(
                    step,
                    original_step(section == "tr808", x, y),
                    "{section} step {x} {y}"
                );
                let want = if section == "tr808" {
                    original_808(x, y)
                } else {
                    original_909(x, y)
                };
                assert_eq!(l.select_hit(section, p), want, "{section} select {x} {y}");
            }
        }
    }
    // the pattern grid (`sub_41A5B0`): button `4 * row + col + 1` of the 68 x 40 view at
    // (1, 19) of the section; its last row is under the bank row (1, 58, 68, 30)
    let cs = &l.controls;
    for r in 0..4 {
        let (ox, oy) = (16, 50 + 115 * r);
        for dy in 0..39 {
            for dx in 0..68 {
                let p = Pos2::new((ox + 1 + dx) as f32 + 0.5, (oy + 19 + dy) as f32 + 0.5);
                let hit: Vec<_> = cs.iter().filter(|c| c.contains(p)).collect();
                let b = behavior::pattern_grid_button(dx as f32, dy as f32);
                assert_eq!(hit.len(), 1, "{r} {dx} {dy}");
                assert_eq!(hit[0].id, format!("pattern{}.pattern{b}", r + 1));
            }
        }
    }
    // the faders: CBitmapFaderView is as high as its track (`sub_469240`), so the PCF's are
    // 45 (12030) with a travel of 34 and k = 10000 / 34 (0x469708); the others 54, 43
    for id in ["pcf.freq", "pcf.q", "pcf.amt", "pcf.dec"] {
        let f = ctl(cs, id, "fader").fader();
        assert_eq!(
            (f.track, f.handle, f.travel(), f.k()),
            (45, 11, 34, 294),
            "{id}"
        );
    }
    for id in ["mixer1.level", "mixer4.level", "master.fader"] {
        let f = ctl(cs, id, "fader").fader();
        assert_eq!(
            (f.track, f.handle, f.travel(), f.k()),
            (54, 11, 43, 232),
            "{id}"
        );
    }
    // the knob spans (docs/ui_spec.md section 2)
    for c in cs.iter().filter(|c| c.kind == "knob") {
        let want = match c.section.as_str() {
            _ if c.id == SELECTOR_808 => 40.0,
            "tb303_1" | "tb303_2" => 62.0,
            "tr808" => 36.0,
            "tr909" => 41.0,
            _ => 64.0,
        };
        assert_eq!(c.knob_span(), want, "{}", c.id);
    }
    // a sample of the view rectangles of sections 3 to 9 (panel pixels)
    for (id, kind, r) in [
        ("transport.play", "trigger", (112.0, 21.0, 32.0, 25.0)),
        ("transport.loop", "toggle", (331.0, 2.0, 17.0, 21.0)),
        ("pattern1.on", "toggle", (20.0, 54.0, 13.0, 14.0)),
        ("pattern2.bankC", "grid", (51.0, 223.0, 17.0, 30.0)),
        ("tb303_2.key1", "trigger", (153.0, 228.0, 8.0, 15.0)),
        ("tb303_1.step", "trigger", (415.0, 140.0, 25.0, 17.0)),
        ("tr808.slot7_switch", "vswitch", (259.0, 335.0, 11.0, 17.0)),
        (SELECTOR_808, "knob", (396.0, 321.0, 29.0, 31.0)),
        ("tr909.flam_button", "toggle", (98.0, 488.0, 14.0, 14.0)),
        ("mixer3.comp", "radio", (498.0, 335.0, 24.0, 11.0)),
        ("master.comp", "radio", (568.0, 131.0, 24.0, 11.0)),
        ("pcf.mode", "vswitch", (575.0, 169.0, 9.0, 18.0)),
        ("delay.steps_down", "repeat", (555.0, 290.0, 11.0, 9.0)),
    ] {
        let c = ctl(cs, id, kind);
        assert_eq!((c.x, c.y, c.w, c.h), r, "{id}");
    }
    let logo = l.logo.as_ref().unwrap();
    assert_eq!((logo.x, logo.y, logo.w, logo.h), (474.0, 1.0, 134.0, 48.0));
}

/// The app headless, the panel at 1:1 in a 624 x 510 window (no menu bar).
struct Harness {
    ctx: egui::Context,
    app: App,
    time: f64,
    pos: Pos2,
}

impl Harness {
    fn new(skin: Option<Arc<dyn Skin>>) -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let mut app = App::new(Panel::new(), None);
        if let Some(s) = skin {
            app.set_skin(s);
        }
        let mut h = Self {
            ctx,
            app,
            time: 0.0,
            pos: Pos2::ZERO,
        };
        h.frame(vec![]);
        h
    }

    fn frame(&mut self, mut events: Vec<Event>) {
        self.time += 0.016;
        events.insert(0, Event::ModifiersChanged(Modifiers::NONE));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        out.textures_delta.clear();
    }

    fn move_to(&mut self, p: Pos2) {
        self.pos = p;
        self.frame(vec![Event::PointerMoved(p)]);
    }

    fn button(&mut self, pressed: bool) {
        let pos = self.pos;
        self.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }

    fn click_at(&mut self, p: Pos2) {
        self.move_to(p);
        self.button(true);
        self.button(false);
    }

    fn drag(&mut self, from: Pos2, to: Pos2) {
        self.move_to(from);
        self.button(true);
        for k in 1..=4 {
            self.move_to(from + (to - from) * (k as f32 / 4.0));
        }
        self.button(false);
    }
}

/// The panel's state, in a form two panels can be compared by (the model's maps print in no
/// fixed order): every control's value, the selections, the rows, the 303s, the events.
fn state(p: &Panel) -> String {
    let mut s = String::new();
    for c in controls(&*orb_ui::skin::svg::modern()) {
        let id = &c.id;
        s += &format!(
            "{id} k{} f{} s{} t{} d{:?} v{:?}\n",
            p.knob(id),
            p.fader(id),
            p.switch(id),
            p.toggle_on(id),
            p.display(id),
            p.value(id),
        );
    }
    s += &format!(
        "{:?} {:?} {} {} {} {:?} {:?} {:?} {:?}",
        p.transport,
        p.rows,
        p.focus,
        p.inst808,
        p.inst909,
        p.steps808,
        p.steps909,
        p.synth,
        p.events
    );
    s
}

/// One input on the control `id` of kind `kind`, done the same way in each skin at the
/// control's own place.
#[derive(Clone, Copy)]
enum Act {
    Click,
    /// A drag from the center, `dy` units up.
    DragUp(f32),
    /// A press `dy` units under the top, then a drag `up` units up (a fader's jump and drag).
    Fader(f32, f32),
}

fn act(h: &mut Harness, c: &Ctl, a: Act) {
    let center = Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0);
    match a {
        Act::Click => h.click_at(center),
        Act::DragUp(dy) => h.drag(center, center - Vec2::new(0.0, dy)),
        Act::Fader(dy, up) => {
            let p = Pos2::new(center.x, c.y + dy);
            h.drag(p, p - Vec2::new(0.0, up));
        }
    }
}

/// Task-267, criterion 4: the same input gives the same panel state in the modern and the
/// classic skin, for every control kind: a trigger, a toggle, a radio, the mode switch, a
/// two-position switch either way, a repeat and a wind button, a pattern and a bank button, a
/// knob of each span, a fader's jump and drag, both step strips, both drum machines' instrument
/// selection and a row's focus.
#[test]
fn the_same_input_gives_the_same_state_in_both_skins() {
    let classic: Arc<dyn Skin> = ClassicSkin::builtin();
    let mut m = Harness::new(None);
    let mut c = Harness::new(Some(classic.clone()));
    let mc = controls(&*orb_ui::skin::svg::modern());
    let cc = controls(&*classic);
    let probes: &[(&str, &str, Act)] = &[
        ("transport.play", "trigger", Act::Click),
        ("transport.stop", "trigger", Act::Click),
        ("transport.mode", "switch", Act::Click),
        ("transport.bar_up", "wind", Act::Click),
        ("transport.loop", "toggle", Act::Click),
        ("transport.tempo_up", "repeat", Act::Click),
        ("pattern2.bankC", "grid", Act::Click),
        ("pattern2.pattern6", "grid", Act::Click),
        ("pattern1.shuffle", "toggle", Act::Click),
        ("tb303_1.wave", "hswitch", Act::Click),
        ("tb303_1.key4", "trigger", Act::Click),
        ("tb303_1.accent", "trigger", Act::Click),
        ("tb303_1.cutoff", "knob", Act::DragUp(20.0)),
        ("tb303_2.accent", "knob", Act::DragUp(-10.0)),
        ("tr808.bd_tone", "knob", Act::DragUp(12.0)),
        ("tr808.slot4_switch", "vswitch", Act::Click),
        ("tr909.sd_level", "knob", Act::DragUp(15.0)),
        ("tr909.flam_button", "toggle", Act::Click),
        ("mixer2.mute", "toggle", Act::Click),
        ("mixer2.pcf", "radio", Act::Click),
        ("mixer3.pan", "knob", Act::DragUp(30.0)),
        ("mixer1.level", "fader", Act::Fader(5.0, 12.0)),
        ("master.fader", "fader", Act::Fader(40.0, 7.0)),
        ("delay.triplet", "vswitch", Act::Click),
        ("pcf.pattern_up", "repeat", Act::Click),
        ("comp.on", "toggle", Act::Click),
    ];
    for &(id, kind, a) in probes {
        let before = state(&m.app.panel);
        act(&mut m, ctl(&mc, id, kind), a);
        act(&mut c, ctl(&cc, id, kind), a);
        assert_ne!(
            state(&m.app.panel),
            before,
            "{id} {kind}: the input changes the state"
        );
        assert_eq!(state(&m.app.panel), state(&c.app.panel), "{id} {kind}");
    }
    // the step strips: a drag over steps 3..6 of each machine, at the middle of its cells
    for section in ["tr808", "tr909"] {
        let mut ends = Vec::new();
        for h in [&mut m, &mut c] {
            let l = h.app.skin().layout().clone();
            let s = l.strips.iter().find(|s| s.section == section).unwrap();
            let at = |i: f32| Pos2::new(s.x + s.cell() * (i + 0.5), s.y + s.h / 2.0);
            ends.push((at(2.0), at(5.0)));
        }
        let before = state(&m.app.panel);
        m.drag(ends[0].0, ends[0].1);
        c.drag(ends[1].0, ends[1].1);
        assert_ne!(state(&m.app.panel), before, "{section} strip");
        assert_eq!(state(&m.app.panel), state(&c.app.panel), "{section} strip");
    }
    // the instrument selection: the 808's CP on its legend, the 909's SD on its legend, in
    // each skin's row (the columns are the original's in both)
    for (section, x, inst) in [("tr808", 100.0 + 22.0 * 7.5, 7), ("tr909", 116.0 + 63.0, 2)] {
        for h in [&mut m, &mut c] {
            let l = h.app.skin().layout().clone();
            let a = l.selects.iter().find(|s| s.section == section).unwrap();
            h.click_at(Pos2::new(x, a.y + a.h / 2.0));
        }
        let (pm, pc) = (&m.app.panel, &c.app.panel);
        let got = if section == "tr808" {
            (pm.inst808, pc.inst808)
        } else {
            (pm.inst909, pc.inst909)
        };
        assert_eq!(got, (inst, inst), "{section}");
        assert_eq!(state(pm), state(pc), "{section} select");
    }
    // a row's focus: a press in the 808's row away from its controls
    m.click_at(Pos2::new(450.0, 300.0));
    c.click_at(Pos2::new(450.0, 300.0));
    assert_eq!(m.app.panel.focus, 2);
    assert_eq!(state(&m.app.panel), state(&c.app.panel), "focus");
}

/// The classic skin takes the 808 selector's drag through the controller (span 40): 20 units up
/// from AC turns it half way, to CP's neighbor.
#[test]
fn the_selector_knob_takes_a_drag() {
    let classic: Arc<dyn Skin> = ClassicSkin::builtin();
    let mut c = Harness::new(Some(classic.clone()));
    let cc = controls(&*classic);
    c.app.panel.inst808 = 0;
    act(&mut c, ctl(&cc, SELECTOR_808, "knob"), Act::DragUp(20.0));
    // p = 500, v = 64, instrument (11 * 64 + 63) / 127 = 6
    assert_eq!(c.app.panel.inst808, 6);
}

/// The panel image is composed only when what it shows changes.
#[test]
fn the_classic_panel_is_composed_only_on_a_change() {
    let skin = Arc::new(
        ClassicSkin::new(
            orb_ui::skin::classic::CLASSIC_TSV,
            Bitmaps::builtin().clone(),
        )
        .unwrap(),
    );
    let mut h = Harness::new(Some(skin.clone()));
    let n = skin.composed();
    assert!(n >= 1);
    h.frame(vec![]);
    h.frame(vec![]);
    assert_eq!(skin.composed(), n);
    h.app.panel.transport.looping = !h.app.panel.transport.looping;
    h.frame(vec![]);
    assert_eq!(skin.composed(), n + 1);
}

fn temp_settings(text: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("ui.toml");
    if let Some(t) = text {
        std::fs::write(&file, t).unwrap();
    }
    (dir, file)
}

/// View > Classic / Modern switches the main window's skin, saves the choice (`theme`) and the
/// next run takes it; Modern is the default; the classic skin follows the kit. The menu has the
/// two radio items directly in View (the user's review, 2026-11-21): no Themes submenu.
#[test]
fn view_menu_switches_and_is_kept() {
    let (_dir, file) = temp_settings(None);
    let mut h = Harness::new(None);
    h.app.settings = Settings::load_from(&file);
    assert_eq!(h.app.theme(), Theme::Modern);
    let modern_ptr = Arc::as_ptr(h.app.skin()) as *const u8;
    h.app.command(Cmd::SetTheme(Theme::Classic));
    h.frame(vec![]);
    assert!(
        h.app.skin().layout().control(SELECTOR_808).is_some(),
        "the classic skin"
    );
    assert_eq!(Settings::load_from(&file).theme, Theme::Classic);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("theme = \"classic\""), "{text}");
    // the input goes to the classic skin's places: its Loop switch at (331, 2)
    let looping = h.app.panel.transport.looping;
    h.click_at(Pos2::new(333.0, 4.0));
    assert_ne!(h.app.panel.transport.looping, looping);
    // another kit: the classic skin of that kit
    let before = Arc::as_ptr(h.app.skin()) as *const u8;
    h.app.doc.kit = Kit::Mod(PathBuf::from("/no/such/mod.rbm"));
    h.frame(vec![]);
    assert_ne!(Arc::as_ptr(h.app.skin()) as *const u8, before);
    h.app.doc.kit = Kit::Standard;
    h.frame(vec![]);
    h.app.command(Cmd::SetTheme(Theme::Modern));
    h.frame(vec![]);
    assert_eq!(Arc::as_ptr(h.app.skin()) as *const u8, modern_ptr);
    assert_eq!(Settings::load_from(&file).theme, Theme::Modern);
    // a new run with Classic saved starts classic
    let mut s = Settings::load_from(&file);
    s.set_theme(Theme::Classic);
    s.save().unwrap();
    let mut h = Harness::new(None);
    h.app.settings = Settings::load_from(&file);
    h.frame(vec![]);
    assert!(h.app.skin().layout().control(SELECTOR_808).is_some());
}

/// The crossfade (task-268): a switch fades the classic image in or out over the fade's time
/// while the input is already the target skin's; the end state draws one skin; a switch during
/// a fade turns around from the alpha it had.
#[test]
fn the_switch_crossfades_both_ways_and_reverses_from_the_current_alpha() {
    use orb_ui::skin::fade::FADE_SECONDS;
    let (_dir, file) = temp_settings(None);
    let mut h = Harness::new(None);
    h.app.settings = Settings::load_from(&file);
    assert_eq!(h.app.fading(), None);
    assert!(h.app.splash_in_frame(), "the modern panel's mod window");
    // to Classic: the target's layout takes the input at once, the alpha rises from 0
    h.app.command(Cmd::SetTheme(Theme::Classic));
    assert!(h.app.skin().layout().control(SELECTOR_808).is_some());
    h.frame(vec![]);
    let first = h.app.fading().expect("fading");
    assert!(
        h.app.splash_in_frame(),
        "the modern panel keeps its picture under the fade"
    );
    assert!(first < 0.01, "{first}");
    let mut last = first;
    while h.time < 0.4 * FADE_SECONDS {
        h.frame(vec![]);
        let a = h.app.fading().expect("fading");
        assert!(a >= last);
        last = a;
    }
    assert!(last > 0.0 && last < 1.0, "{last}");
    // reversed in the middle: the alpha falls from where it was, never jumps
    h.app.command(Cmd::SetTheme(Theme::Modern));
    assert!(h.app.skin().layout().control(SELECTOR_808).is_none());
    h.frame(vec![]);
    let a = h.app.fading().expect("fading");
    assert!(h.app.splash_in_frame());
    assert!((a - last).abs() < 0.05, "{a} vs {last}");
    let mut prev = a;
    for _ in 0..200 {
        h.frame(vec![]);
        match h.app.fading() {
            Some(a) => {
                assert!(h.app.splash_in_frame());
                assert!(a <= prev);
                prev = a;
            }
            None => break,
        }
    }
    // over: the modern skin alone, the classic one dropped
    assert_eq!(h.app.fading(), None);
    assert!(h.app.skin().layout().control(SELECTOR_808).is_none());
    // and back: Modern to Classic runs to alpha 1 and ends with the classic skin alone
    h.app.command(Cmd::SetTheme(Theme::Classic));
    for _ in 0..200 {
        h.frame(vec![]);
        if h.app.fading().is_some() {
            assert!(h.app.splash_in_frame());
        }
        if h.app.fading().is_none() {
            break;
        }
    }
    assert_eq!(h.app.fading(), None);
    assert!(h.app.skin().layout().control(SELECTOR_808).is_some());
}

/// An old settings file of the Theme Viewer's time (`theme_viewer_filter`) still loads, with
/// its other keys, the theme Modern; a save drops the old key and writes `theme`.
#[test]
fn an_old_settings_file_loads() {
    let (_dir, file) = temp_settings(Some(
        "startup_sound = false\nrecent = []\ntheme_viewer_filter = \"linear\"\n",
    ));
    let mut s = Settings::load_from(&file);
    assert!(!s.startup_sound);
    assert_eq!(s.theme, Theme::Modern);
    s.set_theme(Theme::Classic);
    s.save().unwrap();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(!text.contains("theme_viewer_filter"), "{text}");
    let s = Settings::load_from(&file);
    assert_eq!(s.theme, Theme::Classic);
    assert!(!s.startup_sound);
    // a value it does not know is skipped
    let (_dir, file) = temp_settings(Some("theme = \"neon\"\n"));
    assert_eq!(Settings::load_from(&file).theme, Theme::Modern);
}

/// The embedded standard mod repacked with `change` applied to its files, under another name.
fn repacked(dir: &Path, name: &str, change: impl Fn(&str, &mut Vec<u8>) -> bool) -> PathBuf {
    let r = rbm::Rbm::parse(rbm::standard::EMBEDDED.to_vec()).unwrap();
    let t = rbm::KeyTable::embedded(&r).unwrap();
    let mut u = rbm::pack::unpack(&r, &t).unwrap();
    let mut keep_entries = Vec::new();
    let mut keep_files = Vec::new();
    for (e, mut f) in u.manifest.files.drain(..).zip(u.files.drain(..)) {
        if change(&e.name, &mut f) {
            keep_entries.push(e);
            keep_files.push(f);
        }
    }
    u.manifest.files = keep_entries;
    u.manifest.info.name = name.into();
    let bytes = rbm::pack::pack(&u.manifest, &keep_files).unwrap();
    let path = dir.join(format!("{name}.rbm"));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// A mod's own bitmaps are drawn, and a mod without the panel bitmaps draws the built-in ones.
#[test]
fn the_classic_skin_of_a_mod_draws_its_bitmaps() {
    let dir = tempfile::tempdir().unwrap();
    let red = jpeg(592, 50, [255, 0, 0]);
    let with = repacked(dir.path(), "Red", |name, f| {
        if name == "12057.jpg" {
            *f = red.clone();
        }
        true
    });
    let without = repacked(dir.path(), "Bare", |name, _| !name.ends_with(".jpg"));
    let p = Panel::new();
    let img = ClassicSkin::of_kit(&Kit::Mod(with)).compose_panel(&p, 0.0);
    let px = img.pixels[5 * W + 300];
    assert!(px.r() > 200 && px.g() < 60, "{px:?}");
    let bare = ClassicSkin::of_kit(&Kit::Mod(without));
    assert_eq!(bare.bitmaps().fallbacks.len(), 58);
    assert!(
        bare.compose_panel(&p, 0.0).pixels == ClassicSkin::builtin().compose_panel(&p, 0.0).pixels
    );
}

/// Every mod of the user's third-party collection composes (`ORB_THIRDPARTY_MODS_DIR`, the
/// recrypted mods; skipped without it), at a whole scale.
#[test]
fn every_thirdparty_mod_composes() {
    let Some(dir) = orb_testkit::thirdparty_mods() else {
        return;
    };
    let mut mods = Vec::new();
    let mut stack = vec![dir.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rbm")) {
                mods.push(p);
            }
        }
    }
    mods.sort();
    assert!(!mods.is_empty(), "no .rbm in {}", dir.display());
    let mut p = Panel::new();
    p.transport.song_mode = true;
    p.transport.playing = true;
    for m in &mods {
        let skin = ClassicSkin::of_kit(&Kit::Mod(m.clone()));
        let b = skin.bitmaps();
        assert_eq!(b.error, None, "{}", m.display());
        assert_eq!(b.scale(), 1, "{}", m.display());
        let img = skin.compose_panel(&p, 0.0);
        assert_eq!(img.size, [W * b.scale(), bitmaps::H * b.scale()]);
        if !b.fallbacks.is_empty() {
            eprintln!("{}: {} built-in bitmaps", m.display(), b.fallbacks.len());
        }
    }
    eprintln!("{} mods composed", mods.len());
}
