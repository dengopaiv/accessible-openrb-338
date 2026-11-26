//! The crossfade between the modern and the classic skin (task-268, docs/ui_spec.md section 22).
//!
//! The classic panel is one opaque texture, so the fade draws the modern skin and the classic
//! texture over it with the fade's alpha: 0 is all modern, 1 is all classic.

/// How long a full fade (alpha 0 to 1 or back) takes, seconds. The one place to tune it.
pub const FADE_SECONDS: f64 = 0.5;

/// The ease: smoothstep.
pub fn ease(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A fade of the classic overlay's alpha from `from` to `to`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fade {
    pub from: f32,
    pub to: f32,
    /// The egui time of the first frame drawn, None before it.
    start: Option<f64>,
}

impl Fade {
    /// A fade from alpha `from` to `to` (1 classic, 0 modern).
    pub fn new(from: f32, to: f32) -> Self {
        Self {
            from,
            to,
            start: None,
        }
    }

    /// How long the fade takes: a full fade's time scaled by the distance, so that a reversal
    /// from the middle moves at the same speed.
    pub fn seconds(&self) -> f64 {
        FADE_SECONDS * f64::from((self.to - self.from).abs())
    }

    /// The alpha at egui time `now`; the first call starts the clock.
    pub fn alpha(&mut self, now: f64) -> f32 {
        let start = *self.start.get_or_insert(now);
        let d = self.seconds();
        if d <= 0.0 {
            return self.to;
        }
        let k = ease((now - start) / d) as f32;
        self.from + (self.to - self.from) * k
    }

    /// Whether the fade has reached its end at `now` (after [`Fade::alpha`] of that time).
    pub fn done(&self, now: f64) -> bool {
        self.start.is_some_and(|s| now - s >= self.seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_alpha_starts_ends_and_rises_monotonically_with_the_ease() {
        let mut f = Fade::new(0.0, 1.0);
        assert_eq!(f.alpha(10.0), 0.0);
        assert!(!f.done(10.0));
        let mut last = 0.0;
        for i in 1..=100 {
            let a = f.alpha(10.0 + FADE_SECONDS * f64::from(i) / 100.0);
            assert!(a >= last);
            last = a;
        }
        assert_eq!(last, 1.0);
        assert!(f.done(10.0 + FADE_SECONDS));
        // smoothstep: half way at the middle, slow at the ends
        let mut f = Fade::new(0.0, 1.0);
        f.alpha(0.0);
        assert!((f.alpha(FADE_SECONDS / 2.0) - 0.5).abs() < 1e-6);
        assert!(f.alpha(FADE_SECONDS * 0.1) < 0.1);
        assert!(f.alpha(FADE_SECONDS * 0.9) > 0.9);
        // fading out ends at 0, falling
        let mut f = Fade::new(1.0, 0.0);
        f.alpha(0.0);
        assert!(f.alpha(0.1) < 1.0);
        assert_eq!(f.alpha(5.0), 0.0);
    }

    #[test]
    fn a_reversal_continues_from_the_current_alpha() {
        let mut f = Fade::new(0.0, 1.0);
        f.alpha(0.0);
        let mid = f.alpha(FADE_SECONDS * 0.3);
        let mut r = Fade::new(mid, 0.0);
        assert_eq!(r.alpha(1.0), mid);
        assert!(r.seconds() < FADE_SECONDS);
        assert!(r.alpha(1.0 + r.seconds() / 2.0) < mid);
        assert_eq!(r.alpha(2.0), 0.0);
        assert!(r.done(2.0));
    }
}
