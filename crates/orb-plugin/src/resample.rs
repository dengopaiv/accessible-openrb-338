//! The deterministic resampler from the engine's 44.1 kHz to the host's rate (task-197).
//!
//! A fixed-ratio windowed-sinc interpolator: host frame `j` is the engine signal at the engine
//! position `x = j * 44100 / rate - delay`, kept as an exact rational (an integer and a
//! remainder), from the `2W` engine frames around it, each weighted by a Blackman-Harris
//! windowed sinc whose cutoff is 0.94 of the lower Nyquist frequency. The weights of each phase
//! are normalized to a sum of 1. Everything is computed in f64 from the frame index alone: the
//! output does not depend on how the host splits its buffers, and nothing reads a clock.
//!
//! `delay = 64 + W` engine frames: the frames a host frame needs end 64 frames before
//! `j * 44100 / rate`, so the engine block they lie in has been rendered with the host events
//! up to that host frame (`crate::core`). The plugin reports `delay * rate / 44100`, rounded up,
//! as its latency.

use crate::core::BLOCK;

/// The engine's rate.
pub const ENGINE_RATE: u32 = 44_100;

/// The kernel's half width in engine frames when the host rate is at least 44.1 kHz.
const HALF: usize = 32;

/// The phases precomputed at most (more distinct phases are computed per frame).
const MAX_TABLE_PHASES: u64 = 4096;

/// The cutoff, a fraction of the lower Nyquist frequency.
const CUTOFF: f64 = 0.94;

/// Where the engine's frames come from.
pub trait Blocks {
    /// The next engine block: [`BLOCK`] frames of interleaved stereo int16.
    fn next_block(&mut self) -> &[i16];
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// The weights of phase `p` (the fraction `p / den` of an engine frame) for a kernel of half
/// width `half` and cutoff `cutoff`: tap `i` is engine frame `base - half + 1 + i`.
fn weights(den: u64, half: usize, cutoff: f64, p: u64, w: &mut [f64]) {
    let frac = p as f64 / den as f64;
    let half = half as f64;
    let mut sum = 0.0;
    for (i, x) in w.iter_mut().enumerate() {
        let d = (i as f64 - half + 1.0) - frac;
        let t = d / half;
        let win = if t.abs() >= 1.0 {
            0.0
        } else {
            let a = std::f64::consts::PI * t;
            0.35875 + 0.48829 * a.cos() + 0.14128 * (2.0 * a).cos() + 0.01168 * (3.0 * a).cos()
        };
        let s = if d == 0.0 {
            1.0
        } else {
            let a = std::f64::consts::PI * cutoff * d;
            a.sin() / a
        };
        *x = cutoff * s * win;
        sum += *x;
    }
    for x in w.iter_mut() {
        *x /= sum;
    }
}

/// The resampler's state for one host rate.
pub struct Resampler {
    /// Engine frames per host frame, `num / den`, reduced.
    num: u64,
    den: u64,
    /// The kernel's half width `W` in engine frames.
    half: usize,
    /// The delay in engine frames (`64 + W`).
    delay: i64,
    /// The normalized cutoff (cycles per engine frame times 2).
    cutoff: f64,
    /// `den` phases of `2W` weights, when `den` is small enough.
    table: Option<Box<[f64]>>,
    /// The weights of one phase, computed per frame when there is no table.
    scratch: Box<[f64]>,
    /// The position of the next host frame: `pos + rem / den` engine frames, before the delay.
    pos: i64,
    rem: u64,
    /// The last engine frames, indexed by `frame & mask` (frames before 0 are silence).
    ring: Box<[[i16; 2]]>,
    mask: usize,
    /// The next engine frame to be written into the ring.
    next: i64,
}

impl Resampler {
    /// A resampler from 44.1 kHz to `rate` (not 44.1 kHz itself, which needs none).
    pub fn new(rate: u32) -> Resampler {
        let rate = rate.max(1) as u64;
        let g = gcd(ENGINE_RATE as u64, rate);
        let (num, den) = (ENGINE_RATE as u64 / g, rate / g);
        // below 44.1 kHz the cutoff falls to the host's Nyquist frequency and the kernel widens
        let ratio = num as f64 / den as f64;
        let scale = if ratio > 1.0 { 1.0 / ratio } else { 1.0 };
        let half = (HALF as f64 / scale).ceil() as usize;
        let cutoff = CUTOFF * scale;
        let cap = (2 * half + 2 * BLOCK).next_power_of_two();
        let mut r = Resampler {
            num,
            den,
            half,
            delay: (BLOCK + half) as i64,
            cutoff,
            table: None,
            scratch: vec![0.0; 2 * half].into_boxed_slice(),
            pos: 0,
            rem: 0,
            ring: vec![[0; 2]; cap].into_boxed_slice(),
            mask: cap - 1,
            next: 0,
        };
        if den <= MAX_TABLE_PHASES {
            let mut t = vec![0.0; den as usize * 2 * half];
            for (p, w) in t.chunks_mut(2 * half).enumerate() {
                weights(den, half, cutoff, p as u64, w);
            }
            r.table = Some(t.into_boxed_slice());
        }
        r
    }

    /// The delay in host frames, rounded up: what the plugin adds to its reported latency.
    pub fn latency(&self) -> u32 {
        // rounded up, so the reported latency is never shorter than the delay
        (self.delay as u64 * self.den).div_ceil(self.num) as u32
    }

    /// The engine frames the next host frame needs end here (inclusive).
    fn needs(&self) -> i64 {
        self.pos - self.delay + self.half as i64
    }

    /// Puts the next engine frame into the ring.
    fn push(&mut self, l: i16, r: i16) {
        self.ring[self.next as usize & self.mask] = [l, r];
        self.next += 1;
    }

    /// Fills `l` and `r` with host frames, pulling engine blocks from `engine` as they are
    /// needed.
    pub fn fill(&mut self, l: &mut [f32], r: &mut [f32], engine: &mut impl Blocks) {
        for (ol, or) in l.iter_mut().zip(r.iter_mut()) {
            while self.next <= self.needs() {
                let b = engine.next_block();
                for &[fl, fr] in b.as_chunks::<2>().0 {
                    self.push(fl, fr);
                }
            }
            let base = self.pos - self.delay;
            let first = base - self.half as i64 + 1;
            let n = 2 * self.half;
            let w: &[f64] = match &self.table {
                Some(t) => {
                    let p = self.rem as usize;
                    &t[p * n..(p + 1) * n]
                }
                None => {
                    weights(
                        self.den,
                        self.half,
                        self.cutoff,
                        self.rem,
                        &mut self.scratch,
                    );
                    &self.scratch
                }
            };
            let (mut sl, mut sr) = (0.0f64, 0.0f64);
            for (i, &k) in w.iter().enumerate() {
                let f = (first + i as i64) as usize & self.mask;
                let [a, b] = self.ring[f];
                sl += k * a as f64;
                sr += k * b as f64;
            }
            *ol = (sl / 32768.0) as f32;
            *or = (sr / 32768.0) as f32;
            self.rem += self.num;
            self.pos += (self.rem / self.den) as i64;
            self.rem %= self.den;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sine of 100 engine frames, left, and its negation, right.
    struct Sine {
        k: u64,
        block: [i16; 2 * BLOCK],
    }

    impl Blocks for Sine {
        fn next_block(&mut self) -> &[i16] {
            for f in self.block.as_chunks_mut::<2>().0 {
                let a = 2.0 * std::f64::consts::PI * self.k as f64 / 100.0;
                let v = (10000.0 * a.sin()) as i16;
                f[0] = v;
                f[1] = -v;
                self.k += 1;
            }
            &self.block
        }
    }

    fn run(rate: u32, frames: usize, chunk: usize) -> Vec<f32> {
        let mut r = Resampler::new(rate);
        let mut sine = Sine {
            k: 0,
            block: [0; 2 * BLOCK],
        };
        let (mut l, mut rr) = (vec![0f32; frames], vec![0f32; frames]);
        let mut i = 0;
        while i < frames {
            let n = chunk.min(frames - i);
            r.fill(&mut l[i..i + n], &mut rr[i..i + n], &mut sine);
            i += n;
        }
        l.into_iter().zip(rr).flat_map(|(a, b)| [a, b]).collect()
    }

    #[test]
    fn the_ratio_is_reduced_and_the_latency_rounded() {
        let r = Resampler::new(48_000);
        assert_eq!((r.num, r.den, r.half), (147, 160, 32));
        // (64 + 32) * 48000 / 44100 = 104.49
        assert_eq!(r.latency(), 105);
        let r = Resampler::new(96_000);
        assert_eq!(r.latency(), 209);
        let r = Resampler::new(22_050);
        assert_eq!((r.num, r.den, r.half), (2, 1, 64));
        assert_eq!(r.latency(), 64);
    }

    #[test]
    fn the_output_does_not_depend_on_the_chunks() {
        for rate in [48_000, 22_050, 96_000, 44_101] {
            let a = run(rate, 5000, 5000);
            for chunk in [1, 7, 64, 333] {
                assert_eq!(a, run(rate, 5000, chunk), "rate {rate}, chunk {chunk}");
            }
        }
    }

    #[test]
    fn a_sine_comes_out_delayed_by_the_latency() {
        // 441 Hz at 44.1 kHz (period 100 frames) is 441 Hz at 48 kHz: period 48000/441 frames
        let out = run(48_000, 4000, 4000);
        let r = Resampler::new(48_000);
        let d = r.delay as f64 * 48_000.0 / 44_100.0;
        let mut worst = 0.0f64;
        for j in 500..4000 {
            let t = (j as f64 - d) * 44_100.0 / 48_000.0;
            let want = 10000.0 * (2.0 * std::f64::consts::PI * t / 100.0).sin() / 32768.0;
            worst = worst.max((out[2 * j] as f64 - want).abs());
            assert_eq!(out[2 * j], -out[2 * j + 1]);
        }
        // the int16 truncation of the source alone is up to 1/32768
        assert!(worst < 2e-4, "worst error {worst}");
    }
}
