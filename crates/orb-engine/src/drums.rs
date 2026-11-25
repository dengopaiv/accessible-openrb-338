//! The drum voices shared by the TR-808 and the TR-909 (`docs/initial_investigation.md` section
//! 7.1): the classes `SIMPLE_VOICE` (vtable 0x10033D54), `FILTER_VOICE` (0x10033D64),
//! `DUAL_VOICE` (0x10033D74) and `PITCHED_VOICE` (0x10033D84).
//!
//! A voice plays one sample (two for [`Dual`]) of raw `i16` values. The 1/32768 factor is folded
//! into the gain at the trigger (`g = f32(gain * 2^-15)`, 0x1001E2CD). Each block runs 32 pairs
//! of samples: the gain is constant within a pair and is multiplied by `m` after it (by 1.0
//! instead while `hold` blocks remain). The loops keep the gain and the filter state in x87
//! registers and store them as `f32` only at the block end; every output sample is an `f32`
//! store of `out + voice` (the device's bus accumulates the voices in a fixed order).
//!
//! Field offsets are the DLL's, common to all classes: `+4` the sample object, `+8` active,
//! `+0xC` the sample pointer, `+0x10` the frames left, `+0x14` hold, `+0x18` the gain, `+0x1C` the
//! multiplier.

#[cfg(test)]
mod tests;

use std::sync::Arc;

use crate::frame::{AUDIBLE, Block, Split};
use crate::samples::{Retired, Sample};

/// `dbl_10153640`: 2^-15, the int16 scale folded into the gain (0x1001E2CD).
const SCALE: f64 = f64::from_bits(0x3F00_0000_0000_0000);
/// `flt_1015364C` (and the immediate `0x3300D959` at 0x1001E7E2): a gain below it ends a SIMPLE,
/// FILTER or DUAL voice.
pub const SILENT: f32 = f32::from_bits(0x3300_D959); // 3e-8
/// `flt_10153648`, `flt_101536A8`, `flt_10153710`, `flt_10153768`: the multiplier while held.
const HELD: f32 = 1.0;

fn st(x: f64) -> f32 {
    x as f32
}

/// The part every class shares (the `SIMPLE_VOICE` fields).
#[derive(Debug, Clone, Default)]
pub struct Voice {
    /// `+4`: the sample; a trigger without one keeps the last.
    pub sample: Option<Arc<Sample>>,
    /// `+8`: an inactive voice adds nothing.
    pub active: bool,
    /// `+0xC`: the read position (in samples) of the next block.
    pub pos: usize,
    /// `+0x10`: the frames left; the voice ends when it reaches 0 or less.
    pub left: i32,
    /// `+0x14`: the blocks the gain is held (multiplier 1.0).
    pub hold: u32,
    /// `+0x18`: the gain (the DUAL voice's `g1`).
    pub g: f32,
    /// `+0x1C`: the gain multiplier per pair (the DUAL voice's applies to `g2`).
    pub m: f32,
}

impl Voice {
    /// `sub_1001E2B4(sample, gain, m, hold)`: a non-null sample replaces the voice's; with a
    /// sample the voice restarts at its first frame with `g = f32(gain * 2^-15)`.
    pub fn trigger(&mut self, sample: Option<Arc<Sample>>, gain: f32, m: f32, hold: u32) {
        if sample.is_some() {
            self.sample = sample;
        }
        let Some(s) = &self.sample else { return };
        self.hold = hold;
        self.m = m;
        self.g = st(gain as f64 * SCALE);
        self.pos = 0;
        // `sub_1000AA80`: the frame count, `CWaveSample+0x4C`
        self.left = s.frames() as i32;
        self.active = true;
    }

    /// Slot 1, 0x1001EB30 (every class): inactive, gain 0.
    pub fn silence(&mut self) {
        self.active = false;
        self.g = 0.0;
    }

    /// Not in the original: hands the sample of a silenced voice to `r` (a mod change, which
    /// replaces the table the voices' samples come from). The original keeps the pointer; here
    /// the voice keeps nothing, which changes no output: an inactive voice reads no sample, and
    /// every trigger of the 808 and the 909 passes one.
    pub fn retire(sample: &mut Option<Arc<Sample>>, r: &mut Retired) {
        if let Some(s) = sample.take()
            && let Err(s) = r.keep_sample(s)
        {
            // no room: keep it (it is freed when the voice is next triggered)
            *sample = Some(s);
        }
    }

    /// The block's multiplier: 1.0 while held (one hold block used), else `m`.
    fn mult(&mut self) -> f64 {
        if self.hold != 0 {
            self.hold -= 1;
            HELD as f64
        } else {
            self.m as f64
        }
    }

    /// The block end shared by SIMPLE and FILTER: the gain test on the unrounded gain (`fcom`
    /// with the f32 3e-8), the f32 store, then the sample advances by 64 frames.
    fn end_block(&mut self, g: f64) {
        if g < SILENT as f64 {
            self.active = false;
        }
        self.g = st(g);
        self.advance();
    }

    /// Not in the original (task-200): the voice's output peak for a full-scale sample,
    /// `g * 2^15` (the gain with the int16 scale taken out again).
    pub fn peak(&self) -> f64 {
        self.g as f64 / SCALE
    }

    fn advance(&mut self) {
        self.pos += 64;
        self.left -= 64;
        if self.left <= 0 {
            self.active = false;
        }
    }
}

/// Where a voice adds its samples: every output sample is the `f32` store of `out + x`, `x` the
/// voice's term in f64 (the x87 `fadd` of the stored `f32` and the term, then `fstp dword`). An
/// IEEE sum does not depend on the operand order, so `x + out` stores the same.
pub trait Out {
    /// `out[i] = f32(out[i] + x)`.
    fn add(&mut self, i: usize, x: f64);
}

impl Out for Block {
    #[inline(always)]
    fn add(&mut self, i: usize, x: f64) {
        self[i] = st(self[i] as f64 + x);
    }
}

/// Not in the original (task-300, the plugin's per-drum outputs): a voice's samples into one
/// block, or the same terms into two (the machine's bus and a copy for the voice's output, so
/// the bus holds exactly the sum it holds without the copy).
pub enum Sink<'a> {
    One(&'a mut Block),
    Both(&'a mut Block, &'a mut Block),
}

impl Out for Sink<'_> {
    #[inline(always)]
    fn add(&mut self, i: usize, x: f64) {
        match self {
            Sink::One(b) => b.add(i, x),
            Sink::Both(a, b) => {
                a.add(i, x);
                b.add(i, x);
            }
        }
    }
}

/// One voice of a drum machine, of any class, in the machine's render order
/// ([`render_split`]).
pub enum VoiceMut<'a> {
    Simple(&'a mut Simple),
    Filter(&'a mut Filter),
    Dual(&'a mut Dual),
    Pitched(&'a mut Pitched),
}

impl VoiceMut<'_> {
    pub fn process<O: Out + ?Sized>(self, out: &mut O) {
        match self {
            VoiceMut::Simple(v) => v.process(out),
            VoiceMut::Filter(v) => v.process(out),
            VoiceMut::Dual(v) => v.process(out),
            VoiceMut::Pitched(v) => v.process(out),
        }
    }

    /// Slot 1 on the voice.
    pub fn silence(self) {
        match self {
            VoiceMut::Simple(v) => v.silence(),
            VoiceMut::Filter(v) => v.silence(),
            VoiceMut::Dual(v) => v.silence(),
            VoiceMut::Pitched(v) => v.silence(),
        }
    }

    /// For tests: a trigger of any class with sample `s`, a decaying gain and a hold block.
    #[cfg(test)]
    pub fn trigger_test(self, s: Arc<Sample>, gain: f32) {
        let m = 0.993;
        match self {
            VoiceMut::Simple(v) => v.trigger(Some(s), gain, m, 1),
            VoiceMut::Filter(v) => v.trigger(Some(s), gain, m, 1, 0.6),
            VoiceMut::Dual(v) => v.trigger(Some(s.clone()), gain, m, 1, Some(s), 0.7),
            VoiceMut::Pitched(v) => v.trigger(Some(s), gain, m, 1, 0x1A0),
        }
    }
}

/// One output of a drum machine's table (task-300): `id` is the plugin state's key, which never
/// changes once released; `name` (the routing window's box) and `bus` (the plugin's bus) are
/// labels, which may.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrumOutput {
    pub id: &'static str,
    pub name: &'static str,
    pub bus: &'static str,
}

/// Not in the original's export path (task-300; ReWire's per-instrument channels,
/// `docs/rewire.md` section 4): the machine's voices, in its render order, add into `bus`
/// (zeroed first) or into their own output. `map[k]` is voice k's output; `split.outs[o]`
/// (zeroed first) gets the voices of output o: only there when `split.to_bus[o]` is false (out
/// of the bus, which keeps the other voices in their order, so its f32 sum is the same sum
/// without the removed terms), and the same terms as the bus as well when it is true and
/// `split.copy` is set. With every output to the bus, the bus is bit-identical to the plain
/// render's.
pub fn render_split<const N: usize>(
    voices: [VoiceMut<'_>; N],
    map: &[usize; N],
    bus: &mut Block,
    split: Split<'_>,
) {
    bus.fill(0.0);
    for o in split.outs.iter_mut() {
        o.fill(0.0);
    }
    for (v, &o) in voices.into_iter().zip(map) {
        let own = &mut split.outs[o];
        if !split.to_bus[o] {
            v.process(own);
        } else if split.copy {
            v.process(&mut Sink::Both(bus, own));
        } else {
            v.process(bus);
        }
    }
}

fn padded(s: &Option<Arc<Sample>>) -> &[i16] {
    s.as_deref().map(Sample::padded).unwrap_or(&[])
}

/// Sample `i` of `d`, or 0 past the end (the original reads on into its padding and beyond;
/// with the built-in samples and the shipped tables every read stays inside the padding).
fn at(d: &[i16], i: usize) -> f64 {
    d.get(i).copied().unwrap_or(0) as f64
}

/// `SIMPLE_VOICE` (process 0x1001E300): `out += g*s`.
#[derive(Debug, Clone, Default)]
pub struct Simple {
    pub v: Voice,
}

impl Simple {
    pub fn trigger(&mut self, sample: Option<Arc<Sample>>, gain: f32, m: f32, hold: u32) {
        self.v.trigger(sample, gain, m, hold);
    }

    /// `sub_1001E3C0`: sets the hold count (the 808's closed hi-hat clears it on all five
    /// SIMPLE voices).
    pub fn set_hold(&mut self, hold: u32) {
        self.v.hold = hold;
    }

    /// `sub_1001E3D0`: sets the multiplier (the 909's closed hi-hat chokes the open one with
    /// 0.996).
    pub fn set_m(&mut self, m: f32) {
        self.v.m = m;
    }

    pub fn silence(&mut self) {
        self.v.silence();
    }

    /// Not in the original (task-200): active, with a peak at or above [`AUDIBLE`].
    pub fn audible(&self) -> bool {
        self.v.active && self.v.peak() >= AUDIBLE
    }

    /// [`Voice::retire`] on the voice's sample.
    pub fn retire(&mut self, r: &mut Retired) {
        Voice::retire(&mut self.v.sample, r);
    }

    pub fn process<O: Out + ?Sized>(&mut self, out: &mut O) {
        if !self.v.active {
            return;
        }
        let mult = self.v.mult();
        let mut g = self.v.g as f64;
        let d = padded(&self.v.sample);
        let p = self.v.pos;
        for k in 0..32 {
            let (i, j) = (2 * k, 2 * k + 1);
            let g2 = g * mult;
            let s1 = at(d, p + j);
            let s0 = at(d, p + i);
            out.add(j, s1 * g);
            out.add(i, g * s0);
            g = g2;
        }
        self.v.end_block(g);
    }
}

/// `FILTER_VOICE` (process 0x1001E4D0, ctor 0x1001E3E0): a one-pole low-pass, `y = y*c + g*s`,
/// `out += y`. `+0x20` is `c`, `+0x24` is `y`.
#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub v: Voice,
    pub c: f32,
    pub y: f32,
}

impl Filter {
    /// `sub_1001E5C0(sample, gain, m, hold, c)`: the SIMPLE trigger, then `y = 0` and the
    /// coefficient.
    pub fn trigger(&mut self, sample: Option<Arc<Sample>>, gain: f32, m: f32, hold: u32, c: f32) {
        self.v.trigger(sample, gain, m, hold);
        self.y = 0.0;
        self.c = c;
    }

    /// `sub_1001E3D0`: sets the multiplier (the 808 bass drum fades its other voice with it).
    pub fn set_m(&mut self, m: f32) {
        self.v.m = m;
    }

    pub fn silence(&mut self) {
        self.v.silence();
    }

    /// Not in the original (task-200): active, with a peak at or above [`AUDIBLE`]; the peak is
    /// the gain's over the one-pole's DC gain `1 / (1 - c)` (the gain itself when `c >= 1`).
    pub fn audible(&self) -> bool {
        let c = self.c as f64;
        let peak = if c < 1.0 {
            self.v.peak() / (1.0 - c)
        } else {
            self.v.peak()
        };
        self.v.active && peak >= AUDIBLE
    }

    /// [`Voice::retire`] on the voice's sample.
    pub fn retire(&mut self, r: &mut Retired) {
        Voice::retire(&mut self.v.sample, r);
    }

    pub fn process<O: Out + ?Sized>(&mut self, out: &mut O) {
        if !self.v.active {
            return;
        }
        let mult = self.v.mult();
        let mut g = self.v.g as f64;
        let mut y = self.y as f64;
        let c = self.c as f64;
        let d = padded(&self.v.sample);
        let p = self.v.pos;
        for k in 0..32 {
            let (i, j) = (2 * k, 2 * k + 1);
            let yc = y * c;
            let g2 = g * mult;
            let s1g = at(d, p + j) * g;
            let y0 = yc + g * at(d, p + i);
            let cy0 = c * y0;
            out.add(i, y0);
            let y1 = cy0 + s1g;
            out.add(j, y1);
            y = y1;
            g = g2;
        }
        self.y = st(y);
        self.v.end_block(g);
    }
}

/// `DUAL_VOICE` (process 0x1001E700, trigger 0x1001E820): two samples, `out += g1*(A + g2*B)`.
/// `g1` (`+0x18`) stays constant; the multiplier and the hold apply to `g2` (`+0x24`). `+0x20`
/// is B's read position, `+0x28` B's sample. It ends when the shorter sample does, or at once
/// when `g1` is below 3e-8.
#[derive(Debug, Clone, Default)]
pub struct Dual {
    pub v: Voice,
    pub b: Option<Arc<Sample>>,
    pub b_pos: usize,
    pub g2: f32,
}

impl Dual {
    /// `sub_1001E820(a, gain, m, hold, b, g2)`: the SIMPLE trigger with A, then B (a non-null
    /// one replaces the voice's) restarts with `g2` as given, and the frames left become
    /// `min(len A, len B)`.
    pub fn trigger(
        &mut self,
        a: Option<Arc<Sample>>,
        gain: f32,
        m: f32,
        hold: u32,
        b: Option<Arc<Sample>>,
        g2: f32,
    ) {
        self.v.trigger(a, gain, m, hold);
        if b.is_some() {
            self.b = b;
        }
        let Some(bs) = &self.b else { return };
        self.g2 = g2;
        self.b_pos = 0;
        let la = self.v.sample.as_deref().map_or(0, |s| s.frames() as i32);
        let lb = bs.frames() as i32;
        // 0x1001E883: `cmp lenA, lenB; jl` takes A's length, else B's
        self.v.left = if la < lb { la } else { lb };
        self.v.active = true;
    }

    pub fn silence(&mut self) {
        self.v.silence();
    }

    /// Not in the original (task-200): active, with a peak `g1 * (1 + g2)` at or above
    /// [`AUDIBLE`].
    pub fn audible(&self) -> bool {
        self.v.active && self.v.peak() * (1.0 + self.g2 as f64) >= AUDIBLE
    }

    /// [`Voice::retire`] on both samples.
    pub fn retire(&mut self, r: &mut Retired) {
        Voice::retire(&mut self.v.sample, r);
        Voice::retire(&mut self.b, r);
    }

    pub fn process<O: Out + ?Sized>(&mut self, out: &mut O) {
        if !self.v.active {
            return;
        }
        let mult = self.v.mult();
        let g1 = self.v.g as f64;
        let mut g2 = self.g2 as f64;
        let da = padded(&self.v.sample);
        let db = padded(&self.b);
        let (pa, pb) = (self.v.pos, self.b_pos);
        for k in 0..32 {
            let (i, j) = (2 * k, 2 * k + 1);
            let g2n = g2 * mult;
            let x0 = (at(da, pa + i) + at(db, pb + i) * g2) * g1;
            out.add(i, x0);
            let x1 = (g2 * at(db, pb + j) + at(da, pa + j)) * g1;
            out.add(j, x1);
            g2 = g2n;
        }
        self.g2 = st(g2);
        // 0x1001E7E2: the integer compare of g1's bits (stored back unchanged)
        if (self.v.g.to_bits() as i32) < SILENT.to_bits() as i32 {
            self.v.active = false;
        }
        self.b_pos += 64;
        self.v.advance();
    }
}

/// `PITCHED_VOICE` (process 0x1001E9A0, trigger 0x1001EAD0): the sample resampled with a 24.8
/// fixed-point phase and integer linear interpolation, `s = i16(a + ((f*(b - a)) >> 8))`.
/// `+0x24` is the step, `+0x28` the phase, `+0x2C` the end (`frames << 8`). It ends when the
/// phase passes the end; the gain is not tested.
#[derive(Debug, Clone, Default)]
pub struct Pitched {
    pub v: Voice,
    pub step: u32,
    pub phase: u32,
    pub end: u32,
}

impl Pitched {
    /// `sub_1001EAD0(sample, gain, m, hold, step)`.
    pub fn trigger(
        &mut self,
        sample: Option<Arc<Sample>>,
        gain: f32,
        m: f32,
        hold: u32,
        step: u32,
    ) {
        self.v.trigger(sample.clone(), gain, m, hold);
        self.step = step;
        self.phase = 0;
        // the frame count of the sample argument (`mov ecx, ebp` at 0x1001EB17)
        let frames = sample.as_deref().map_or(0, |s| s.frames() as u32);
        self.end = frames << 8;
    }

    pub fn silence(&mut self) {
        self.v.silence();
    }

    /// Not in the original (task-200): active, with a peak at or above [`AUDIBLE`].
    pub fn audible(&self) -> bool {
        self.v.active && self.v.peak() >= AUDIBLE
    }

    /// [`Voice::retire`] on the voice's sample.
    pub fn retire(&mut self, r: &mut Retired) {
        Voice::retire(&mut self.v.sample, r);
    }

    fn sample_at(d: &[i16], phase: u32) -> f64 {
        let f = (phase & 0xFF) as i32;
        let i = (phase >> 8) as usize;
        let a = d.get(i).copied().unwrap_or(0) as i32;
        let b = d.get(i + 1).copied().unwrap_or(0) as i32;
        (a + ((f * (b - a)) >> 8)) as i16 as f64
    }

    pub fn process<O: Out + ?Sized>(&mut self, out: &mut O) {
        if !self.v.active {
            return;
        }
        let mult = self.v.mult();
        let mut g = self.v.g as f64;
        let d = padded(&self.v.sample);
        let mut ph = self.phase;
        for k in 0..32 {
            let (i, j) = (2 * k, 2 * k + 1);
            let s0 = Self::sample_at(d, ph);
            ph = ph.wrapping_add(self.step);
            out.add(i, s0 * g);
            let g2 = g * mult;
            let s1 = Self::sample_at(d, ph);
            out.add(j, g * s1);
            ph = ph.wrapping_add(self.step);
            g = g2;
        }
        self.v.g = st(g);
        self.phase = ph;
        if ph > self.end {
            self.v.active = false;
        }
    }
}
