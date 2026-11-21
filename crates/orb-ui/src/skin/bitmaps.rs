//! The classic skin's raster source (task-267, docs/ui_spec.md section 21): a kit's 58 panel
//! bitmaps (`12015.jpg` .. `12571.jpg`, docs/rbm.md 13.1), their scale, and the blits that
//! compose the panel from them. No input code.
//!
//! [`Bitmaps`] decodes a kit's bitmaps, each falling back to the built-in one (the embedded
//! standard mod's) when the mod lacks it or it does not decode. A set may be a hi-res one: its
//! scale N is the background 12057's size over its 1x size (592 x 50), and every bitmap is N
//! times its 1x size ([`find_scale`]). [`compose`] draws a list of [`Op`]s, all in the
//! original's 624 x 510 panel units, into one image of 624N x 510N pixels: places, frame cells,
//! digit pitches and part offsets scale by N.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use egui::{Color32, ColorImage};
use orb_cli::mods::Kit;

/// The panel's size in the original's pixels (the classic skin's units).
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

/// The bitmap whose size gives a set's scale: the transport background, 592 x 50 at 1x.
pub const SCALE_ID: u16 = 12057;
pub const SCALE_1X: [usize; 2] = [592, 50];

/// The largest bitmap side decoded (the widest built-in one is 1674; a 4x set's 6696): a mod's
/// JPEG that claims more falls back to the built-in bitmap instead of taking the memory.
const MAX_SIDE: u32 = 8192;

/// A kit's decoded panel bitmaps at one scale.
#[derive(Clone)]
pub struct Bitmaps {
    map: HashMap<u16, Arc<ColorImage>>,
    /// The set's scale N: every bitmap is N times its 1x size.
    scale: usize,
    /// The bitmaps that fell back to the built-in ones, with the reason (missing, broken).
    pub fallbacks: Vec<(u16, String)>,
    /// Why the kit's own set could not be used (its sizes are no whole scale, or the file is
    /// no recrypted mod): the built-in set is drawn instead.
    pub error: Option<String>,
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

/// An image `n` times larger, each pixel an `n` x `n` block (nearest).
pub fn upscale(img: &ColorImage, n: usize) -> ColorImage {
    if n == 1 {
        return img.clone();
    }
    let [w, h] = img.size;
    let mut out = ColorImage::filled([w * n, h * n], Color32::BLACK);
    for y in 0..h * n {
        for x in 0..w * n {
            out.pixels[y * w * n + x] = img.pixels[(y / n) * w + x / n];
        }
    }
    out
}

/// The scale N of a set of bitmaps (task-267, the user's hi-res classic sets): the background
/// 12057's width over its 1x width 592 and its height over its 1x height 50, each to the
/// nearest whole number; N is that number when both agree, else 1. Every bitmap's sides must
/// then be multiples of N. A set without 12057 is at 1x. `Err` names the bitmap that breaks
/// the rule.
///
/// The sides are rounded, not required to be exact multiples, because the mods' own 1x sets do
/// not keep the built-in size: 51 of the 320 mods of the user's collection have a 12057 of 600,
/// 608 or 609 x 50, 593 x 51 or 592 x 90 (the original draws a bitmap at its own size,
/// docs/ui_spec.md section 21), and they are 1x sets. Requiring both sides to agree keeps a 1x
/// background that is only wider (888 x 50 and up) or only taller at 1x; a 1x background
/// about N times larger both ways would still read as N.
pub fn find_scale(map: &HashMap<u16, ColorImage>) -> Result<usize, String> {
    let n = match map.get(&SCALE_ID) {
        None => 1,
        Some(bg) => {
            let ratio = |side: usize, one: usize| (side as f64 / one as f64).round() as usize;
            let (nw, nh) = (
                ratio(bg.size[0], SCALE_1X[0]),
                ratio(bg.size[1], SCALE_1X[1]),
            );
            if nw == nh { nw.max(1) } else { 1 }
        }
    };
    let mut ids: Vec<_> = map.keys().copied().collect();
    ids.sort_unstable();
    for id in ids {
        let [w, h] = map[&id].size;
        if w % n != 0 || h % n != 0 {
            return Err(format!(
                "bitmap {id} is {w} x {h}: not a multiple of the set's scale {n} (from the \
                 background {SCALE_ID})"
            ));
        }
    }
    Ok(n)
}

impl Bitmaps {
    /// The built-in kit's bitmaps (the embedded standard mod's, 1x), decoded once.
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
            Bitmaps {
                map,
                scale: 1,
                fallbacks,
                error: None,
            }
        })
    }

    /// The built-in set standing in for a kit whose own set cannot be used (`why`).
    fn builtin_for(why: String) -> Self {
        let mut b = Self::builtin().clone();
        b.fallbacks = IDS.iter().map(|&id| (id, why.clone())).collect();
        b.error = Some(why);
        b
    }

    /// A set of decoded bitmaps (`missing` lists those that could not be had and why): its
    /// scale is found ([`find_scale`]), and each missing bitmap is the built-in one upscaled to
    /// it. A set of no whole scale gives the built-in set with the error.
    pub fn from_images(map: HashMap<u16, ColorImage>, missing: Vec<(u16, String)>) -> Self {
        let n = match find_scale(&map) {
            Ok(n) => n,
            Err(e) => return Self::builtin_for(e),
        };
        let builtin = Self::builtin();
        let mut out: HashMap<u16, Arc<ColorImage>> = map
            .into_iter()
            .map(|(id, img)| (id, Arc::new(img)))
            .collect();
        for (id, _) in &missing {
            if let Some(b) = builtin.map.get(id) {
                let img = if n == 1 {
                    b.clone()
                } else {
                    Arc::new(upscale(b, n))
                };
                out.insert(*id, img);
            }
        }
        Self {
            map: out,
            scale: n,
            fallbacks: missing,
            error: None,
        }
    }

    /// The bitmaps of a set of files: `file(name)` gives a file's bytes or `None` when there is
    /// none. Each bitmap missing or broken falls back to the built-in one (the original's
    /// loader `sub_41703A` falls back file by file to the exe's resource).
    pub fn from_files(mut file: impl FnMut(&str) -> Option<Vec<u8>>) -> Self {
        let mut map = HashMap::new();
        let mut missing = Vec::new();
        for id in IDS {
            let img = match file(&format!("{id}.jpg")) {
                None => Err("missing".to_string()),
                Some(b) => decode(&b),
            };
            match img {
                Ok(img) => {
                    map.insert(id, img);
                }
                Err(e) => missing.push((id, e)),
            }
        }
        Self::from_images(map, missing)
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
                    Err(e) => Self::builtin_for(e),
                }
            }
        }
    }

    pub fn get(&self, id: u16) -> Option<&ColorImage> {
        self.map.get(&id).map(|a| a.as_ref())
    }

    /// The set's scale N (1 for the original's sizes).
    pub fn scale(&self) -> usize {
        self.scale
    }
}

/// One blit, in panel units: frame `frame` of the `n`-frame strip `id` (frames side by side, or
/// stacked when `vertical`), from `(sx, sy)` within the frame's cell, `w` x `h` to `(x, y)` of
/// the panel. The frame's cell is `(size + 1) / n` units (the frame and one unit of gap,
/// `CBitmapSequence`, 0x46DC43: `x = frame * (fw + 1)` with `fw = (w + 1) / n - 1`), the size
/// taken at 1x (the bitmap's over the set's scale).
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

    /// The source rectangle's top-left corner in a bitmap of `size` (1x units).
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

    /// The size drawn from a bitmap of `size` (1x units). A picture or a horizontal strip
    /// (whole frames, `sx` and `sy` 0) draws its frame at the bitmap's own frame size,
    /// `(w + 1) / n - 1` wide and the bitmap's height, whatever the control's size: the
    /// original sizes these views by their bitmap, anchored at the control's top-left corner
    /// (review round 4 of task-187, docs/ui_spec.md section 21). A vertical strip (the grid,
    /// the bank row, the meters) and a part of a frame draw `w` x `h`.
    pub fn size(&self, size: [usize; 2]) -> (i32, i32) {
        if self.vertical || self.sx != 0 || self.sy != 0 {
            (self.w, self.h)
        } else {
            let n = self.n.max(1) as i32;
            ((size[0] as i32 + 1) / n - 1, size[1] as i32)
        }
    }
}

/// Draws `ops` in order into a panel image of 624N x 510N pixels, N the set's scale (black
/// where nothing is drawn): each blit's place, source and size are its panel units times N,
/// its frame cell found from the bitmap's 1x size. Every blit is clipped to its bitmap and to
/// the panel, so no bitmap size can make it fail.
pub fn compose(b: &Bitmaps, ops: &[Op]) -> ColorImage {
    let n = b.scale.max(1) as i32;
    let (pw, ph) = (W as i32 * n, H as i32 * n);
    let mut img = ColorImage::filled([pw as usize, ph as usize], Color32::BLACK);
    for op in ops {
        let Some(src) = b.get(op.id) else {
            continue;
        };
        let size1 = [src.size[0] / n as usize, src.size[1] / n as usize];
        let (sx, sy) = op.source(size1);
        let (w, h) = op.size(size1);
        let (sx, sy, w, h, x0, y0) = (sx * n, sy * n, w * n, h * n, op.x * n, op.y * n);
        let (bw, bh) = (src.size[0] as i32, src.size[1] as i32);
        for j in 0..h {
            let (y, v) = (y0 + j, sy + j);
            if !(0..ph).contains(&y) || !(0..bh).contains(&v) {
                continue;
            }
            for i in 0..w {
                let (x, u) = (x0 + i, sx + i);
                if !(0..pw).contains(&x) || !(0..bw).contains(&u) {
                    continue;
                }
                img.pixels[(y * pw + x) as usize] = src.pixels[(v * bw + u) as usize];
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

/// The seven-segment frames of an `n`-digit display (12015: frame 0 blank, frame d + 1 the
/// digit d): right aligned, leading zeros blank, all blank when the display is disabled
/// (`None`).
pub fn digits(value: Option<i32>, n: usize) -> Vec<u16> {
    crate::behavior::digit_frames(value, n)
        .into_iter()
        .map(|d| d.map_or(0, |d| d as u16 + 1))
        .collect()
}

/// The handle's top (`handle` high) from the top of a track `track` high for the fader
/// position `p` (0..=999): the top at 999, the bottom at 0, linear over the travel
/// `track - handle`, rounded to a whole unit [G: the original's draw of CBitmapFaderView was
/// not read; the ends follow docs/ui_spec.md section 2].
pub fn handle_top(p: i32, track: i32, handle: i32) -> i32 {
    let travel = (track - handle) as f32;
    let max = crate::behavior::FADER_MAX;
    ((1.0 - p.clamp(0, max) as f32 / max as f32) * travel).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: usize, h: usize) -> ColorImage {
        ColorImage::filled([w, h], Color32::RED)
    }

    #[test]
    fn the_scale_is_the_backgrounds() {
        let mut m = HashMap::new();
        assert_eq!(find_scale(&m), Ok(1));
        m.insert(SCALE_ID, solid(592, 50));
        m.insert(12031, solid(15, 11));
        assert_eq!(find_scale(&m), Ok(1));
        m.insert(SCALE_ID, solid(1184, 100));
        let e = find_scale(&m).unwrap_err();
        assert!(e.contains("12031") && e.contains("15 x 11"), "{e}");
        m.insert(12031, solid(30, 22));
        assert_eq!(find_scale(&m), Ok(2));
        m.insert(SCALE_ID, solid(1185, 100));
        assert!(find_scale(&m).unwrap_err().contains("12057"));
        // the mods' own 1x backgrounds of other sizes are 1x sets, and so is a background
        // only wider or only taller, all its bitmaps even-sized
        m.insert(SCALE_ID, solid(888, 50));
        assert_eq!(find_scale(&m), Ok(1));
        m.insert(SCALE_ID, solid(1184, 52));
        assert_eq!(find_scale(&m), Ok(1));
        m.insert(12031, solid(15, 11));
        for (w, h) in [(608, 50), (609, 50), (593, 51), (592, 90), (300, 50)] {
            m.insert(SCALE_ID, solid(w, h));
            assert_eq!(find_scale(&m), Ok(1), "{w} x {h}");
        }
        m.insert(SCALE_ID, solid(1779, 153));
        m.insert(12031, solid(45, 33));
        assert_eq!(find_scale(&m), Ok(3));
    }

    #[test]
    fn upscale_repeats_pixels() {
        let mut img = ColorImage::filled([2, 1], Color32::BLACK);
        img.pixels[1] = Color32::WHITE;
        let up = upscale(&img, 2);
        assert_eq!(up.size, [4, 2]);
        assert_eq!(up.pixels[1], Color32::BLACK);
        assert_eq!(up.pixels[2], Color32::WHITE);
        assert_eq!(up.pixels[7], Color32::WHITE);
    }
}
