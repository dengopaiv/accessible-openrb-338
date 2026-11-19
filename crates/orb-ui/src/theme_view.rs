//! The mod theme viewer (task-187): a view-only window that draws the panel as the original
//! does, from the current kit's 58 panel bitmaps (`12015.jpg` .. `12571.jpg`, docs/rbm.md
//! 13.1), at the original's geometry (docs/ui_spec.md sections 1-9 and 21), and follows the
//! live panel state. The main panel keeps its own SVG art (task-078); this window only shows a
//! mod's look.
//!
//! The viewer takes no input that changes the song, the transport or the main panel: it has no
//! widget that senses a click or a drag, it shows the "not allowed" cursor over the panel and
//! it writes nothing to the status line. Its one key is Space, which switches its scaling
//! between nearest pixel (the default) and bilinear, kept in the UI settings
//! (`theme_viewer_filter`).
//!
//! [`Bitmaps`] decodes a kit's bitmaps, each falling back to the built-in one (the embedded
//! standard mod's) when the mod lacks it or it does not decode; [`ops`] lists what to draw
//! for a panel state (bitmap, film-strip frame, place); [`compose`] draws the list into one
//! 624 x 510 image. The window redraws its texture only when the list changes.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use egui::{Color32, ColorImage, Context, CursorIcon, Rect, Sense, TextureHandle, Ui};
use orb_cli::mods::Kit;

use crate::app::{App, View};
use crate::behavior;
use crate::layout::{PANEL_H, PANEL_W};
use crate::model::{Panel, peak_meter_frame};

/// The panel's size in pixels.
pub const W: usize = 624;
pub const H: usize = 510;

/// The 58 panel bitmaps a mod replaces (docs/rbm.md 13.1; docs/ui_spec.md section 21).
pub const IDS: [u16; 58] = [
    12015, 12016, 12017, 12019, 12020, 12030, 12031, 12032, 12040, 12041, 12057, 12058, 12060,
    12061, 12062, 12063, 12064, 12065, 12066, 12067, 12068, 12500, 12501, 12502, 12503, 12504,
    12505, 12506, 12507, 12509, 12510, 12520, 12521, 12522, 12523, 12524, 12525, 12550, 12551,
    12552, 12554, 12555, 12556, 12557, 12558, 12559, 12560, 12561, 12562, 12563, 12564, 12565,
    12566, 12567, 12568, 12569, 12570, 12571,
];

/// The largest bitmap side the viewer decodes (the widest built-in one is 1674): a mod's
/// JPEG that claims more falls back to the built-in bitmap instead of taking the memory.
const MAX_SIDE: u32 = 8192;

/// A kit's decoded panel bitmaps.
#[derive(Clone)]
pub struct Bitmaps {
    map: HashMap<u16, Arc<ColorImage>>,
    /// The bitmaps that fell back to the built-in ones, with the reason (missing, broken).
    pub fallbacks: Vec<(u16, String)>,
}

/// Decodes a JPEG within [`MAX_SIDE`]; `Err` says why not.
pub fn decode(bytes: &[u8]) -> Result<ColorImage, String> {
    let mut r =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Jpeg);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(256 << 20);
    r.limits(limits);
    let img = r.decode().map_err(|e| e.to_string())?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}

impl Bitmaps {
    /// The built-in kit's bitmaps (the embedded standard mod's), decoded once.
    pub fn builtin() -> &'static Bitmaps {
        static B: OnceLock<Bitmaps> = OnceLock::new();
        B.get_or_init(|| {
            let mut map = HashMap::new();
            let mut fallbacks = Vec::new();
            for id in IDS {
                match rbm::standard::builtin_file(&format!("{id}.jpg")).map(|b| decode(&b)) {
                    Some(Ok(img)) => {
                        map.insert(id, Arc::new(img));
                    }
                    Some(Err(e)) => fallbacks.push((id, e)),
                    None => fallbacks.push((id, "missing".into())),
                }
            }
            Bitmaps { map, fallbacks }
        })
    }

    /// The bitmaps of a set of files: `file(name)` gives a file's bytes or `None` when there is
    /// none. Each bitmap missing or broken falls back to the built-in one (the original's
    /// loader `sub_41703A` falls back file by file to the exe's resource).
    pub fn from_files(mut file: impl FnMut(&str) -> Option<Vec<u8>>) -> Self {
        let builtin = Self::builtin();
        let mut map = HashMap::new();
        let mut fallbacks = Vec::new();
        for id in IDS {
            let img = match file(&format!("{id}.jpg")) {
                None => Err("missing".to_string()),
                Some(b) => decode(&b),
            };
            match img {
                Ok(img) => {
                    map.insert(id, Arc::new(img));
                }
                Err(e) => {
                    fallbacks.push((id, e));
                    if let Some(b) = builtin.map.get(&id) {
                        map.insert(id, b.clone());
                    }
                }
            }
        }
        Self { map, fallbacks }
    }

    /// The bitmaps of a kit: the built-in kit's, or a recrypted mod file's laid over them. A
    /// file that cannot be read or is not a recrypted mod gives the built-in bitmaps, with
    /// every one listed as a fallback.
    pub fn of_kit(kit: &Kit) -> Self {
        match kit {
            Kit::Standard => Self::builtin().clone(),
            Kit::Mod(path) => {
                let opened = std::fs::read(path)
                    .map_err(|e| e.to_string())
                    .and_then(|b| rbm::Rbm::parse(b).map_err(|e| e.to_string()))
                    .and_then(|r| {
                        rbm::KeyTable::embedded(&r)
                            .filter(|t| r.slots_covered(t))
                            .map(|t| (r, t))
                            .ok_or_else(|| "not recrypted".to_string())
                    });
                match opened {
                    Ok((r, t)) => Self::from_files(|name| {
                        r.find(name)?;
                        r.file_by_name(name, &t).ok()
                    }),
                    Err(e) => {
                        let mut b = Self::builtin().clone();
                        b.fallbacks = IDS.iter().map(|&id| (id, e.clone())).collect();
                        b
                    }
                }
            }
        }
    }

    pub fn get(&self, id: u16) -> Option<&ColorImage> {
        self.map.get(&id).map(|a| a.as_ref())
    }
}

/// One blit: frame `frame` of the `n`-frame strip `id` (frames side by side, or stacked when
/// `vertical`), from `(sx, sy)` within the frame's cell, `w` x `h` pixels to `(x, y)` of the
/// panel. The frame's cell is `(size + 1) / n` pixels (the frame and one pixel of gap,
/// `CBitmapSequence`, 0x46DC43: `x = frame * (fw + 1)` with `fw = (w + 1) / n - 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Op {
    pub id: u16,
    pub n: u16,
    pub vertical: bool,
    pub frame: u16,
    pub sx: i32,
    pub sy: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Op {
    /// Frame `frame` of a horizontal strip, the frame's whole size at `(x, y)`.
    pub const fn h(id: u16, n: u16, frame: u16, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            id,
            n,
            vertical: false,
            frame,
            sx: 0,
            sy: 0,
            x,
            y,
            w,
            h,
        }
    }

    /// The same for a vertical strip.
    pub const fn v(id: u16, n: u16, frame: u16, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            vertical: true,
            ..Self::h(id, n, frame, x, y, w, h)
        }
    }

    /// A single picture.
    pub const fn pic(id: u16, x: i32, y: i32, w: i32, h: i32) -> Self {
        Self::h(id, 1, 0, x, y, w, h)
    }

    /// The source rectangle's top-left corner in a bitmap of `size`.
    pub fn source(&self, size: [usize; 2]) -> (i32, i32) {
        let n = self.n.max(1) as i32;
        if self.vertical {
            let cell = (size[1] as i32 + 1) / n;
            (self.sx, self.frame as i32 * cell + self.sy)
        } else {
            let cell = (size[0] as i32 + 1) / n;
            (self.frame as i32 * cell + self.sx, self.sy)
        }
    }

    /// The size drawn from a bitmap of `size`. A picture or a horizontal strip (whole frames,
    /// `sx` and `sy` 0) draws its frame at the bitmap's own frame size, `(w + 1) / n - 1`
    /// wide and the bitmap's height, whatever the control's size: the original sizes these
    /// views by their bitmap, anchored at the control's top-left corner (review round 4 of
    /// task-187: every horizontal strip of the ReBirthday mod, whose sizes differ from the
    /// built-in ones, matches the original's screenshot exactly that way, docs/ui_spec.md
    /// section 21). A vertical strip (the grid, the bank row, the meters) and a part of a frame
    /// draw `w` x `h`.
    pub fn size(&self, size: [usize; 2]) -> (i32, i32) {
        if self.vertical || self.sx != 0 || self.sy != 0 {
            (self.w, self.h)
        } else {
            let n = self.n.max(1) as i32;
            ((size[0] as i32 + 1) / n - 1, size[1] as i32)
        }
    }
}

/// Draws `ops` in order into a panel-sized image (black where nothing is drawn). Every blit is
/// clipped to its bitmap and to the panel, so no bitmap size can make it fail.
pub fn compose(b: &Bitmaps, ops: &[Op]) -> ColorImage {
    let mut img = ColorImage::filled([W, H], Color32::BLACK);
    for op in ops {
        let Some(src) = b.get(op.id) else {
            continue;
        };
        let (sx, sy) = op.source(src.size);
        let (w, h) = op.size(src.size);
        let (bw, bh) = (src.size[0] as i32, src.size[1] as i32);
        for j in 0..h {
            let (y, v) = (op.y + j, sy + j);
            if !(0..H as i32).contains(&y) || !(0..bh).contains(&v) {
                continue;
            }
            for i in 0..w {
                let (x, u) = (op.x + i, sx + i);
                if !(0..W as i32).contains(&x) || !(0..bw).contains(&u) {
                    continue;
                }
                img.pixels[y as usize * W + x as usize] =
                    src.pixels[v as usize * src.size[0] + u as usize];
            }
        }
    }
    img
}

/// The frame of an `n`-frame knob strip for the knob position `p` (0..=1000): `((n - 1) * p +
/// 500) / 1000` (`sub_4A6204`, docs/ui_spec.md section 2).
pub fn knob_frame(n: u16, p: i32) -> u16 {
    ((n as i32 - 1) * p.clamp(0, 1000) + 500).div_euclid(1000) as u16
}

/// The seven-segment frames of an `n`-digit display (12015: frame 0 blank, frame d + 1 the digit
/// d): right aligned, leading zeros blank, all blank when the display is disabled (`None`).
pub fn digits(value: Option<i32>, n: usize) -> Vec<u16> {
    behavior::digit_frames(value, n)
        .into_iter()
        .map(|f| {
            f.strip_prefix("seg_")
                .and_then(|d| d.parse::<u16>().ok())
                .map_or(0, |d| d + 1)
        })
        .collect()
}

/// The lights the panel model does not hold: the Sync and MIDI LEDs (a plugin's; dark in the
/// standalone app).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lights {
    pub sync: bool,
    pub midi: bool,
}

fn knob(o: &mut Vec<Op>, p: &Panel, id: &str, strip: (u16, u16, i32, i32), x: i32, y: i32) {
    let (bmp, n, w, h) = strip;
    o.push(Op::h(bmp, n, knob_frame(n, p.knob(id)), x, y, w, h));
}

/// The long fader track (the mixer channels' and the master's): 15 x 54.
pub const TRACK_LONG: (u16, i32) = (12032, 54);
/// The PCF's short fader track: 15 x 45 (the original draws it at the PCF's four faders,
/// docs/ui_spec.md section 21).
pub const TRACK_PCF: (u16, i32) = (12030, 45);

/// The handle's top (12031, 11 high) from the top of a track `track_h` high for the fader
/// position `p` (0..=999): the top at 999, the bottom at 0, linear over the travel
/// `track_h - 11` [G: the original's draw of CBitmapFaderView was not read; the ends follow
/// docs/ui_spec.md section 2].
pub fn handle_top(p: i32, track_h: i32) -> i32 {
    let travel = (track_h - behavior::FADER_HANDLE) as f32;
    ((1.0 - p.clamp(0, behavior::FADER_MAX) as f32 / behavior::FADER_MAX as f32) * travel).round()
        as i32
}

fn fader(o: &mut Vec<Op>, p: &Panel, id: &str, track: (u16, i32), x: i32, y: i32) {
    let (bmp, h) = track;
    o.push(Op::pic(bmp, x, y, 15, h));
    o.push(Op::pic(12031, x, y + handle_top(p.fader(id), h), 15, 11));
}

fn display(o: &mut Vec<Op>, value: Option<i32>, n: usize, x: i32, y: i32) {
    for (i, f) in digits(value, n).into_iter().enumerate() {
        o.push(Op::h(12015, 11, f, x + 9 * i as i32, y, 9, 16));
    }
}

fn toggle(o: &mut Vec<Op>, bmp: u16, on: bool, x: i32, y: i32, w: i32, h: i32) {
    o.push(Op::h(bmp, 2, on as u16, x, y, w, h));
}

/// The arrows of a value display: up `12019` over down `12020`, at rest (the viewer has no
/// pressed look).
fn arrows(o: &mut Vec<Op>, x: i32, y: i32) {
    o.push(Op::h(12019, 2, 0, x, y, 11, 9));
    o.push(Op::h(12020, 2, 0, x, y + 9, 11, 9));
}

const KNOB_SMALL: (u16, u16, i32, i32) = (12552, 32, 18, 19);
const KNOB_303: (u16, u16, i32, i32) = (12507, 54, 30, 30);
const KNOB_808_RED: (u16, u16, i32, i32) = (12521, 36, 13, 13);
const KNOB_808_WHITE: (u16, u16, i32, i32) = (12522, 36, 13, 13);
const KNOB_909: (u16, u16, i32, i32) = (12565, 41, 12, 12);

/// What the panel draws for the state `p` at the time `now` (the 303's note flash), in the
/// order drawn: backgrounds first, then the controls of each section (docs/ui_spec.md
/// section 21 lists every bitmap and place).
pub fn ops(p: &Panel, now: f64, lights: Lights) -> Vec<Op> {
    let mut o = Vec::with_capacity(700);
    // backgrounds
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

    transport(&mut o, p, lights);
    for r in 0..4 {
        pattern_section(&mut o, p, r);
    }
    for k in 0..2 {
        tb303(&mut o, p, k, now);
    }
    tr808(&mut o, p);
    tr909(&mut o, p);
    for c in 0..4 {
        mixer(&mut o, p, c);
    }
    effects(&mut o, p);
    o
}

fn transport(o: &mut Vec<Op>, p: &Panel, lights: Lights) {
    let t = &p.transport;
    knob(o, p, "transport.shuffle", KNOB_SMALL, 27, 13);
    // Sync and MIDI: three frames, off, green, red
    o.push(Op::h(12041, 3, lights.sync as u16, 55, 2, 11, 11));
    o.push(Op::h(12041, 3, lights.midi as u16, 90, 2, 11, 11));
    display(o, p.display("transport.tempo"), 3, 60, 27);
    arrows(o, 89, 26);
    o.push(Op::h(12016, 2, t.song_mode as u16, 177, 2, 26, 19));
    toggle(o, 12559, t.playing, 112, 21, 32, 25);
    toggle(o, 12560, false, 144, 21, 32, 25);
    toggle(o, 12561, false, 176, 21, 32, 25);
    toggle(o, 12562, false, 208, 21, 32, 25);
    toggle(o, 12563, t.recording, 240, 21, 32, 25);
    display(o, p.display("transport.bar"), 3, 287, 27);
    arrows(o, 316, 26);
    toggle(o, 12551, t.looping, 331, 2, 17, 21);
    display(o, p.display("transport.loop_start"), 3, 345, 27);
    arrows(o, 374, 26);
    display(o, p.display("transport.loop_length"), 3, 388, 27);
    arrows(o, 417, 26);
}

fn pattern_section(o: &mut Vec<Op>, p: &Panel, r: usize) {
    let y = 50 + 115 * r as i32;
    let row = &p.rows[r];
    let s = format!("pattern{}", r + 1);
    // x 4 of the section: where the original draws it (an exact match in the ReBirthday
    // screenshot; docs/ui_spec.md section 4 had 5)
    toggle(o, 12556, row.enabled, 20, y + 4, 13, 14);
    // the grid: frame 0 none lit, frame k button k (CABCPositionView)
    let lit = row.lit_button().map_or(0, |b| b as u16 + 1);
    o.push(Op::v(12554, 9, lit, 17, y + 19, 68, 40));
    // the bank row: one frame per bank, A..D
    o.push(Op::v(12555, 4, row.bank.min(3) as u16, 17, y + 58, 68, 30));
    toggle(
        o,
        12571,
        p.toggle_on(&format!("{s}.shuffle")),
        19,
        y + 93,
        18,
        20,
    );
    display(o, p.display(&format!("{s}.steps")), 2, 52, y + 95);
    arrows(o, 72, y + 94);
}

fn tb303(o: &mut Vec<Op>, p: &Panel, k: usize, now: f64) {
    let y = 50 + 115 * k as i32;
    let s = format!("tb303_{}", k + 1);
    let id = |n: &str| format!("{s}.{n}");
    o.push(Op::h(
        12504,
        2,
        p.switch_position(&id("wave")) as u16,
        114,
        y + 10,
        16,
        12,
    ));
    for (i, n) in ["tune", "cutoff", "reso", "envmod", "decay", "accent"]
        .into_iter()
        .enumerate()
    {
        knob(o, p, &id(n), KNOB_303, 162 + 33 * i as i32, y + 12);
    }
    display(o, p.display(&id("edit_step")), 2, 420, y + 25);
    o.push(Op::h(12510, 2, 0, 110, y + 74, 15, 8));
    let led = |o: &mut Vec<Op>, bmp: u16, name: &str, x: i32, ly: i32| {
        o.push(Op::h(
            bmp,
            2,
            p.led_lit(&id(name), now) as u16,
            x,
            y + ly,
            8,
            8,
        ));
    };
    led(o, 12506, "pitch_led", 113, 66);
    o.push(Op::h(12502, 2, 0, 108, y + 96, 17, 10));
    // the white keys (12501) under their LEDs (12505), the black keys (12509) and theirs
    // (12506)
    for (note, x) in [
        (0, 142),
        (2, 163),
        (4, 184),
        (5, 205),
        (7, 226),
        (9, 247),
        (11, 268),
    ]
    .into_iter()
    .chain([(12, 289)])
    {
        o.push(Op::h(12501, 2, 0, x, y + 89, 9, 16));
        led(o, 12505, &format!("key{note}_led"), x, 81);
    }
    for (note, x) in [(1, 153), (3, 174), (6, 216), (8, 237), (10, 258)] {
        o.push(Op::h(12509, 2, 0, x, y + 63, 8, 15));
        led(o, 12506, &format!("key{note}_led"), x - 1, 55);
    }
    for (name, x) in [("down", 314), ("up", 339), ("accent", 364), ("slide", 389)] {
        o.push(Op::h(12501, 2, 0, x, y + 89, 9, 16));
        led(o, 12505, &format!("{name}_led"), x, 81);
    }
    led(o, 12505, "note_led", 336, 59);
    led(o, 12505, "rest_led", 374, 59);
    o.push(Op::h(12502, 2, 0, 384, y + 58, 17, 10));
    o.push(Op::h(12502, 2, 0, 418, y + 62, 17, 10));
    o.push(Op::h(12503, 2, 0, 415, y + 90, 25, 17));
}

fn tr808(o: &mut Vec<Op>, p: &Panel) {
    let (x0, y0) = (90, 280);
    const RED: [(&str, i32); 12] = [
        ("ac_level", 14),
        ("bd_level", 36),
        ("sd_level", 58),
        ("lt_level", 80),
        ("mt_level", 102),
        ("ht_level", 124),
        ("rs_level", 146),
        ("cp_level", 168),
        ("cb_level", 190),
        ("cy_level", 212),
        ("oh_level", 234),
        ("ch_level", 256),
    ];
    for (n, x) in RED {
        knob(o, p, &format!("tr808.{n}"), KNOB_808_RED, x0 + x, y0 + 9);
    }
    const WHITE: [(&str, i32, i32); 10] = [
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
    ];
    for (n, x, y) in WHITE {
        knob(o, p, &format!("tr808.{n}"), KNOB_808_WHITE, x0 + x, y0 + y);
    }
    for (slot, x) in [(3, 81), (4, 103), (5, 125), (6, 146), (7, 169)] {
        let f = p.switch_position(&format!("tr808.slot{slot}_switch")) as u16;
        o.push(Op::h(12523, 2, f, x0 + x, y0 + 55, 11, 17));
    }
    // the instrument selector: the knob shows `(127 * i + 5) / 11` for instrument i
    // (`sub_441E79`)
    let v = (127 * p.inst808 as i32 + 5) / 11;
    let f = knob_frame(12, behavior::knob_from_engine(v));
    o.push(Op::h(12525, 12, f, x0 + 306, y0 + 41, 29, 31));
    // the step LEDs, lit in the group's color (frame i / 4); the playing step inverted
    for i in 0..16 {
        let lit = p.steps808[p.inst808.min(11)][i] != (p.playing_step808 == Some(i));
        if lit {
            o.push(Op::h(
                12524,
                4,
                (i / 4) as u16,
                x0 + 16 + 16 * i as i32,
                y0 + 91,
                11,
                5,
            ));
        }
    }
}

/// Where the TR-909 lights its selected instrument's LED (12567, 6 x 6), in the section, by
/// instrument (AC, BD, SD, LT, MT, HT, RS, CP, CH, OH, CC, RC): the unlit dots of the 909's
/// background (12564), the AC one in its box, the others in the name-label row (review round 4
/// of task-187: the original lights BD's, matched pixel for pixel at (29, 76) with the
/// ReBirthday mod, and the built-in screenshot shows BD's lit red) [C for BD's place, the
/// others by the dots of the art].
pub const TR909_SELECTED_LED: [(i32, i32); 12] = [
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

fn tr909(o: &mut Vec<Op>, p: &Panel) {
    let (x0, y0) = (90, 395);
    const KNOBS: [(&str, i32, i32); 28] = [
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
    ];
    for (n, x, y) in KNOBS {
        knob(o, p, &format!("tr909.{n}"), KNOB_909, x0 + x, y0 + y);
    }
    toggle(
        o,
        12568,
        p.toggle_on("tr909.flam_button"),
        x0 + 8,
        y0 + 93,
        14,
        14,
    );
    // the selected instrument's LED (12567) over its unlit dot of the background
    let (lx, ly) = TR909_SELECTED_LED[p.inst909.min(11)];
    o.push(Op::pic(12567, x0 + lx, y0 + ly, 6, 6));
    // the step LEDs: value 1 frame 0, 2 frame 1, 3 and above frame 3 (`sub_444E30`); the
    // playing step inverted (an unlit step shows value 2, a lit one dark)
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
}

fn mixer(o: &mut Vec<Op>, p: &Panel, c: usize) {
    let (x0, y0) = (456, 50 + 115 * c as i32);
    let s = format!("mixer{}", c + 1);
    let id = |n: &str| format!("{s}.{n}");
    toggle(o, 12550, p.toggle_on(&id("mute")), x0 + 3, y0 + 4, 17, 15);
    let m = p.meters.get(&id("meter")).copied().unwrap_or(0);
    o.push(Op::h(
        12558,
        4,
        peak_meter_frame(m, 4),
        x0 + 51,
        y0 + 4,
        14,
        15,
    ));
    toggle(o, 12040, p.toggle_on(&id("dist")), x0 + 42, y0 + 19, 24, 11);
    toggle(o, 12040, p.toggle_on(&id("pcf")), x0 + 42, y0 + 37, 24, 11);
    toggle(o, 12040, p.toggle_on(&id("comp")), x0 + 42, y0 + 55, 24, 11);
    knob(o, p, &id("pan"), KNOB_SMALL, x0 + 10, y0 + 25);
    fader(o, p, &id("level"), TRACK_LONG, x0 + 10, y0 + 59);
    knob(o, p, &id("delay"), KNOB_SMALL, x0 + 42, y0 + 79);
}

fn effects(o: &mut Vec<Op>, p: &Panel) {
    // the original's peak-meter scale (`model::peak_meter_frame`) over each strip's frames
    let meter = |p: &Panel, id: &str, frames: u16| {
        peak_meter_frame(p.meters.get(id).copied().unwrap_or(0), frames)
    };
    // master (526, 50): the 11-frame meters, the fader, the stereo compressor switch
    // the right one at x 50 of the section (an exact match in the ReBirthday screenshot;
    // docs/ui_spec.md section 9 had 51)
    for (id, x) in [("master.meter_l", 542), ("master.meter_r", 576)] {
        o.push(Op::v(12557, 11, meter(p, id, 11), x, 74, 15, 56));
    }
    fader(o, p, "master.fader", TRACK_LONG, 559, 75);
    toggle(o, 12040, p.toggle_on("master.comp"), 568, 131, 24, 11);
    // the effects' on/off buttons (12066) and input meters (12067)
    for (s, y) in [("pcf", 148), ("delay", 259), ("dist", 352), ("comp", 417)] {
        let dy = if s == "pcf" { 2 } else { 3 };
        toggle(
            o,
            12066,
            p.toggle_on(&format!("{s}.on")),
            529,
            y + dy,
            16,
            16,
        );
        let f = meter(p, &format!("{s}.meter"), 4);
        o.push(Op::h(12067, 4, f, 590, y + dy, 13, 16));
    }
    // PCF (526, 148)
    display(o, p.display("pcf.pattern"), 2, 536, 171);
    arrows(o, 555, 170);
    o.push(Op::h(
        12017,
        2,
        p.switch_position("pcf.mode") as u16,
        575,
        169,
        9,
        18,
    ));
    for (i, n) in ["freq", "q", "amt", "dec"].into_iter().enumerate() {
        // the short track 12030, not 12032: the original's PCF faders are 45 high (the
        // screenshot matches 12030 at each, 1.5; 12032 would run over the FREQ..DEC labels)
        fader(
            o,
            p,
            &format!("pcf.{n}"),
            TRACK_PCF,
            530 + 19 * i as i32,
            203,
        );
    }
    // delay (526, 259)
    display(o, p.display("delay.steps"), 2, 535, 282);
    arrows(o, 555, 281);
    o.push(Op::h(
        12017,
        2,
        p.switch_position("delay.triplet") as u16,
        575,
        281,
        9,
        18,
    ));
    knob(o, p, "delay.pan", KNOB_SMALL, 538, 317);
    knob(o, p, "delay.feedback", KNOB_SMALL, 579, 317);
    // dist (526, 352)
    knob(o, p, "dist.amount", KNOB_SMALL, 538, 380);
    knob(o, p, "dist.shape", KNOB_SMALL, 579, 380);
    // comp (526, 417): the gain-reduction LEDs, 7 px each, a bar from the left of the lit row
    // (frame 1) over the unlit one, as the original's draw `sub_424C20` blits them
    o.push(Op::v(12570, 2, 0, 534, 442, 63, 10));
    for i in 0..crate::model::gr_lit(p.gr_byte) {
        let x = 7 * i as i32;
        o.push(Op {
            sx: x,
            ..Op::v(12570, 2, 1, 534 + x, 442, 7, 10)
        });
    }
    knob(o, p, "comp.ratio", KNOB_SMALL, 538, 473);
    knob(o, p, "comp.threshold", KNOB_SMALL, 579, 473);
}

/// The viewer's state in the app: whether it is open and what it drew last.
#[derive(Default)]
pub struct Viewer {
    pub open: bool,
    kit: Option<Kit>,
    bitmaps: Option<Bitmaps>,
    drawn: Vec<Op>,
    tex: Option<TextureHandle>,
    /// The menu item was chosen again while open: focus the window at its next frame.
    focus: bool,
    /// Whether the texture scales by nearest pixel (else bilinear).
    nearest: bool,
    /// The number of the native window: a new number is a new window (the Linux way to bring
    /// the viewer to the front, [`App::show_theme_viewer`]).
    serial: u32,
    /// The window's inner size, as its last pass saw it (points).
    size: Option<egui::Vec2>,
    /// The inner size the window is created with: `size` as it was when the window was
    /// opened, and constant while the window lives. The builder must not follow the live
    /// `size`: egui turns a changed builder size into `ViewportCommand::InnerSize`, which
    /// fought the user's drag (task-255).
    open_size: Option<egui::Vec2>,
    /// The passes still to repaint at once after the window opened: Wayland draws the window's
    /// frame and title bar only on a repaint (the user's review of task-187).
    fresh: u8,
    /// How the menu item and the mod window's click bring an open viewer to the front: a new
    /// window (`true`) or `ViewportCommand::Focus`. `None`: the platform's way, a new window on
    /// Linux only ([`REOPEN_TO_FOCUS`]).
    reopen: Option<bool>,
    /// How many times the panel image was composed (a test checks it follows the state).
    pub composed: u64,
}

/// Whether this platform brings an open viewer to the front by closing it and opening a new
/// window of the same size (Linux: GNOME's Mutter, on Wayland and X11, raises a window only
/// with an activation token from the click, which egui does not pass, so `Focus` does
/// nothing; a new window gets the focus. The user's decision, review of task-187), rather than
/// `ViewportCommand::Focus` (macOS, Windows).
pub const REOPEN_TO_FOCUS: bool = cfg!(target_os = "linux");

/// The passes repainted at once after the window opens.
const FRESH_PASSES: u8 = 3;

/// The viewer's native window number `serial` (its viewport id).
pub fn viewport_id(serial: u32) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("orb-theme-viewer", serial))
}

/// The id of the area the panel is drawn in (a test finds the viewer's rectangle by it).
pub fn panel_id() -> egui::Id {
    egui::Id::unique("orb-theme-viewer-panel")
}

/// The window's title.
pub const TITLE: &str = "Theme Viewer";

/// The desk around the panel when the window's shape differs from the panel's.
/// Black (the user's review round 4 of task-187).
pub const DESK: Color32 = Color32::BLACK;

/// Draws the panel texture in `ui`'s whole area, letterboxed at the panel's aspect, and shows
/// the "not allowed" cursor over it. Senses nothing but the hover.
fn draw(ui: &mut Ui, tex: &TextureHandle) {
    let area = ui.max_rect();
    ui.painter().rect_filled(area, 0.0, DESK);
    let view = View::fit(area, ui.ctx().pixels_per_point());
    let rect = view.rect(0.0, 0.0, PANEL_W, PANEL_H);
    let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
    ui.painter().image(tex.id(), rect, uv, Color32::WHITE);
    ui.interact(rect, panel_id(), Sense::hover());
    // over the panel, and not under another window (the same frame's pointer)
    let over = ui.ctx().pointer_hover_pos().is_some_and(|p| {
        rect.contains(p) && ui.ctx().layer_id_at(p).is_none_or(|l| l == ui.layer_id())
    });
    if over {
        ui.ctx().set_cursor_icon(CursorIcon::NotAllowed);
    }
}

impl App {
    /// Mods > Theme Viewer: opens the viewer (it is shown from the next frame on), or brings
    /// the open one to the front with the focus: `ViewportCommand::Focus`, or on Linux a new
    /// window of the same inner size in place of the open one ([`REOPEN_TO_FOCUS`]). Nothing
    /// in a plugin's editor, which has no such item.
    pub fn show_theme_viewer(&mut self) {
        if self.is_plugin() {
            return;
        }
        let v = &mut self.theme_viewer;
        if !v.open {
            v.open = true;
            v.serial += 1;
            v.fresh = FRESH_PASSES;
            v.open_size = v.size;
        } else if v.reopen.unwrap_or(REOPEN_TO_FOCUS) {
            // the open window goes (its viewport is not shown again) and a new one comes
            v.serial += 1;
            v.fresh = FRESH_PASSES;
            v.open_size = v.size;
        } else {
            v.focus = true;
        }
    }

    /// Sets how an open viewer is brought to the front (`true`: a new window; `false`:
    /// `ViewportCommand::Focus`), whatever the platform; for the tests of both ways.
    pub fn set_theme_viewer_reopens(&mut self, reopen: bool) {
        self.theme_viewer.reopen = Some(reopen);
    }

    /// The viewport id of the viewer's window (the last one opened).
    pub fn theme_viewer_viewport(&self) -> egui::ViewportId {
        viewport_id(self.theme_viewer.serial)
    }

    /// Closes the viewer, as its close button does.
    pub fn close_theme_viewer(&mut self) {
        self.theme_viewer.open = false;
        self.theme_viewer.tex = None;
        self.theme_viewer.drawn.clear();
    }

    pub fn theme_viewer_is_open(&self) -> bool {
        self.theme_viewer.open
    }

    /// The bitmaps the viewer draws with (`None` before its first frame).
    pub fn theme_viewer_bitmaps(&self) -> Option<&Bitmaps> {
        self.theme_viewer.bitmaps.as_ref()
    }

    /// The scaling of the viewer's texture: `Some(true)` nearest pixel, `Some(false)` bilinear,
    /// `None` before its first frame.
    pub fn theme_viewer_nearest(&self) -> Option<bool> {
        self.theme_viewer
            .tex
            .is_some()
            .then_some(self.theme_viewer.nearest)
    }

    /// What the viewer drew last.
    pub fn theme_viewer_ops(&self) -> &[Op] {
        &self.theme_viewer.drawn
    }

    /// Draws the viewer when it is open: a native window of its own (egui's immediate
    /// viewport; a backend without more windows shows it inside the main one, egui's
    /// fallback). Its close button closes it. Nothing in a plugin's editor.
    pub fn theme_viewer_ui(&mut self, ctx: &Context) {
        if !self.theme_viewer.open || self.is_plugin() {
            return;
        }
        let kit = self.doc.kit.clone();
        let now = ctx.input(|i| i.time);
        // the Sync and MIDI LEDs are a plugin's: dark in the standalone app
        let ops = ops(&self.panel, now, Lights::default());
        let nearest = self.settings.viewer_nearest;
        let v = &mut self.theme_viewer;
        if v.kit.as_ref() != Some(&kit) || v.bitmaps.is_none() {
            v.bitmaps = Some(Bitmaps::of_kit(&kit));
            v.kit = Some(kit);
            v.drawn.clear();
        }
        if v.tex.is_none() || v.drawn != ops || v.nearest != nearest {
            let img = compose(v.bitmaps.as_ref().expect("set above"), &ops);
            v.composed += 1;
            let options = if nearest {
                egui::TextureOptions::NEAREST
            } else {
                egui::TextureOptions::LINEAR
            };
            match &mut v.tex {
                Some(t) => t.set(img, options),
                None => v.tex = Some(ctx.load_texture("theme-viewer", img, options)),
            }
            v.drawn = ops;
            v.nearest = nearest;
        }
        let tex = v.tex.clone().expect("set above");
        let title = format!("{TITLE}: {}", self.doc.mod_name);
        let focus = std::mem::take(&mut v.focus);
        let id = viewport_id(v.serial);
        if v.fresh > 0 {
            // a new window: repaint at once for a few passes, so that it gets its frame and
            // title bar without waiting for the mouse (Wayland), on every platform
            v.fresh -= 1;
            ctx.request_repaint_of(id);
            ctx.request_repaint();
        }
        let mut open = true;
        let mut toggle = false;
        let mut size = None;
        let builder = egui::ViewportBuilder::default()
            .with_title(title)
            // a window opened in place of the open one keeps its size; the size is the one
            // at creation, never the live one (a changed size is a resize command)
            .with_inner_size(v.open_size.unwrap_or(egui::vec2(PANEL_W, PANEL_H)))
            .with_min_inner_size([PANEL_W / 4.0, PANEL_H / 4.0])
            .with_resizable(true);
        ctx.show_viewport_immediate(id, builder, |ui, _class| {
            if focus {
                // the menu item again: the open window comes to the front with the focus
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            draw(ui, &tex);
            size = ui.input(|i| i.viewport().inner_rect).map(|r| r.size());
            if ui.input(|i| i.viewport().close_requested()) {
                open = false;
            }
            // the one key the viewer takes (the user's review of task-187): Space switches the
            // scaling between bilinear and nearest pixel. This is the viewer window's own input,
            // so the main panel never sees the key (no play or stop).
            toggle = ui.input(|i| {
                i.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::Space,
                            pressed: true,
                            repeat: false,
                            ..
                        }
                    )
                })
            });
        });
        if size.is_some() {
            self.theme_viewer.size = size;
        }
        if toggle {
            let nearest = !self.settings.viewer_nearest;
            self.settings.set_viewer_nearest(nearest);
            if let Err(e) = self.settings.save() {
                self.fail(e);
            }
            ctx.request_repaint();
        }
        if !open {
            self.close_theme_viewer();
        }
    }
}
