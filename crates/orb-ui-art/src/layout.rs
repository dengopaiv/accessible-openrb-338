//! Where every control sits, and the composition of the full panel from the parts. The list is
//! written to `layout.tsv` and drawn into `panel.svg`. The sections and the controls' places
//! are the original's (docs/ui_spec.md sections 3 to 9) as the redesign of task-179 draws them
//! (the approved mockup; docs/ui_spec.md section 19 lists where it leaves the spec's tables).
//! A control's rectangle is its hit area; its part may draw beyond it (`Part::bleed`).

use std::collections::HashMap;

use crate::parts::{
    BANK_Y, BLACK_303, FG_PCF, FG_TR808, GR_COLORS, KEY_X, KEY_Y, KEYCAP, KNOB_SWEEP, KNOB_X_303,
    KNOBS, KNOBS_909, LEGEND_808, LOGO_X, LOGO_Y, MOD_PICTURE, MOD_VIEW_808, NAMES_808, NAMES_909,
    SEAM, UPPER_808, UPPER_CHIP_808, WHITE_303, chip_909, led_909,
};
use crate::svg::{Part, Svg, defs_for, n};

pub const PANEL_W: f32 = 624.0;
pub const PANEL_H: f32 = 510.0;

/// How a control is composed from parts.
pub enum Extra {
    /// One part at (x, y).
    Plain,
    /// `<knob>_body` and `<knob>_pointer` rotated by the value.
    Knob { design: usize, v: f32 },
    /// `fader_track` and `fader_handle` at `(1 - v) * 43` px from the top.
    Fader { v: f32 },
    /// `n` digit frames, centered in the display rectangle.
    Digits { n: usize, text: String },
}

pub struct Ctl {
    pub id: String,
    pub section: String,
    pub kind: &'static str,
    pub part: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub extra: Extra,
    /// The row's parameters in `layout.tsv` (`key=value`, space-separated; empty for none):
    /// the geometry and the state parts the app's skin reads (assets/ui/README.md).
    pub params: String,
}

pub struct Layout {
    /// Section backgrounds: part name and panel position.
    pub backgrounds: Vec<(String, f32, f32)>,
    /// The section of each background, in the same order (`logo` for the wordmark).
    pub sections: Vec<String>,
    pub ctls: Vec<Ctl>,
    /// What the app lays over the controls in the demo state: the selected instruments' marks
    /// and the glass over the mod's picture (part, panel position).
    pub overlays: Vec<(String, f32, f32)>,
    /// Static art drawn over the controls (part, panel position): `fg_tr808`, `fg_pcf`.
    pub foregrounds: Vec<(String, f32, f32)>,
}

struct Builder<'a> {
    sizes: &'a HashMap<String, (f32, f32)>,
    layout: Layout,
    section: String,
    ox: f32,
    oy: f32,
}

impl Builder<'_> {
    fn section(&mut self, name: &str, bg: &str, ox: f32, oy: f32) {
        self.section = name.to_string();
        self.ox = ox;
        self.oy = oy;
        self.layout.backgrounds.push((bg.to_string(), ox, oy));
        self.layout.sections.push(name.to_string());
    }

    /// Sets the parameters of the control pushed last.
    fn params(&mut self, params: String) {
        self.layout.ctls.last_mut().expect("a control").params = params;
    }

    fn size(&self, part: &str) -> (f32, f32) {
        *self
            .sizes
            .get(part)
            .unwrap_or_else(|| panic!("layout names a missing part {part}"))
    }

    fn push(&mut self, id: &str, kind: &'static str, part: &str, x: f32, y: f32, extra: Extra) {
        let (w, h) = match &extra {
            Extra::Knob { design, .. } => (KNOBS[*design].w, KNOBS[*design].h),
            _ => self.size(part),
        };
        self.layout.ctls.push(Ctl {
            id: format!("{}.{id}", self.section),
            section: self.section.clone(),
            kind,
            part: part.to_string(),
            x: self.ox + x,
            y: self.oy + y,
            w,
            h,
            extra,
            params: String::new(),
        });
    }

    fn part(&mut self, id: &str, kind: &'static str, part: &str, x: f32, y: f32) {
        self.push(id, kind, part, x, y, Extra::Plain);
    }

    fn knob(&mut self, id: &str, design: usize, x: f32, y: f32, v: f32) {
        self.push(
            id,
            "knob",
            KNOBS[design].name,
            x,
            y,
            Extra::Knob { design, v },
        );
        let span = knob_span(&self.section);
        self.params(format!("span={}", n(span)));
    }

    fn fader(&mut self, id: &str, x: f32, y: f32, v: f32) {
        self.push(id, "fader", "fader_track", x, y, Extra::Fader { v });
        let (_, handle) = self.size("fader_handle");
        self.params(format!("handle={}", n(handle)));
    }

    /// A seven-segment display: the rectangle holds `digits` cells of 9 x 16, centered.
    fn display(&mut self, id: &str, x: f32, y: f32, w: f32, digits: usize, text: &str) {
        self.layout.ctls.push(Ctl {
            id: format!("{}.{id}", self.section),
            section: self.section.clone(),
            kind: "display",
            part: "seg_0..9,seg_blank".into(),
            x: self.ox + x,
            y: self.oy + y,
            w,
            h: 16.0,
            extra: Extra::Digits {
                n: digits,
                text: text.to_string(),
            },
            params: String::new(),
        });
    }

    /// The up and down arrows of a display, at (x, y) and (x, y + 9).
    fn arrows(&mut self, id: &str, kind: &'static str, x: f32, y: f32) {
        self.part(&format!("{id}_up"), kind, "arrow_inc_rest", x, y);
        self.part(&format!("{id}_down"), kind, "arrow_dec_rest", x, y + 9.0);
    }

    /// A part laid over the controls in the demo (section coordinates).
    fn overlay(&mut self, part: &str, x: f32, y: f32) {
        self.layout
            .overlays
            .push((part.to_string(), self.ox + x, self.oy + y));
    }
}

pub const KNOB_303: usize = 0;
pub const KNOB_808_RED: usize = 1;
pub const KNOB_808_WHITE: usize = 2;
pub const KNOB_909: usize = 3;
pub const KNOB_GRAY: usize = 4;

/// The pixels of vertical drag for a knob's full range, by section: the original's (the
/// knob ctor's fourth argument, docs/ui_spec.md section 2): TB-303 62, TR-808 36, TR-909 41,
/// the others 64. Written to `layout.tsv` as the knobs' `span`.
pub fn knob_span(section: &str) -> f32 {
    match section {
        "tb303_1" | "tb303_2" => 62.0,
        "tr808" => 36.0,
        "tr909" => 41.0,
        _ => 64.0,
    }
}

/// The header meters' part (0..=4 segments lit) for each of the meter's 11 states: state 1
/// (byte 2) lights the first segment, the amber third from 7 and the red top one from 9, where
/// the master's 10-segment meters light their amber and red segments (task-179; docs/ui_spec.md
/// section 19). Written to `layout.tsv` as the header meters' `map`.
pub const HEADER_METER_MAP: [u8; 11] = [0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4];

/// The mod window's name text, in the window: (x, top, w, h), its font size and color.
pub const MOD_NAME: (f32, f32, f32, f32) = (2.0, 86.0, 80.4, 19.0);
pub const MOD_NAME_FONT: f32 = 6.6;
pub const MOD_NAME_INK: &str = "e8e2c2";

/// The drum machines' step strips, the hit areas of the step keys' row (section pixels, x y w
/// h): 16 cells of 16 px from x 14 (808) and of 21 px from x 26 (909), the original's
/// (docs/ui_spec.md sections 6 and 7).
pub const STRIP_808: (f32, f32, f32, f32) = (14.0, 90.0, 256.0, 20.0);
pub const STRIP_909: (f32, f32, f32, f32) = (26.0, 85.0, 336.0, 18.0);

/// The rows' focus click areas: x 16..526, y 50 + 115 r, 115 high (`sub_42ADBB`).
pub const ROW_AREA: (f32, f32, f32, f32) = (16.0, 50.0, 510.0, 115.0);

/// The control rectangles of the whole panel, with the demo state of the mockup (song mode,
/// playing, loop on, tempo 140, bar 9; row 1 has the focus; README.md of assets/ui).
pub fn layout(parts: &[Part]) -> Layout {
    let sizes: HashMap<String, (f32, f32)> =
        parts.iter().map(|p| (p.name.clone(), (p.w, p.h))).collect();
    let mut b = Builder {
        sizes: &sizes,
        layout: Layout {
            backgrounds: Vec::new(),
            sections: Vec::new(),
            ctls: Vec::new(),
            overlays: Vec::new(),
            foregrounds: Vec::new(),
        },
        section: String::new(),
        ox: 0.0,
        oy: 0.0,
    };
    b.section("wood_left", "wood_left", 0.0, 0.0);
    b.section("wood_right", "wood_right", 608.0, 0.0);
    transport(&mut b);
    for r in 0..4 {
        pattern(&mut b, r);
    }
    tb303(&mut b, 0);
    tb303(&mut b, 1);
    tr808(&mut b);
    tr909(&mut b);
    for c in 0..4 {
        mixer(&mut b, c);
    }
    master(&mut b);
    effects(&mut b);
    b.layout
}

fn transport(b: &mut Builder) {
    b.section("transport", "bg_transport", 16.0, 0.0);
    // the wordmark over the transport background (its pieces are not drawn here)
    b.layout
        .backgrounds
        .push(("logo".to_string(), LOGO_X, LOGO_Y));
    b.layout.sections.push("logo".to_string());
    b.knob("shuffle", KNOB_GRAY, 11.0, 13.0, 0.35);
    b.display("tempo", 44.0, 27.0, 27.0, 3, "140");
    b.arrows("tempo", "repeat", 73.0, 26.0);
    b.part("mode", "switch", "switch_mode_1", 161.0, 2.0);
    b.part("play", "trigger", "btn_play_lit", 96.0, 21.0);
    b.part("stop", "trigger", "btn_stop_rest", 128.0, 21.0);
    b.part("rewind", "wind", "btn_rewind_rest", 160.0, 21.0);
    b.part("ff", "wind", "btn_ff_rest", 192.0, 21.0);
    b.part("record", "toggle", "btn_record_rest", 224.0, 21.0);
    b.display("bar", 271.0, 27.0, 27.0, 3, "  9");
    b.arrows("bar", "wind", 300.0, 26.0);
    b.display("loop_start", 329.0, 27.0, 27.0, 3, "  1");
    b.arrows("loop_start", "repeat", 358.0, 26.0);
    b.display("loop_length", 372.0, 27.0, 27.0, 3, " 16");
    b.arrows("loop_length", "repeat", 401.0, 26.0);
    b.part("loop", "toggle", "toggle_loop_on", 317.0, 3.0);
}

fn pattern(b: &mut Builder, r: usize) {
    let oy = 50.0 + 115.0 * r as f32;
    b.section(
        &format!("pattern{}", r + 1),
        &format!("bg_pattern_{}", r + 1),
        16.0,
        oy,
    );
    let on = if r == 1 { "off" } else { "on" };
    b.part("on", "toggle", &format!("power_pat_{on}"), 5.0, 4.0);
    let lit = if r == 3 { 1 } else { 0 };
    let half = KEYCAP / 2.0;
    for i in 0..8 {
        let state = if i == lit { "on" } else { "off" };
        b.part(
            &format!("pattern{}", i + 1),
            "grid",
            &format!("pat_btn_{}_{state}", i + 1),
            KEY_X[i % 4] - half,
            KEY_Y[i / 4] - half,
        );
    }
    for (i, name) in ["A", "B", "C", "D"].into_iter().enumerate() {
        let state = if i == 0 { "on" } else { "off" };
        b.part(
            &format!("bank{name}"),
            "grid",
            &format!("bank_btn_{name}_{state}"),
            KEY_X[i] - half,
            BANK_Y - half,
        );
    }
    let shuffle = if r == 1 { "on" } else { "off" };
    b.part(
        "shuffle",
        "toggle",
        &format!("toggle_shuffle_{shuffle}"),
        4.5,
        94.7,
    );
    b.display("steps", 36.0, 95.0, 19.0, 2, "16");
    b.arrows("steps", "repeat", 56.0, 94.0);
    b.section(
        &format!("focus{}", r + 1),
        if r == 0 { "focus_on" } else { "focus_off" },
        86.0,
        oy,
    );
}

fn tb303(b: &mut Builder, r: usize) {
    let oy = 50.0 + 115.0 * r as f32;
    b.section(&format!("tb303_{}", r + 1), "bg_tb303", 90.0, oy);
    let first = r == 0;
    b.part(
        "wave",
        "hswitch",
        if first {
            "switch_wave_1"
        } else {
            "switch_wave_0"
        },
        24.65,
        18.5,
    );
    let vals = if first {
        [0.42, 0.34, 0.5, 0.32, 0.5, 0.5]
    } else {
        [0.5, 0.28, 0.62, 0.3, 0.72, 0.4]
    };
    for (k, name) in ["tune", "cutoff", "reso", "envmod", "decay", "accent"]
        .into_iter()
        .enumerate()
    {
        b.knob(name, KNOB_303, KNOB_X_303[k], 12.0, vals[k]);
    }
    b.display("edit_step", 330.0, 25.0, 19.0, 2, " 1");
    let pressed = if first { "rest" } else { "pressed" };
    b.part(
        "back",
        "trigger",
        &format!("btn_pill_17x10_{pressed}"),
        328.0,
        62.0,
    );
    b.part(
        "step",
        "trigger",
        &format!("btn_step_{pressed}"),
        324.6,
        90.0,
    );
    let led = |on: bool| if on { "on" } else { "off" };
    b.part(
        "pitch_led",
        "led",
        &format!("led_303s_{}", led(first)),
        24.0,
        67.0,
    );
    b.part("pitch_button", "trigger", "btn_pill_15x8_rest", 19.5, 73.6);
    b.part("clear", "trigger", "btn_pill_17x10_rest", 18.0, 96.0);
    // the keys (C lit) with their LEDs
    for (i, note) in WHITE_303.into_iter().enumerate() {
        let x = 52.0 + 21.0 * i as f32;
        b.part(&format!("key{note}"), "trigger", "key_white_rest", x, 89.0);
        let state = led(note == 0);
        b.part(
            &format!("key{note}_led"),
            "led",
            &format!("led_303_{state}"),
            x + 1.5,
            82.0,
        );
    }
    for (bx, note) in BLACK_303 {
        b.part(
            &format!("key{note}"),
            "trigger",
            "key_black_rest",
            bx - 4.2,
            62.6,
        );
        b.part(
            &format!("key{note}_led"),
            "led",
            "led_303s_off",
            bx - 3.0,
            56.0,
        );
    }
    // the step's note and rest LEDs and the note / pause button, then the flags
    let flags: &[&str] = if first {
        &["rest"]
    } else {
        &["note", "accent"]
    };
    b.part(
        "note_led",
        "led",
        &format!("led_303_{}", led(flags.contains(&"note"))),
        247.0,
        60.0,
    );
    b.part(
        "rest_led",
        "led",
        &format!("led_303_{}", led(flags.contains(&"rest"))),
        285.0,
        60.0,
    );
    b.part("note_pause", "trigger", "btn_pill_17x10_rest", 294.0, 58.0);
    for (i, name) in ["down", "up", "accent", "slide"].into_iter().enumerate() {
        let x = 224.0 + 25.0 * i as f32;
        b.part(name, "trigger", "btn_flag_rest", x, 89.0);
        let state = led(flags.contains(&name));
        b.part(
            &format!("{name}_led"),
            "led",
            &format!("led_303_{state}"),
            x + 1.5,
            82.0,
        );
    }
}

/// (x, y, design, id) of the 808's knobs.
const KNOBS_808: [(f32, f32, usize, &str); 22] = [
    (14.0, 9.0, KNOB_808_RED, "ac_level"),
    (36.0, 9.0, KNOB_808_RED, "bd_level"),
    (36.0, 31.0, KNOB_808_WHITE, "bd_tone"),
    (36.0, 54.0, KNOB_808_WHITE, "bd_decay"),
    (58.0, 9.0, KNOB_808_RED, "sd_level"),
    (58.0, 31.0, KNOB_808_WHITE, "sd_tone"),
    (58.0, 54.0, KNOB_808_WHITE, "sd_snappy"),
    (80.0, 9.0, KNOB_808_RED, "lt_level"),
    (80.0, 31.0, KNOB_808_WHITE, "lt_tuning"),
    (102.0, 9.0, KNOB_808_RED, "mt_level"),
    (102.0, 31.0, KNOB_808_WHITE, "mt_tuning"),
    (124.0, 9.0, KNOB_808_RED, "ht_level"),
    (124.0, 31.0, KNOB_808_WHITE, "ht_tuning"),
    (146.0, 9.0, KNOB_808_RED, "rs_level"),
    (168.0, 9.0, KNOB_808_RED, "cp_level"),
    (190.0, 9.0, KNOB_808_RED, "cb_level"),
    (212.0, 9.0, KNOB_808_RED, "cy_level"),
    (212.0, 31.0, KNOB_808_WHITE, "cy_tone"),
    (212.0, 54.0, KNOB_808_WHITE, "cy_decay"),
    (234.0, 9.0, KNOB_808_RED, "oh_level"),
    (234.0, 54.0, KNOB_808_WHITE, "oh_decay"),
    (256.0, 9.0, KNOB_808_RED, "ch_level"),
];

/// The 808's step keys: their left edges are 16 + 16 i, their tops 90.2.
pub const STEP_X0_808: f32 = 16.0;
pub const STEP_Y_808: f32 = 90.2;

fn tr808(b: &mut Builder) {
    b.section("tr808", "bg_tr808", 90.0, 280.0);
    let level = [
        0.62, 0.78, 0.66, 0.7, 0.7, 0.7, 0.72, 0.68, 0.64, 0.7, 0.66, 0.7,
    ];
    for &(x, y, design, id) in KNOBS_808.iter() {
        let col = ((x - 14.0) / 22.0) as usize;
        let v = match y as i32 {
            9 => level[col],
            31 => 0.5,
            _ => [0.45, 0.55, 0.3, 0.6][(col * 7) % 4],
        };
        b.knob(id, design, x, y, v);
    }
    for (i, x) in [81.0, 103.0, 125.0, 146.0, 169.0].into_iter().enumerate() {
        let pos = if i < 3 { 1 } else { 0 };
        b.part(
            &format!("slot{}_switch", i + 3),
            "vswitch",
            &format!("switch_808_{pos}"),
            x,
            55.0,
        );
    }
    let (fx, fy, _, _) = FG_TR808;
    b.layout
        .foregrounds
        .push(("fg_tr808".into(), b.ox + fx, b.oy + fy));
    let (x, y, _, _) = MOD_VIEW_808;
    b.part("mod_view", "modview", "mod_view", x, y);
    let (px, py, side) = MOD_PICTURE;
    let (nx, ny, nw, nh) = MOD_NAME;
    b.params(format!(
        "glass=mod_view_glass picture={},{},{},{} name={},{},{},{} font={} ink={MOD_NAME_INK}",
        n(px),
        n(py),
        n(side),
        n(side),
        n(nx),
        n(ny),
        n(nw),
        n(nh),
        n(MOD_NAME_FONT)
    ));
    b.overlay("mod_view_glass", x + px, y + py);
    // BD selected: its legend chip's mark
    let (lx, ly, _, _) = LEGEND_808;
    b.part("select_bd", "area", "sel808_BD", lx + 22.0, ly);
    b.overlay(&format!("sel808_{}", NAMES_808[1]), lx + 22.0, ly);
    let colors = ["red", "orange", "yellow", "white"];
    for i in 0..16 {
        let on = i % 4 == 0;
        b.part(
            &format!("step{}", i + 1),
            "steps",
            &format!(
                "step808_{}_{}",
                colors[i / 4],
                if on { "on" } else { "off" }
            ),
            STEP_X0_808 + 16.0 * i as f32,
            STEP_Y_808,
        );
    }
}

/// The 909's step keys: their left edges are 29 + 21 i, their tops 85.4.
pub const STEP_X0_909: f32 = 29.0;
pub const STEP_Y_909: f32 = 85.4;
/// The 909's flam button.
pub const FLAM_909: (f32, f32) = (7.6, 92.6);

fn tr909(b: &mut Builder) {
    b.section("tr909", "bg_tr909", 90.0, 395.0);
    let r1 = [
        0.5, 0.74, 0.5, 0.66, 0.5, 0.66, 0.5, 0.66, 0.5, 0.66, 0.66, 0.66, 0.66, 0.66, 0.66,
    ];
    for (i, (x, y, id, label)) in KNOBS_909.into_iter().enumerate() {
        let v = match id {
            "ac_level" => 0.5,
            "flam" => 0.75,
            _ if y < 50.0 => r1[i - 1],
            _ if label == "DEC" && x < 250.0 => 0.86,
            _ => 0.5,
        };
        b.knob(id, KNOB_909, x, y, v);
    }
    b.part(
        "flam_button",
        "toggle",
        "toggle_flam_on",
        FLAM_909.0,
        FLAM_909.1,
    );
    // BD selected: its chip's label lit and its name label's LED
    let (cx, _, _) = chip_909(1);
    b.part("select_bd", "area", "sel909_BD", cx, 4.6);
    b.overlay(&format!("sel909_{}", NAMES_909[1]), cx, 4.6);
    let (lx, ly) = led_909(1);
    b.overlay("led909_on", lx - 2.5, ly - 2.5);
    for i in 0..16 {
        let value = match i {
            0 | 4 | 8 | 12 => 2,
            14 => 1,
            10 => 3,
            _ => 0,
        };
        b.part(
            &format!("step{}", i + 1),
            "steps",
            &format!("step909_{value}"),
            STEP_X0_909 + 21.0 * i as f32,
            STEP_Y_909,
        );
    }
}

fn mixer(b: &mut Builder, c: usize) {
    b.section(
        &format!("mixer{}", c + 1),
        &format!("bg_mixer_{}", c + 1),
        456.0,
        50.0 + 115.0 * c as f32,
    );
    b.part("mute", "toggle", "power_mix_on", 3.0, 4.0);
    b.part(
        "meter",
        "meter",
        &format!("meter_14x14_{}", [2, 2, 3, 1][c]),
        51.0,
        4.0,
    );
    b.params(header_meter_params());
    b.knob("pan", KNOB_GRAY, 10.0, 25.0, 0.5);
    b.knob("delay", KNOB_GRAY, 42.0, 79.0, 0.2);
    b.fader("level", 10.0, 59.0, 0.72);
    let dist = if c == 1 { "mixsw_on" } else { "mixsw_off" };
    b.part("dist", "toggle", dist, 42.0, 19.0);
    b.part("pcf", "radio", "mixsw_off", 42.0, 37.0);
    b.part("comp", "radio", "mixsw_off", 42.0, 55.0);
}

fn master(b: &mut Builder) {
    b.section("master", "bg_master", 526.0, 50.0);
    b.part("meter_l", "meter", "meter_15x56_6", 16.0, 24.0);
    b.part("meter_r", "meter", "meter_15x56_6", 51.0, 24.0);
    b.fader("fader", 33.0, 25.0, 0.8);
    b.part("comp", "radio", "mixsw_on", 42.0, 81.0);
}

/// An effect's header lamp and meter, the header at `y` of the section.
fn fx_header(b: &mut Builder, y: f32, on: bool, level: u8) {
    let state = if on { "on" } else { "off" };
    b.part("on", "toggle", &format!("power_fx_{state}"), 3.0, y - 0.5);
    b.part(
        "meter",
        "meter",
        &format!("meter_14x14_{level}"),
        63.0,
        y + 0.5,
    );
    b.params(header_meter_params());
}

/// A header meter's `map` parameter ([`HEADER_METER_MAP`]).
fn header_meter_params() -> String {
    let map: Vec<String> = HEADER_METER_MAP.iter().map(|v| v.to_string()).collect();
    format!("map={}", map.join(","))
}

fn effects(b: &mut Builder) {
    b.section("pcf", "bg_pcf", 526.0, 148.0);
    fx_header(b, 2.5, true, 0);
    b.display("pattern", 10.0, 23.0, 18.0, 2, "25");
    b.arrows("pattern", "repeat", 29.0, 22.0);
    b.part("mode", "vswitch", "switch_fx_1", 49.0, 21.0);
    for (i, (id, v)) in [("freq", 0.0), ("q", 0.55), ("amt", 1.0), ("dec", 0.3)]
        .into_iter()
        .enumerate()
    {
        b.fader(id, 4.0 + 19.0 * i as f32, 50.0, v);
    }
    let (fx, fy, _, _) = FG_PCF;
    b.layout
        .foregrounds
        .push(("fg_pcf".into(), b.ox + fx, b.oy + fy));

    b.section("delay", "bg_delay", 526.0, 259.0);
    fx_header(b, 3.0, true, 4);
    b.display("steps", 9.0, 23.0, 19.0, 2, " 3");
    b.arrows("steps", "repeat", 29.0, 22.0);
    b.part("triplet", "vswitch", "switch_fx_1", 49.0, 22.0);
    b.knob("pan", KNOB_GRAY, 12.0, 58.0, 0.5);
    b.knob("feedback", KNOB_GRAY, 53.0, 58.0, 0.42);

    b.section("dist", "bg_dist", 526.0, 352.0);
    fx_header(b, 3.0, true, 1);
    b.knob("amount", KNOB_GRAY, 12.0, 28.0, 0.5);
    b.knob("shape", KNOB_GRAY, 53.0, 28.0, 0.36);

    b.section("comp", "bg_comp", 526.0, 417.0);
    fx_header(b, 3.0, true, 3);
    for (i, color) in GR_COLORS.into_iter().enumerate() {
        // the demo: the idle compressor at unity gain, LEDs 1 to 5 lit, as the app lights them
        let state = match (i < 5, color) {
            (true, "g") => "green".to_string(),
            (true, "y") => "amber".to_string(),
            (true, _) => "red".to_string(),
            (false, c) => format!("off_{c}"),
        };
        b.part(
            &format!("gr{}", i + 1),
            "meter",
            &format!("gr_led_{state}"),
            8.0 + 7.0 * i as f32,
            26.5,
        );
        // lit in the color of its place (green, amber, red), or dark
        let lit = match color {
            "g" => "green",
            "y" => "amber",
            _ => "red",
        };
        b.params(format!("on=gr_led_{lit} off=gr_led_off_{color}"));
    }
    b.knob("ratio", KNOB_GRAY, 12.0, 57.0, 0.52);
    b.knob("threshold", KNOB_GRAY, 53.0, 57.0, 0.5);
}

/// The `layout.tsv` text, the modern skin's layout table (assets/ui/README.md): the panel's
/// size, the backgrounds and the wordmark in drawing order, the row focus bars and click
/// areas, one line per control, the foregrounds, the drum machines' step strips and
/// instrument hit areas, and the selection marks; panel pixel rectangles.
pub fn tsv(parts: &[Part], l: &Layout) -> String {
    let size = |name: &str| {
        let p = parts
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("missing part {name}"));
        (p.w, p.h)
    };
    let mut s = String::from("id\tsection\tkind\tpart\tx\ty\tw\th\tparams\n");
    let mut row =
        |id: &str, sec: &str, kind: &str, part: &str, r: (f32, f32, f32, f32), p: &str| {
            let (x, y, w, h) = r;
            s.push_str(&format!(
                "{id}\t{sec}\t{kind}\t{part}\t{}\t{}\t{}\t{}",
                n(x),
                n(y),
                n(w),
                n(h)
            ));
            if !p.is_empty() {
                s.push('\t');
                s.push_str(p);
            }
            s.push('\n');
        };
    row("panel", "-", "panel", "-", (0.0, 0.0, PANEL_W, PANEL_H), "");
    // the backgrounds in the app's drawing order: the cheeks, the transport and the wordmark,
    // then each row's pattern section, instrument and mixer channel, then the master and the
    // effects
    let mut order = vec!["wood_left", "wood_right", "transport", "logo"];
    let rows: Vec<[String; 3]> = (0..4)
        .map(|r| {
            let synth = match r {
                0 | 1 => format!("tb303_{}", r + 1),
                2 => "tr808".into(),
                _ => "tr909".into(),
            };
            [
                format!("pattern{}", r + 1),
                synth,
                format!("mixer{}", r + 1),
            ]
        })
        .collect();
    for r in &rows {
        order.extend(r.iter().map(String::as_str));
    }
    order.extend(["master", "pcf", "delay", "dist", "comp"]);
    for sec in order {
        let i = l
            .sections
            .iter()
            .position(|s| s == sec)
            .unwrap_or_else(|| panic!("no section {sec}"));
        let (part, x, y) = &l.backgrounds[i];
        let (w, h) = size(part);
        if sec == "logo" {
            row("logo", "transport", "logo", part, (*x, *y, w, h), "");
        } else {
            row(&format!("{sec}.bg"), sec, "bg", part, (*x, *y, w, h), "");
        }
    }
    let (fw, fh) = size("focus_off");
    let (ax, ay, aw, ah) = ROW_AREA;
    for r in 0..4 {
        let i = l
            .sections
            .iter()
            .position(|s| *s == format!("focus{}", r + 1))
            .expect("a focus bar");
        let (_, x, y) = l.backgrounds[i];
        let sec = format!("pattern{}", r + 1);
        let p = format!("row={r} on=focus_on");
        row(
            &format!("{sec}.focus"),
            &sec,
            "focus",
            "focus_off",
            (x, y, fw, fh),
            &p,
        );
        let rect = (ax, ay + ah * r as f32, aw, ah);
        row(
            &format!("{sec}.row"),
            &sec,
            "row",
            "-",
            rect,
            &format!("row={r}"),
        );
    }
    for c in &l.ctls {
        row(
            &c.id,
            &c.section,
            c.kind,
            &c.part,
            (c.x, c.y, c.w, c.h),
            &c.params,
        );
    }
    for (part, x, y) in &l.foregrounds {
        let (w, h) = size(part);
        let sec = part.strip_prefix("fg_").unwrap_or(part);
        row(&format!("{sec}.fg"), sec, "fg", part, (*x, *y, w, h), "");
    }
    // the drum machines: their step strips, instrument hit areas and selection marks
    let (o8, o9) = ((90.0, 280.0), (90.0, 395.0));
    let at = |o: (f32, f32), (x, y, w, h): (f32, f32, f32, f32)| (o.0 + x, o.1 + y, w, h);
    row(
        "tr808.strip",
        "tr808",
        "strip",
        "-",
        at(o8, STRIP_808),
        "cells=16",
    );
    row(
        "tr909.strip",
        "tr909",
        "strip",
        "-",
        at(o9, STRIP_909),
        "cells=16",
    );
    // the 808: the legend row (12 cells of 22 from x 10) and the upper labels (the slots 3..7
    // from x 78); the 909: the legend row and the name row, AC to x 26, then BD..HT 42 apart
    // and RS..RC 21 apart (docs/ui_spec.md sections 6 and 7)
    let cells = |o: (f32, f32), segs: &[(f32, f32, u32)]| {
        let v: Vec<String> = segs
            .iter()
            .map(|(x, w, first)| format!("{}:{}:{first}", n(o.0 + x), n(*w)))
            .collect();
        format!("cells={} count=12", v.join(","))
    };
    let legend808 = cells(o8, &[(10.0, 22.0, 0)]);
    row(
        "tr808.legend",
        "tr808",
        "select",
        "-",
        at(o8, (10.0, 71.0, 264.0, 14.0)),
        &legend808,
    );
    let upper808 = cells(o8, &[(78.0, 22.0, 3)]);
    row(
        "tr808.upper",
        "tr808",
        "select",
        "-",
        at(o8, (78.0, 43.0, 106.0, 12.0)),
        &upper808,
    );
    let segs909 = format!(
        "{} open=1",
        cells(o9, &[(4.0, 22.0, 0), (26.0, 42.0, 1), (236.0, 21.0, 6)])
    );
    row(
        "tr909.legend",
        "tr909",
        "select",
        "-",
        at(o9, (4.0, 4.0, 358.0, 14.0)),
        &segs909,
    );
    row(
        "tr909.names",
        "tr909",
        "select",
        "-",
        at(o9, (26.0, 74.0, 336.0, 10.0)),
        &segs909,
    );
    let (lx, ly, lw, lh) = LEGEND_808;
    for (k, name) in NAMES_808.into_iter().enumerate() {
        let r = at(o8, (lx + 22.0 * k as f32, ly, lw, lh));
        let p = format!("slot={k}");
        row(
            &format!("tr808.mark_{name}"),
            "tr808",
            "mark",
            &format!("sel808_{name}"),
            r,
            &p,
        );
    }
    let (ux, uy, uw, uh) = UPPER_CHIP_808;
    for (k, name) in UPPER_808.into_iter().enumerate() {
        let r = at(o8, (ux + 22.0 * k as f32, uy, uw, uh));
        let p = format!("slot={k} upper=1");
        row(
            &format!("tr808.mark_{name}"),
            "tr808",
            "mark",
            &format!("sel808_{name}"),
            r,
            &p,
        );
    }
    // the 909's chip labels and its LEDs show the selection
    for (i, name) in NAMES_909.into_iter().enumerate() {
        let (x, w, _) = chip_909(i);
        let sel = format!("sel909_{name}");
        let p = format!("slot={i}");
        row(
            &format!("tr909.mark_{name}"),
            "tr909",
            "mark",
            &sel,
            at(o9, (x, 4.6, w, 9.2)),
            &p,
        );
    }
    for (i, name) in NAMES_909.into_iter().enumerate() {
        let (cx, cy) = led_909(i);
        let led = if i == 0 { "led909ac" } else { "led909" };
        let p = format!("slot={i}");
        let r = at(o9, (cx - 2.5, cy - 2.5, 5.0, 5.0));
        row(
            &format!("tr909.led_{name}"),
            "tr909",
            "mark",
            &format!("{led}_on"),
            r,
            &p,
        );
    }
    s
}

/// The whole panel as one SVG (the demo state), 624 x 510 panel pixels.
pub fn compose(parts: &[Part], l: &Layout) -> String {
    let by: HashMap<&str, &Part> = parts.iter().map(|p| (p.name.as_str(), p)).collect();
    let get = |name: &str| {
        *by.get(name)
            .unwrap_or_else(|| panic!("missing part {name}"))
    };
    let mut s = Svg::new();
    s.rect(0.0, 0.0, PANEL_W, PANEL_H, 0.0, SEAM);
    for (name, x, y) in &l.backgrounds {
        s.g_at(*x, *y).raw(&get(name).body).end_g();
    }
    for c in &l.ctls {
        match &c.extra {
            Extra::Plain => {
                if c.kind != "area" {
                    s.g_at(c.x, c.y).raw(&get(&c.part).body).end_g();
                }
            }
            Extra::Knob { design, v } => {
                let k = &KNOBS[*design];
                s.g_at(c.x, c.y);
                s.raw(&get(&format!("{}_body", k.name)).body);
                let a = -KNOB_SWEEP + v * 2.0 * KNOB_SWEEP;
                s.g_rotate(a, k.cx, k.cy);
                s.raw(&get(&format!("{}_pointer", k.name)).body);
                s.end_g().end_g();
            }
            Extra::Fader { v } => {
                s.g_at(c.x, c.y).raw(&get("fader_track").body).end_g();
                s.g_at(c.x, c.y + (1.0 - v) * 43.0)
                    .raw(&get("fader_handle").body)
                    .end_g();
            }
            Extra::Digits { n: count, text } => {
                let chars: Vec<char> = text.chars().collect();
                let x0 = c.x + (c.w - 9.0 * *count as f32) / 2.0;
                for i in 0..*count {
                    let ch = chars.get(i).copied().unwrap_or(' ');
                    let name = if ch == ' ' {
                        "seg_blank".to_string()
                    } else {
                        format!("seg_{ch}")
                    };
                    s.g_at(x0 + 9.0 * i as f32, c.y)
                        .raw(&get(&name).body)
                        .end_g();
                }
            }
        }
    }
    for (name, x, y) in l.foregrounds.iter().chain(&l.overlays) {
        s.g_at(*x, *y).raw(&get(name).body).end_g();
    }
    let (w, h) = (n(PANEL_W), n(PANEL_H));
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" \
         viewBox=\"0 0 {w} {h}\">{}{}</svg>\n",
        defs_for(&s.s),
        s.s
    )
}
