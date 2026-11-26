//! The audio output: an [`Audio`] in a cpal output stream (task-040 chose cpal).
//!
//! The stream runs at 44.1 kHz whenever the device offers it, and then the engine's samples go
//! out unchanged (`x / 32768` as f32, converted to the device's sample format by cpal's
//! `FromSample`). A device without 44.1 kHz gets the engine's output resampled to its default
//! rate by rubato's asynchronous sinc resampler ([`rubato::Async::new_sinc`], 128 taps, a
//! Blackman-Harris window, 256x oversampling, cubic interpolation), fed one engine call (1024
//! frames) at a time (`FixedAsync::Input`); its buffers are allocated when the stream is built
//! and `process_into_buffer` allocates nothing. Stereo goes to the first two channels of a
//! device with more (the others are zero), and the mean of both to a mono device.
//!
//! The device buffer is requested as [`BUFFER_FRAMES`] (task-176; the reason is on that constant).
//!
//! The offline render does not use any of this: it is [`orb_engine::frame::export`].

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{
    BufferSize, ErrorKind, FromSample, SampleFormat, SizedSample, StreamConfig, SupportedBufferSize,
};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{
    Async, FixedAsync, Resampler as _, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

use crate::devices;
pub use crate::devices::Request;
use crate::{Audio, CALL_FRAMES, RATE};

/// The frames converted per step of a device callback.
const SCRATCH_FRAMES: usize = 1024;

/// An error of [`start`].
#[derive(Debug)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

fn err(what: &str, e: impl std::fmt::Display) -> Error {
    Error(format!("{what}: {e}"))
}

/// A running output stream; dropping it stops the sound.
pub struct Stream {
    stream: cpal::Stream,
    /// The engine, handed back by the callback when the stream is dropped ([`Stream::stop`]).
    recover: rtrb::Consumer<Audio>,
    /// The driver (cpal host) the stream runs on, e.g. `CoreAudio`.
    pub driver: String,
    /// The device's id ([`devices::device_id`]), the one [`Request::device`] names.
    pub device_id: String,
    /// Whether the stream was asked for on the system default device ([`Request::device`] is
    /// `None`): the one that follows the system.
    pub follows_default: bool,
    /// The device's name.
    pub device: String,
    /// The stream's configuration.
    pub config: StreamConfig,
    /// The stream's sample format.
    pub format: SampleFormat,
    /// Whether the engine's output is resampled (the device does not offer 44.1 kHz).
    pub resampled: bool,
    /// The buffer asked for, in frames (`None`: the device's default buffer; see
    /// [`BUFFER_FRAMES`]).
    pub requested: Option<u32>,
    health: Health,
    stats: CallbackStats,
    /// Marks an ASIO driver as held while the stream lives ([`devices::DriverHold`]).
    _hold: Option<devices::DriverHold>,
}

impl Stream {
    /// The buffer the device granted, in frames: the size of the first data callback (`None`
    /// before it ran). cpal exposes no period getter, and the hosts hand out whole periods.
    pub fn granted(&self) -> Option<u32> {
        Some(self.stats.snapshot().first).filter(|&n| n > 0)
    }

    /// Stops the stream and takes the engine back, so that another stream can run it. `None`
    /// when the callback did not hand it back within two seconds (the stream's thread is stuck).
    pub fn stop(self) -> Option<Audio> {
        let Stream {
            stream,
            mut recover,
            ..
        } = self;
        drop(stream);
        let t = Instant::now();
        loop {
            if let Ok(a) = recover.pop() {
                return Some(a);
            }
            if t.elapsed().as_millis() > 2000 {
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }

    /// What is in use, for the Audio Output window.
    pub fn in_use(&self) -> InUse {
        InUse {
            driver: self.driver.clone(),
            device: self.device.clone(),
            follows_default: self.follows_default,
            rate: self.config.sample_rate,
            resampled: self.resampled,
            channels: self.config.channels,
            format: self.format.to_string(),
            requested: self.requested,
            granted: self.granted(),
            output_latency_ns: self.stats.latency_ns(),
            underruns: self.health.underruns(),
        }
    }

    /// The data callback's statistics, which outlive borrowing the stream.
    pub fn stats(&self) -> CallbackStats {
        self.stats.clone()
    }

    /// The start line's buffer part, e.g. `buffer 2048 frames requested, first callback 2048
    /// frames`. The size seen is the first callback's, which on ALSA is the frames available and
    /// not necessarily the period, so the line does not call it "granted".
    pub fn buffer_text(&self) -> String {
        let req = match self.requested {
            Some(n) => format!("{n} frames requested"),
            None => "default size requested".into(),
        };
        match self.granted() {
            Some(n) => format!("buffer {req}, first callback {n} frames"),
            None => format!("buffer {req}, no callback yet"),
        }
    }

    /// Whether the sound has really stopped ([`Health::failure`]).
    pub fn failed(&self) -> bool {
        self.health.failure().is_some()
    }

    /// Why the sound has stopped, if it has ([`Health::failure`]).
    pub fn failure(&self) -> Option<String> {
        self.health.failure()
    }

    /// A handle on the stream's health that outlives borrowing the stream (the UI keeps it).
    pub fn health(&self) -> Health {
        self.health.clone()
    }
}

/// What a running stream is using ([`Stream::in_use`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InUse {
    pub driver: String,
    pub device: String,
    /// The stream follows the system default device.
    pub follows_default: bool,
    /// The device's rate, in Hz.
    pub rate: u32,
    /// The engine's 44.1 kHz output is resampled to `rate`.
    pub resampled: bool,
    pub channels: u16,
    pub format: String,
    /// The buffer asked for, in frames (`None`: the device's default).
    pub requested: Option<u32>,
    /// The first callback's size, in frames (`None` before it ran).
    pub granted: Option<u32>,
    /// The last callback's predicted playback time minus its call time, in nanoseconds, as the
    /// host reports it (`None` before a callback, or when the host gives none).
    pub output_latency_ns: Option<u64>,
    /// Xrun errors and callback gaps clearly longer than the buffer since the stream opened.
    pub underruns: u64,
}

/// How long the data callback may stay silent after a stream error before the sound counts as
/// stopped.
pub const STALL_MS: u64 = 500;

/// The words (8 bytes each) of an error text [`Health`] keeps.
const TEXT_WORDS: usize = 16;

/// The shortest time between two log lines about stream errors.
pub const LOG_EVERY_MS: u64 = 1000;

/// The rule that turns the stream's error reports into "the sound stopped" (task-139). cpal's
/// ALSA worker (cpal 0.18.2, src/host/alsa/mod.rs:1015-1018, 1004-1010) reports every error of
/// a cycle (e.g. WSLg's PulseAudio plugin returning EIO from `snd_pcm_avail_delay`) through the
/// error callback and goes on polling and calling the data callback; only a device that is gone
/// (`DeviceNotAvailable`, mod.rs:1015-1018) ends the worker. So an error alone does not mean
/// the sound stopped:
///
/// - `DeviceNotAvailable`, `HostUnavailable`, `StreamInvalidated`: fail at once, the stream cannot
///   go on;
/// - every other kind (`Xrun`, `RealtimeDenied`, `DeviceChanged`, `DeviceBusy` included) never
///   fails on the error alone; it sets a pending error, and the sound counts as stopped when the
///   data callback then does not run for [`STALL_MS`] (an error is pending and the last data
///   callback is that old). A data callback clears the pending error.
///
/// A silent stop with no error reported at all is not detected: nothing is pending then.
///
/// The data callback only reads the clock and stores atomics ([`Health::data`]); the error
/// callback allocates (the error text), which is outside the audio thread's render path.
///
/// Times are milliseconds from any fixed origin, passed in by the caller (a clock in the stream,
/// numbers in the tests). All fields are atomics: the data callback runs on the audio thread.
#[derive(Clone)]
pub struct Health(Arc<HealthInner>);

struct HealthInner {
    /// The time of the last data callback.
    last_data: AtomicU64,
    /// Whether an error came after the last data callback.
    pending: AtomicBool,
    /// Set by a device-gone error.
    fatal: AtomicBool,
    /// The underruns counted since the stream opened ([`Health::underrun`]).
    underruns: AtomicU64,
    /// The last error's text (cut to [`TEXT_WORDS`] * 8 bytes), packed little-endian into
    /// atomics so that the audio thread's code holds no lock (tests/no_alloc.rs); a read racing
    /// a write may show a mix of two texts, which only the message sees.
    text: [AtomicU64; TEXT_WORDS],
    text_len: AtomicUsize,
    /// The errors not logged yet, and the time of the last log line (`NO_LOG` for none).
    held: AtomicU64,
    last_log: AtomicU64,
    /// The stream's clock origin.
    origin: Instant,
}

impl Health {
    /// A rule whose stream started at `now` ms.
    fn new(now: u64) -> Self {
        Health(Arc::new(HealthInner {
            last_data: AtomicU64::new(now),
            pending: AtomicBool::new(false),
            fatal: AtomicBool::new(false),
            underruns: AtomicU64::new(0),
            text: std::array::from_fn(|_| AtomicU64::new(0)),
            text_len: AtomicUsize::new(0),
            held: AtomicU64::new(0),
            last_log: AtomicU64::new(NO_LOG),
            origin: Instant::now(),
        }))
    }

    /// A rule for tests and for [`start`], starting at 0.
    pub fn started() -> Self {
        Self::new(0)
    }

    /// Milliseconds since the stream was built.
    pub fn now(&self) -> u64 {
        self.0.origin.elapsed().as_millis() as u64
    }

    /// The data callback ran at `now`.
    pub fn data(&self, now: u64) {
        // the time first, then the flag: a reader that sees the flag clear sees this time
        self.0.last_data.store(now, Ordering::Relaxed);
        self.0.pending.store(false, Ordering::Release);
    }

    /// One underrun: an xrun error, or a callback gap clearly longer than the buffer
    /// ([`is_gap`]). Atomics only, so the audio thread may call it.
    pub fn underrun(&self) {
        self.0.underruns.fetch_add(1, Ordering::Relaxed);
    }

    /// The underruns since the stream opened.
    pub fn underruns(&self) -> u64 {
        self.0.underruns.load(Ordering::Relaxed)
    }

    /// Whether the stream's device is gone for good (a fatal error was reported), as against a
    /// stall: the one case where reopening on another device helps.
    pub fn device_gone(&self) -> bool {
        self.0.fatal.load(Ordering::Relaxed)
    }

    /// The error callback got an error of `kind` with `text`.
    pub fn error(&self, kind: ErrorKind, text: &str) {
        if kind == ErrorKind::Xrun {
            self.underrun();
        }
        match kind {
            ErrorKind::DeviceNotAvailable
            | ErrorKind::HostUnavailable
            | ErrorKind::StreamInvalidated => self.0.fatal.store(true, Ordering::Relaxed),
            _ => self.0.pending.store(true, Ordering::Release),
        }
        let mut b = text.as_bytes();
        // cut at a character boundary
        let mut n = b.len().min(TEXT_WORDS * 8);
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        b = &b[..n];
        for (i, w) in self.0.text.iter().enumerate() {
            let mut x = [0u8; 8];
            let part = b.get(i * 8..).unwrap_or(&[]);
            let k = part.len().min(8);
            x[..k].copy_from_slice(&part[..k]);
            w.store(u64::from_le_bytes(x), Ordering::Relaxed);
        }
        self.0.text_len.store(n, Ordering::Relaxed);
    }

    fn last_error(&self) -> String {
        let n = self.0.text_len.load(Ordering::Relaxed).min(TEXT_WORDS * 8);
        let mut v = Vec::with_capacity(TEXT_WORDS * 8);
        for w in &self.0.text {
            v.extend_from_slice(&w.load(Ordering::Relaxed).to_le_bytes());
        }
        v.truncate(n);
        String::from_utf8_lossy(&v).into_owned()
    }

    /// Why the sound has stopped at `now`, if it has.
    pub fn failure_at(&self, now: u64) -> Option<String> {
        let last = self.last_error();
        if self.0.fatal.load(Ordering::Relaxed) {
            return Some(format!("the audio device is gone or unusable: {last}"));
        }
        // the flag first (Acquire), then the time it was set against
        if self.0.pending.load(Ordering::Acquire) {
            let silent = now.saturating_sub(self.0.last_data.load(Ordering::Relaxed));
            if silent >= STALL_MS {
                return Some(format!(
                    "the audio output stopped ({silent} ms without sound): {last}"
                ));
            }
        }
        None
    }

    /// [`Health::failure_at`] now; it also writes the log line for errors held back
    /// ([`Health::flush_log`]), so the end of a burst is reported by whoever polls.
    pub fn failure(&self) -> Option<String> {
        let now = self.now();
        if let Some(n) = self.flush_log(now) {
            eprintln!(
                "orb-player: audio output: {n} more errors (last: {})",
                self.last_error()
            );
        }
        self.failure_at(now)
    }

    /// An error at `now` to log: `Some(n)` when a line is due, for this error and the `n - 1`
    /// held back since the previous line; `None` when it is held back (one line per
    /// [`LOG_EVERY_MS`]).
    pub fn note_error(&self, now: u64) -> Option<u64> {
        self.0.held.fetch_add(1, Ordering::Relaxed);
        let last = self.0.last_log.load(Ordering::Relaxed);
        if last != NO_LOG && now.saturating_sub(last) < LOG_EVERY_MS {
            return None;
        }
        self.0.last_log.store(now, Ordering::Relaxed);
        Some(self.0.held.swap(0, Ordering::Relaxed))
    }

    /// The count of errors held back when a second has passed since the last line (the burst
    /// ended or went on without a new error), else `None`.
    pub fn flush_log(&self, now: u64) -> Option<u64> {
        let last = self.0.last_log.load(Ordering::Relaxed);
        if last == NO_LOG || now.saturating_sub(last) < LOG_EVERY_MS {
            return None;
        }
        match self.0.held.swap(0, Ordering::Relaxed) {
            0 => None,
            n => {
                self.0.last_log.store(now, Ordering::Relaxed);
                Some(n)
            }
        }
    }
}

/// `last_log` before the first line.
const NO_LOG: u64 = u64::MAX;

/// The engine's output at the device's rate, channel count and sample format: the body of the
/// stream's callback, usable without a device.
pub struct Output {
    audio: Audio,
    channels: usize,
    resampler: Option<Resample>,
    scratch: Box<[f32; 2 * SCRATCH_FRAMES]>,
}

/// The resampler and its buffers.
struct Resample {
    rs: Async<f32>,
    input: Vec<f32>,
    output: Vec<f32>,
    /// The frames of `output` the last chunk produced, and those already handed out.
    len: usize,
    pos: usize,
    /// [`Output::inject_resampler_error`].
    fail_next: bool,
}

impl Resample {
    fn new(rate: u32) -> Result<Self, Error> {
        let params = SincInterpolationParameters::new(128, WindowFunction::BlackmanHarris2)
            .oversampling_factor(256)
            .interpolation(SincInterpolationType::Cubic);
        let rs = Async::<f32>::new_sinc(
            rate as f64 / RATE as f64,
            1.1,
            &params,
            CALL_FRAMES,
            2,
            FixedAsync::Input,
        )
        .map_err(|e| err("the resampler", e))?;
        let max = rs.output_frames_max();
        Ok(Resample {
            rs,
            input: vec![0.0; 2 * CALL_FRAMES],
            output: vec![0.0; 2 * max],
            len: 0,
            pos: 0,
            fail_next: false,
        })
    }

    /// One engine call through the resampler into `output`; the frames written, or `None` when
    /// the buffers or the resampler refuse (the sizes are the resampler's own, so only a defect
    /// or [`Output::inject_resampler_error`] gets there).
    fn chunk(&mut self, audio: &mut Audio) -> Option<usize> {
        audio.render_f32(&mut self.input);
        // a forced error: an output buffer too small for the resampler
        let max = if self.fail_next {
            self.fail_next = false;
            0
        } else {
            self.output.len() / 2
        };
        let inp = InterleavedSlice::new(&self.input[..], 2, CALL_FRAMES).ok()?;
        let mut outp = InterleavedSlice::new_mut(&mut self.output[..], 2, max).ok()?;
        let (_, written) = self.rs.process_into_buffer(&inp, &mut outp, None).ok()?;
        Some(written)
    }

    /// Fills `out` (interleaved stereo) at the device rate. On a resampler error it returns
    /// `false` and the caller silences the rest of the device callback; the engine call of the
    /// failed chunk is lost and the next callback starts a new chunk.
    fn pull(&mut self, audio: &mut Audio, out: &mut [f32]) -> bool {
        let frames = out.len() / 2;
        let mut i = 0;
        while i < frames {
            if self.pos == self.len {
                self.pos = 0;
                match self.chunk(audio) {
                    Some(n) => self.len = n,
                    None => {
                        self.len = 0;
                        return false;
                    }
                }
                continue;
            }
            let n = (self.len - self.pos).min(frames - i);
            out[2 * i..2 * (i + n)].copy_from_slice(&self.output[2 * self.pos..2 * (self.pos + n)]);
            self.pos += n;
            i += n;
        }
        true
    }
}

impl Output {
    /// `audio` for a device of `channels` channels at `rate` Hz (resampled unless 44100).
    pub fn new(audio: Audio, channels: usize, rate: u32) -> Result<Self, Error> {
        let resampler = Self::resampler(channels, rate)?;
        Ok(Self::with(audio, channels, resampler))
    }

    /// The resampler a device of `channels` channels at `rate` Hz needs (`None` at 44100): the
    /// part of [`Output::new`] that can fail, so that [`open`] can make it before it hands the
    /// engine over and get the engine back from a refused stream.
    fn resampler(channels: usize, rate: u32) -> Result<Option<Resample>, Error> {
        if channels == 0 {
            return Err(Error("a device with no channels".into()));
        }
        if rate == RATE {
            Ok(None)
        } else {
            Resample::new(rate).map(Some)
        }
    }

    fn with(audio: Audio, channels: usize, resampler: Option<Resample>) -> Self {
        Output {
            audio,
            channels,
            resampler,
            scratch: Box::new([0.0; 2 * SCRATCH_FRAMES]),
        }
    }

    /// The callback: fills `data` (interleaved, the device's channels). A resampler error makes
    /// the rest of this callback silent and is counted ([`crate::Snapshot::output_errors`]).
    pub fn fill<T: SizedSample + FromSample<f32>>(&mut self, data: &mut [T]) {
        let ch = self.channels;
        let silent = T::from_sample(0.0f32);
        let mut failed = false;
        for chunk in data.chunks_mut(ch * SCRATCH_FRAMES) {
            if failed {
                chunk.fill(silent);
                continue;
            }
            let frames = chunk.len() / ch;
            let s = &mut self.scratch[..2 * frames];
            match &mut self.resampler {
                None => self.audio.render_f32(s),
                Some(r) => {
                    if !r.pull(&mut self.audio, s) {
                        failed = true;
                        self.audio.output_error();
                        chunk.fill(silent);
                        continue;
                    }
                }
            }
            for (f, lr) in chunk.chunks_exact_mut(ch).zip(s.as_chunks::<2>().0) {
                if ch == 1 {
                    f[0] = T::from_sample((lr[0] + lr[1]) * 0.5);
                } else {
                    f[0] = T::from_sample(lr[0]);
                    f[1] = T::from_sample(lr[1]);
                    for x in &mut f[2..] {
                        *x = silent;
                    }
                }
            }
        }
    }

    pub fn audio(&self) -> &Audio {
        &self.audio
    }

    /// For tests: the resampled frames not yet handed out are dropped, and the next resampler
    /// chunk (made by the next [`Output::fill`]) gets an output buffer too small, so the
    /// resampler returns an error. Returns `false` when this output does not resample.
    #[doc(hidden)]
    pub fn inject_resampler_error(&mut self) -> bool {
        match &mut self.resampler {
            Some(r) => {
                r.pos = r.len;
                r.fail_next = true;
                true
            }
            None => false,
        }
    }
}

/// The device buffer [`start`] asks for, in frames (task-176). With `BufferSize::Default` cpal's
/// ALSA host (cpal 0.18.2, src/host/alsa/mod.rs:1642-1656) keeps the period the plugin picks and
/// caps the ring at two periods; behind pipewire-alsa a graph cycle can pull more than that, and
/// every cycle then underruns (GitHub issue #7: an `Xrun` 23-24 times a second). With
/// `BufferSize::Fixed(n)` cpal sets the period to `n` and the ring to `2n` (mod.rs:1600-1636), so
/// 2048 frames give a ring of about 93 ms at 44.1 kHz. The engine's timing does not change
/// (commands still apply per [`CALL_FRAMES`] engine call); the buffer only adds latency. It is
/// requested on every host (CoreAudio and WASAPI take it as a request) and clamped into the
/// device's range; a device that does not state a range, or refuses the request, gets
/// `BufferSize::Default`.
pub const BUFFER_FRAMES: u32 = 2048;

/// The buffer to request from a device whose buffer range is `range`: [`BUFFER_FRAMES`] clamped
/// into a stated range, `None` (the default buffer) when the range is unknown.
pub fn buffer_request(range: &SupportedBufferSize) -> Option<u32> {
    buffer_request_for(BUFFER_FRAMES, range)
}

/// [`buffer_request`] for a buffer of `want` frames.
pub fn buffer_request_for(want: u32, range: &SupportedBufferSize) -> Option<u32> {
    match *range {
        SupportedBufferSize::Range { min, max } if min <= max => Some(want.clamp(min, max)),
        _ => None,
    }
}

/// Whether a data callback that came `gap_ns` after the previous one, with `frames` frames at
/// `rate` Hz, is a gap clearly longer than the buffer: more than two buffers and 20 ms. A
/// callback that late means the device ran dry.
pub fn is_gap(gap_ns: u64, frames: u32, rate: u32) -> bool {
    let buffer_ns = frames as u64 * 1_000_000_000 / rate.max(1) as u64;
    gap_ns > 2 * buffer_ns + 20_000_000
}

/// The data callback's bookkeeping around the engine call: the gap check, the playback latency
/// the host reports, the statistics and the health clock. Atomics and the clock only, so it
/// allocates nothing (tests/no_alloc.rs calls it inside the armed region); the callback of
/// [`start`] and [`open`] is `enter`, the engine call, `leave`.
pub struct CallbackBook {
    health: Health,
    stats: CallbackStats,
    rate: u32,
    last: Option<Instant>,
}

impl CallbackBook {
    pub fn new(health: Health, stats: CallbackStats, rate: u32) -> Self {
        CallbackBook {
            health,
            stats,
            rate,
            last: None,
        }
    }

    /// A callback of `frames` frames starts at `now`; the host predicts the data plays
    /// `latency` later. A callback far later than its buffer ([`is_gap`]) is an underrun the
    /// device did not report.
    pub fn enter(&mut self, now: Instant, frames: u32, latency: std::time::Duration) {
        if let Some(prev) = self.last
            && is_gap(
                now.duration_since(prev).as_nanos() as u64,
                frames,
                self.rate,
            )
        {
            self.health.underrun();
        }
        self.last = Some(now);
        self.stats.note_latency(latency.as_nanos() as u64);
    }

    /// The callback ended after `ns` nanoseconds.
    pub fn leave(&self, frames: u32, ns: u64) {
        self.stats.record(frames, ns);
        self.health.data(self.health.now());
    }
}

/// `latency_ns` before the first callback.
const NO_LATENCY: u64 = u64::MAX;

/// What the data callback saw: all atomics, written with no allocation or lock.
struct StatsInner {
    calls: AtomicU64,
    first: AtomicU32,
    min: AtomicU32,
    max: AtomicU32,
    longest_ns: AtomicU64,
    /// The last callback's playback minus callback time (`NO_LATENCY` for none).
    latency_ns: AtomicU64,
}

/// The data callback's statistics ([`Stream::stats`]).
#[derive(Clone)]
pub struct CallbackStats(Arc<StatsInner>);

/// A copy of [`CallbackStats`] at one time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackSnapshot {
    /// The data callbacks run.
    pub calls: u64,
    /// The frames of the first callback (the buffer the device granted, when it hands out whole
    /// periods), 0 before it.
    pub first: u32,
    /// The fewest and the most frames of a callback.
    pub min: u32,
    pub max: u32,
    /// The longest callback, in nanoseconds.
    pub longest_ns: u64,
}

impl Default for CallbackStats {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackStats {
    pub fn new() -> Self {
        CallbackStats(Arc::new(StatsInner {
            calls: AtomicU64::new(0),
            first: AtomicU32::new(0),
            min: AtomicU32::new(u32::MAX),
            max: AtomicU32::new(0),
            longest_ns: AtomicU64::new(0),
            latency_ns: AtomicU64::new(NO_LATENCY),
        }))
    }

    /// The playback time minus the call time the host gave the last callback (audio thread: one
    /// store).
    fn note_latency(&self, ns: u64) {
        self.0.latency_ns.store(ns, Ordering::Relaxed);
    }

    /// The last [`CallbackStats::note_latency`] value, if any.
    pub fn latency_ns(&self) -> Option<u64> {
        Some(self.0.latency_ns.load(Ordering::Relaxed)).filter(|&n| n != NO_LATENCY)
    }

    /// One callback of `frames` frames that took `ns` nanoseconds (audio thread; the stores only
    /// race with readers).
    fn record(&self, frames: u32, ns: u64) {
        let s = &*self.0;
        if s.calls.fetch_add(1, Ordering::Relaxed) == 0 {
            s.first.store(frames, Ordering::Relaxed);
        }
        s.min.fetch_min(frames, Ordering::Relaxed);
        s.max.fetch_max(frames, Ordering::Relaxed);
        s.longest_ns.fetch_max(ns, Ordering::Relaxed);
    }

    /// The statistics now.
    pub fn snapshot(&self) -> CallbackSnapshot {
        let s = &*self.0;
        let calls = s.calls.load(Ordering::Relaxed);
        CallbackSnapshot {
            calls,
            first: s.first.load(Ordering::Relaxed),
            min: if calls == 0 {
                0
            } else {
                s.min.load(Ordering::Relaxed)
            },
            max: s.max.load(Ordering::Relaxed),
            longest_ns: s.longest_ns.load(Ordering::Relaxed),
        }
    }
}

impl CallbackSnapshot {
    /// The exit line of `orb play`: the callback sizes and the longest callback.
    pub fn summary(&self) -> String {
        if self.calls == 0 {
            return "no audio callback ran".into();
        }
        format!(
            "{} callbacks, {}..{} frames each, longest {:.2} ms",
            self.calls,
            self.min,
            self.max,
            self.longest_ns as f64 / 1e6
        )
    }
}

/// The configuration to open. With no rate asked for: 44.1 kHz with two channels when offered
/// (f32 preferred), else 44.1 kHz with any channel count, else the device's default (resampled).
/// With a rate asked for: that rate in the same order of preference, or an error when the device
/// does not offer it. The third value is the device's buffer range for that configuration.
fn choose(
    device: &cpal::Device,
    rate: Option<u32>,
) -> Result<(StreamConfig, SampleFormat, SupportedBufferSize), Error> {
    let want = rate.unwrap_or(RATE);
    let ranges: Vec<_> = device
        .supported_output_configs()
        .map_err(|e| err("the output configurations", e))?
        .filter(|r| r.min_sample_rate() <= want && want <= r.max_sample_rate())
        .collect();
    let score = |r: &cpal::SupportedStreamConfigRange| {
        (
            r.channels() == 2,
            r.sample_format() == SampleFormat::F32,
            r.channels() >= 2,
        )
    };
    if let Some(r) = ranges.iter().max_by_key(|r| score(r)) {
        let c = r.with_sample_rate(want);
        return Ok((c.config(), c.sample_format(), *r.buffer_size()));
    }
    if rate.is_some() {
        return Err(Error(format!("the device does not offer {want} Hz")));
    }
    let c = device
        .default_output_config()
        .map_err(|e| err("the default output configuration", e))?;
    Ok((c.config(), c.sample_format(), *c.buffer_size()))
}

/// Holds the [`Output`] inside the stream's callback and hands its [`Audio`] back when the
/// callback is dropped, so that a refused build can be retried. The hand-back is a one-place
/// wait-free queue, pushed to only in `drop`, never by the data callback.
struct Keeper {
    out: Option<Output>,
    back: rtrb::Producer<Audio>,
}

impl Drop for Keeper {
    fn drop(&mut self) {
        if let Some(out) = self.out.take() {
            let _ = self.back.push(out.audio);
        }
    }
}

fn build<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: StreamConfig,
    out: Output,
    back: rtrb::Producer<Audio>,
    health: Health,
    stats: CallbackStats,
) -> Result<cpal::Stream, Error> {
    let channels = out.channels;
    let rate = config.sample_rate;
    let mut keeper = Keeper {
        out: Some(out),
        back,
    };
    let mut book = CallbackBook::new(health.clone(), stats.clone(), rate);
    device
        .build_output_stream::<T, _, _>(
            config,
            move |data: &mut [T], info: &cpal::OutputCallbackInfo| {
                let t = Instant::now();
                let frames = (data.len() / channels) as u32;
                let ts = info.timestamp();
                book.enter(
                    t,
                    frames,
                    ts.playback.saturating_duration_since(ts.callback),
                );
                if let Some(out) = &mut keeper.out {
                    out.fill(data);
                }
                book.leave(frames, t.elapsed().as_nanos() as u64);
            },
            move |e: cpal::Error| {
                // the rule is in `Health`; the log is one line a second, with a count
                health.error(e.kind(), &e.to_string());
                if let Some(n) = health.note_error(health.now()) {
                    if n > 1 {
                        eprintln!("orb-player: audio output: {e} ({n} errors)");
                    } else {
                        eprintln!("orb-player: audio output: {e}");
                    }
                }
            },
            None,
        )
        .map_err(|e| err("the output stream", e))
}

/// Plays `audio` through the default output device (what `orb play` does).
pub fn start(audio: Audio) -> Result<Stream, Error> {
    open(audio, &Request::default()).map_err(|r| r.error)
}

/// A stream that could not be opened, and the engine, which is handed back when it can be
/// (`None` only when the refused stream's thread never gave it up).
pub struct Refused {
    pub error: Error,
    pub audio: Option<Audio>,
}

/// Opens a stream for `request` and runs `audio` in it. Only the default request falls back to
/// the device's own buffer when the device refuses the size asked for (task-176); any other
/// request that the device refuses is an error, and the engine comes back with it.
pub fn open(audio: Audio, request: &Request) -> Result<Stream, Box<Refused>> {
    let mut audio = Some(audio);
    let refuse = |error: Error, audio: &mut Option<Audio>| {
        Box::new(Refused {
            error,
            audio: audio.take(),
        })
    };
    let host = match devices::host(request.driver.as_deref()) {
        Ok(h) => h,
        Err(e) => return Err(refuse(e, &mut audio)),
    };
    let driver = host.id().name().to_string();
    let found = match &request.device {
        Some(d) => devices::find_device(&host, &d.id),
        None => devices::default_device(&host),
    };
    let Some(device) = found else {
        let e = match &request.device {
            Some(d) => Error(format!("the audio device {} is not available", d.name)),
            None => Error("no default output device".into()),
        };
        return Err(refuse(e, &mut audio));
    };
    let name = devices::device_name(&device);
    let device_id = devices::device_id(&device).unwrap_or_default();
    let (config, format, range) = match choose(&device, request.rate) {
        Ok(c) => c,
        Err(e) => return Err(refuse(e, &mut audio)),
    };
    // ASIO's buffer is set in the driver's own control panel: the driver's size is used
    let requested = if devices::is_asio(Some(&driver)) {
        None
    } else {
        buffer_request_for(request.buffer, &range)
    };
    let resampled = config.sample_rate != RATE;
    let health = Health::started();
    let stats = CallbackStats::new();
    // the buffer as requested, then (for the default request only) the device's default one
    let mut tries = Vec::new();
    if let Some(n) = requested {
        tries.push((Some(n), BufferSize::Fixed(n)));
    }
    if requested.is_none() || request.is_default() {
        tries.push((None, BufferSize::Default));
    }
    let mut last = Error("no output stream".into());
    for (req, buffer_size) in tries {
        let Some(a) = audio.take() else {
            break;
        };
        let (back, mut recover) = rtrb::RingBuffer::new(1);
        let resampler = match Output::resampler(config.channels as usize, config.sample_rate) {
            Ok(r) => r,
            Err(e) => return Err(refuse(e, &mut Some(a))),
        };
        let out = Output::with(a, config.channels as usize, resampler);
        let config = StreamConfig {
            buffer_size,
            ..config
        };
        let h = health.clone();
        let st = stats.clone();
        let built = match format {
            SampleFormat::F32 => build::<f32>(&device, config, out, back, h, st),
            SampleFormat::F64 => build::<f64>(&device, config, out, back, h, st),
            SampleFormat::I16 => build::<i16>(&device, config, out, back, h, st),
            SampleFormat::I32 => build::<i32>(&device, config, out, back, h, st),
            SampleFormat::U16 => build::<u16>(&device, config, out, back, h, st),
            SampleFormat::U8 => build::<u8>(&device, config, out, back, h, st),
            SampleFormat::I8 => build::<i8>(&device, config, out, back, h, st),
            f => Err(Error(format!("unsupported sample format {f}"))),
        };
        match built {
            Ok(stream) => {
                if let Err(e) = stream.play() {
                    drop(stream);
                    audio = recover.pop().ok();
                    return Err(refuse(err("starting the stream", e), &mut audio));
                }
                // the first callback tells the granted size; it comes within a few periods
                let t = Instant::now();
                while stats.snapshot().calls == 0 && t.elapsed().as_millis() < 300 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                let _hold = devices::DriverHold::new(&driver);
                return Ok(Stream {
                    stream,
                    recover,
                    driver,
                    device_id,
                    follows_default: request.device.is_none(),
                    device: name,
                    config,
                    format,
                    resampled,
                    requested: req,
                    health,
                    stats,
                    _hold,
                });
            }
            Err(e) => {
                // the refused callback has been dropped and handed the engine back
                audio = recover.pop().ok();
                last = e;
            }
        }
    }
    Err(refuse(last, &mut audio))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ERR: &str = "snd_pcm_avail_delay: I/O error";

    #[test]
    fn buffer_choice() {
        let r = |min, max| SupportedBufferSize::Range { min, max };
        // inside the range: 2048 as is
        assert_eq!(buffer_request(&r(64, 8192)), Some(2048));
        assert_eq!(buffer_request(&r(2048, 2048)), Some(2048));
        // outside: clamped to the nearer end
        assert_eq!(buffer_request(&r(4096, 16384)), Some(4096));
        assert_eq!(buffer_request(&r(16, 512)), Some(512));
        // unknown (or a nonsensical range): the default buffer
        assert_eq!(buffer_request(&SupportedBufferSize::Unknown), None);
        assert_eq!(buffer_request(&r(512, 16)), None);
    }

    #[test]
    fn a_gap_is_more_than_two_buffers_and_twenty_milliseconds() {
        // 2048 frames at 48 kHz is 42.7 ms: the limit is 105.3 ms
        assert!(!is_gap(42_000_000, 2048, 48_000));
        assert!(!is_gap(105_000_000, 2048, 48_000));
        assert!(is_gap(106_000_000, 2048, 48_000));
        // a small buffer is judged with the 20 ms floor
        assert!(!is_gap(31_000_000, 256, 44_100));
        assert!(is_gap(32_000_000, 256, 44_100));
        assert!(!is_gap(1, 0, 0));
    }

    #[test]
    fn underruns_count_from_xruns_and_gaps() {
        let h = Health::new(0);
        assert_eq!(h.underruns(), 0);
        h.error(ErrorKind::Xrun, "xrun");
        h.underrun();
        h.error(ErrorKind::BackendError, "other");
        assert_eq!(h.underruns(), 2);
        assert!(!h.device_gone());
        h.error(ErrorKind::DeviceNotAvailable, "gone");
        assert!(h.device_gone());
    }

    #[test]
    fn a_requested_size_is_clamped_into_the_range() {
        let r = |min, max| SupportedBufferSize::Range { min, max };
        assert_eq!(buffer_request_for(256, &r(64, 8192)), Some(256));
        assert_eq!(buffer_request_for(256, &r(480, 480)), Some(480));
        assert_eq!(buffer_request_for(8192, &r(64, 4096)), Some(4096));
        assert_eq!(buffer_request_for(256, &SupportedBufferSize::Unknown), None);
    }

    #[test]
    fn callback_stats() {
        let s = CallbackStats::new();
        assert_eq!(s.snapshot().summary(), "no audio callback ran");
        s.record(2048, 1_500_000);
        s.record(1024, 400_000);
        s.record(2048, 2_000_000);
        let n = s.snapshot();
        assert_eq!((n.calls, n.first, n.min, n.max), (3, 2048, 1024, 2048));
        assert_eq!(n.longest_ns, 2_000_000);
        assert_eq!(
            n.summary(),
            "3 callbacks, 1024..2048 frames each, longest 2.00 ms"
        );
    }

    #[test]
    fn transient_error_that_keeps_playing_does_not_fail() {
        let h = Health::new(0);
        for t in (0..5000).step_by(20) {
            h.data(t);
            // an error in every cycle, as WSLg's PulseAudio plugin gives
            h.error(ErrorKind::BackendError, ERR);
            assert_eq!(h.failure_at(t + 10), None, "t={t}");
        }
        // errors with no data callback yet, but not for long
        h.error(ErrorKind::BackendError, ERR);
        assert_eq!(h.failure_at(5000 + STALL_MS - 1 - 20), None);
    }

    #[test]
    fn lasting_stop_fails() {
        let h = Health::new(0);
        h.data(1000);
        h.error(ErrorKind::BackendError, ERR);
        assert_eq!(h.failure_at(1000 + STALL_MS - 1), None);
        let m = h.failure_at(1000 + STALL_MS).expect("stalled");
        assert!(m.contains(ERR) && m.contains("500 ms"), "{m}");
        // the data callback running again clears it
        h.data(2000);
        assert_eq!(h.failure_at(2001), None);
        // no error, no data callback (an idle device): not a failure
        assert_eq!(h.failure_at(60_000), None);
    }

    #[test]
    fn device_gone_fails_at_once() {
        for k in [
            ErrorKind::DeviceNotAvailable,
            ErrorKind::HostUnavailable,
            ErrorKind::StreamInvalidated,
        ] {
            let h = Health::new(0);
            h.data(100);
            h.error(k, "Device disconnected");
            let m = h.failure_at(100).expect("fails at once");
            assert!(m.contains("Device disconnected"), "{m}");
            // a data callback does not undo it
            h.data(101);
            assert!(h.failure_at(102).is_some());
        }
    }

    #[test]
    fn soft_kinds_fail_only_by_a_lasting_silence() {
        for k in [
            ErrorKind::Xrun,
            ErrorKind::RealtimeDenied,
            ErrorKind::DeviceChanged,
            ErrorKind::DeviceBusy,
        ] {
            // not on the error alone, and not when data follows
            let h = Health::new(0);
            h.data(100);
            h.error(k, "soft");
            assert_eq!(h.failure_at(100), None);
            h.data(150);
            assert_eq!(h.failure_at(10_000), None);
            // but a silence of STALL_MS after it fails
            h.error(k, "soft");
            assert_eq!(h.failure_at(150 + STALL_MS - 1), None);
            assert!(h.failure_at(150 + STALL_MS).is_some());
        }
    }

    #[test]
    fn log_is_one_line_a_second_with_a_count() {
        let h = Health::new(0);
        assert_eq!(h.note_error(0), Some(1));
        assert_eq!(h.note_error(20), None);
        assert_eq!(h.note_error(999), None);
        assert_eq!(h.note_error(1000), Some(3));
        assert_eq!(h.note_error(1500), None);
        assert_eq!(h.note_error(2100), Some(2));
    }

    #[test]
    fn the_tail_of_a_burst_is_flushed_by_the_poller() {
        let h = Health::new(0);
        assert_eq!(h.flush_log(5000), None);
        assert_eq!(h.note_error(0), Some(1));
        assert_eq!(h.note_error(10), None);
        assert_eq!(h.note_error(20), None);
        assert_eq!(h.flush_log(999), None);
        assert_eq!(h.flush_log(1000), Some(2));
        assert_eq!(h.flush_log(5000), None);
    }
}
