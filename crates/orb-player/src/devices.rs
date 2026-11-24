//! The drivers, devices, rates and buffer sizes the Audio Output window offers (task-191), and
//! the request that names a choice of them.
//!
//! A *driver* is a cpal host: CoreAudio on macOS, WASAPI on Windows, ALSA on Linux (and ASIO
//! when task-193 builds it in). A device is named by cpal's [`cpal::DeviceId`] in its text form
//! (`CoreAudio:...`), which survives a restart; its name is for display only. What the window
//! lists for a device comes from pure functions of the device's supported configurations
//! ([`offered_rates`], [`buffer_sizes`]), so the rules are tested without a device.

pub use cpal::SupportedBufferSize;
use cpal::SupportedStreamConfigRange;
use cpal::traits::{DeviceTrait, HostTrait};

use crate::output::{BUFFER_FRAMES, Error};

/// The largest buffer the window offers, in frames; the smallest is the device's own minimum
/// ([`UNKNOWN_MIN_BUFFER`] for a device that states none).
pub const MAX_BUFFER: u32 = 8192;
/// The smallest size offered for a device that states no range.
pub const UNKNOWN_MIN_BUFFER: u32 = 64;

/// The rates the window offers when the device's configurations allow them. A device that
/// allows a continuous range gets these; a rate outside them is not offered.
pub const STANDARD_RATES: [u32; 11] = [
    8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000, 176400, 192000,
];

/// What to open: the driver, the device, the rate and the buffer. The default is the platform's
/// first driver, the system default device, the device's default rate and 2048 frames, which is
/// what `orb play` opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// The driver's name ([`drivers`]); `None`: the platform's default host.
    pub driver: Option<String>,
    /// A named device; `None`: the system default, which the stream follows.
    pub device: Option<DeviceRef>,
    /// A rate the device lists; `None`: the device's default (44.1 kHz when it offers it, else
    /// its own default rate, with the engine's output resampled).
    pub rate: Option<u32>,
    /// The buffer asked for, in frames.
    pub buffer: u32,
}

impl Default for Request {
    fn default() -> Self {
        Request {
            driver: None,
            device: None,
            rate: None,
            buffer: BUFFER_FRAMES,
        }
    }
}

/// A named device: its id (what is looked up) and its name (what the user is told).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceRef {
    pub id: String,
    pub name: String,
}

impl Request {
    /// Whether this is the request `orb play` makes (everything default). Only that one falls
    /// back to the device's default buffer when the device refuses the size
    /// ([`crate::output::open`]); any other request that fails is reported.
    pub fn is_default(&self) -> bool {
        *self == Request::default()
    }

    /// The same request on the system default device, when a named device is missing or has
    /// gone.
    pub fn on_system_default(&self) -> Request {
        Request {
            device: None,
            ..self.clone()
        }
    }
}

/// The drivers cpal offers on this build, the platform's default first.
pub fn drivers() -> Vec<String> {
    let first = cpal::default_host().id();
    let mut v: Vec<cpal::HostId> = cpal::available_hosts();
    // the default first, the others in cpal's order
    v.sort_by_key(|h| *h != first);
    v.iter().map(|h| h.name().to_string()).collect()
}

/// The host named `driver` (`None`: the default one).
pub fn host(driver: Option<&str>) -> Result<cpal::Host, Error> {
    let Some(name) = driver else {
        return Ok(cpal::default_host());
    };
    let id = cpal::available_hosts()
        .into_iter()
        .find(|h| h.name() == name)
        .ok_or_else(|| Error(format!("the audio driver {name} is not available")))?;
    cpal::host_from_id(id).map_err(|e| Error(format!("the audio driver {name}: {e}")))
}

/// A device's text id.
pub fn device_id(device: &cpal::Device) -> Option<String> {
    device.id().ok().map(|i| i.to_string())
}

/// A device's name for the user.
pub fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|d| d.to_string())
        .unwrap_or_else(|_| "?".into())
}

/// The device `id` names on `host`, if it is there.
pub fn find_device(host: &cpal::Host, id: &str) -> Option<cpal::Device> {
    let id: cpal::DeviceId = id.parse().ok()?;
    host.device_by_id(&id)
}

/// The system default output device's id on a driver, `None` when there is none.
pub fn default_device_id(driver: Option<&str>) -> Option<String> {
    let host = host(driver).ok()?;
    host.default_output_device().and_then(|d| device_id(&d))
}

/// One output device of a driver.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

/// The output devices of a driver, in the driver's order.
pub fn list_devices(driver: Option<&str>) -> Result<Vec<DeviceInfo>, Error> {
    let host = host(driver)?;
    let devices = host
        .output_devices()
        .map_err(|e| Error(format!("the output devices: {e}")))?;
    Ok(devices
        .filter_map(|d| {
            Some(DeviceInfo {
                id: device_id(&d)?,
                name: device_name(&d),
            })
        })
        .collect())
}

/// What a device allows: the rates and the buffer sizes the window offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Caps {
    /// The rates the device lists, ascending.
    pub rates: Vec<u32>,
    /// The buffer sizes, ascending ([`buffer_sizes`]).
    pub buffers: Vec<u32>,
    /// The device's own range, which a request is clamped into.
    pub range: SupportedBufferSize,
}

impl Caps {
    /// The size a request for `want` frames really gets: `want` clamped into the device's
    /// range (as `output::open` does), `want` itself when the device states none.
    pub fn requested(&self, want: u32) -> u32 {
        crate::output::buffer_request_for(want, &self.range).unwrap_or(want)
    }
}

/// The capabilities of a device (`None`: the driver's default output device).
pub fn caps(driver: Option<&str>, device: Option<&str>) -> Result<Caps, Error> {
    let host = host(driver)?;
    let d = match device {
        Some(id) => find_device(&host, id)
            .ok_or_else(|| Error("the audio device is not available".into()))?,
        None => host
            .default_output_device()
            .ok_or_else(|| Error("no default output device".into()))?,
    };
    let configs: Vec<SupportedStreamConfigRange> = d
        .supported_output_configs()
        .map_err(|e| Error(format!("the output configurations: {e}")))?
        .collect();
    let spans: Vec<(u32, u32)> = configs
        .iter()
        .map(|c| (c.min_sample_rate(), c.max_sample_rate()))
        .collect();
    let range = merged_buffer_range(configs.iter().map(|c| *c.buffer_size()));
    Ok(Caps {
        rates: offered_rates(&spans),
        buffers: buffer_sizes(&range),
        range,
    })
}

/// The rates of [`STANDARD_RATES`] that fall into one of the `(min, max)` spans, ascending.
pub fn offered_rates(spans: &[(u32, u32)]) -> Vec<u32> {
    STANDARD_RATES
        .into_iter()
        .filter(|r| spans.iter().any(|&(lo, hi)| lo <= *r && *r <= hi))
        .collect()
}

/// The widest buffer range of several configurations (`Unknown` when none states one).
pub fn merged_buffer_range(
    ranges: impl Iterator<Item = SupportedBufferSize>,
) -> SupportedBufferSize {
    let mut out = SupportedBufferSize::Unknown;
    for r in ranges {
        if let SupportedBufferSize::Range { min, max } = r
            && min <= max
        {
            out = match out {
                SupportedBufferSize::Range { min: a, max: b } => SupportedBufferSize::Range {
                    min: a.min(min),
                    max: b.max(max),
                },
                SupportedBufferSize::Unknown => SupportedBufferSize::Range { min, max },
            };
        }
    }
    out
}

/// The buffer sizes to offer for a device's range: from the device's minimum to
/// [`MAX_BUFFER`], clamped into its range. The minimum itself comes first (it is a power of
/// two or it is not), then the powers of two above it up to the cap. A device with one fixed
/// size gets just it, and one whose minimum is above [`MAX_BUFFER`] gets its minimum. A
/// device that states no range gets the powers of two from [`UNKNOWN_MIN_BUFFER`].
pub fn buffer_sizes(range: &SupportedBufferSize) -> Vec<u32> {
    let (min, max) = match *range {
        SupportedBufferSize::Range { min, max } if min <= max && min > 0 => (min, max),
        _ => (UNKNOWN_MIN_BUFFER, MAX_BUFFER),
    };
    let cap = max.min(MAX_BUFFER);
    if min >= cap {
        return vec![min];
    }
    let mut v = vec![min];
    let mut n = min.next_power_of_two();
    while n <= cap {
        if n > min {
            v.push(n);
        }
        n *= 2;
    }
    v
}

/// The time `frames` frames last at `rate` Hz, in milliseconds (2048 frames at 44.1 kHz:
/// 46.4 ms). The engine's own 1024-frame step is not part of it.
pub fn latency_ms(frames: u32, rate: u32) -> f64 {
    frames as f64 * 1000.0 / rate.max(1) as f64
}

/// The sizes of a device that states no range: all the powers of two of the window.
pub fn default_buffer_sizes() -> Vec<u32> {
    buffer_sizes(&SupportedBufferSize::Unknown)
}

/// The size to select after the device (and so the list) changed: `want` when offered, else
/// the offered size nearest to it.
pub fn nearest_buffer(offered: &[u32], want: u32) -> u32 {
    offered
        .iter()
        .copied()
        .min_by_key(|n| n.abs_diff(want))
        .unwrap_or(want)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(min: u32, max: u32) -> SupportedBufferSize {
        SupportedBufferSize::Range { min, max }
    }

    #[test]
    fn buffer_sizes_run_from_the_devices_minimum_to_8192() {
        assert_eq!(
            buffer_sizes(&r(16, 4096)),
            vec![16, 32, 64, 128, 256, 512, 1024, 2048, 4096]
        );
        assert_eq!(
            buffer_sizes(&r(256, 8192)),
            vec![256, 512, 1024, 2048, 4096, 8192]
        );
        // the cap is 8192 whatever the device allows
        assert_eq!(buffer_sizes(&r(2048, 1_000_000)), vec![2048, 4096, 8192]);
        // a minimum that is not a power of two is offered itself, then the powers above it
        assert_eq!(buffer_sizes(&r(14, 100)), vec![14, 16, 32, 64]);
        assert_eq!(buffer_sizes(&r(500, 2100)), vec![500, 512, 1024, 2048]);
        // unknown: the powers of two from 64
        assert_eq!(
            buffer_sizes(&SupportedBufferSize::Unknown),
            vec![64, 128, 256, 512, 1024, 2048, 4096, 8192]
        );
        assert_eq!(buffer_sizes(&r(512, 16)).len(), 8, "nonsense is unknown");
    }

    #[test]
    fn a_fixed_or_high_range_gets_the_devices_own_size() {
        assert_eq!(buffer_sizes(&r(480, 480)), vec![480]);
        assert_eq!(buffer_sizes(&r(2048, 2048)), vec![2048]);
        assert_eq!(buffer_sizes(&r(9000, 12000)), vec![9000]);
        assert_eq!(buffer_sizes(&r(600, 700)), vec![600]);
    }

    #[test]
    fn a_request_is_clamped_into_the_devices_range() {
        let caps = |range| Caps {
            rates: vec![],
            buffers: buffer_sizes(&range),
            range,
        };
        let high = caps(r(4096, 8192));
        assert_eq!(high.requested(2048), 4096);
        assert_eq!(high.requested(8192), 8192);
        assert_eq!(caps(r(16, 512)).requested(2048), 512);
        assert_eq!(caps(SupportedBufferSize::Unknown).requested(2048), 2048);
    }

    #[test]
    fn latency_is_frames_over_the_rate() {
        assert_eq!(format!("{:.1}", latency_ms(2048, 44100)), "46.4");
        assert_eq!(format!("{:.1}", latency_ms(256, 48000)), "5.3");
        assert_eq!(format!("{:.1}", latency_ms(8192, 44100)), "185.8");
    }

    #[test]
    fn the_selected_size_moves_to_the_nearest_offered() {
        let v = [512, 1024, 2048];
        assert_eq!(nearest_buffer(&v, 2048), 2048);
        assert_eq!(nearest_buffer(&v, 8192), 2048);
        assert_eq!(nearest_buffer(&v, 256), 512);
        assert_eq!(nearest_buffer(&[16, 32, 64], 2048), 64);
        assert_eq!(nearest_buffer(&[480], 2048), 480);
        assert_eq!(nearest_buffer(&[], 2048), 2048);
    }

    #[test]
    fn rates_are_the_standard_ones_a_span_holds() {
        assert_eq!(offered_rates(&[(44100, 48000)]), vec![44100, 48000]);
        assert_eq!(
            offered_rates(&[(8000, 48000), (88200, 96000)]),
            vec![8000, 11025, 16000, 22050, 32000, 44100, 48000, 88200, 96000]
        );
        assert_eq!(offered_rates(&[(48000, 48000)]), vec![48000]);
        assert_eq!(offered_rates(&[(50000, 60000)]), Vec::<u32>::new());
        assert_eq!(offered_rates(&[]), Vec::<u32>::new());
    }

    #[test]
    fn buffer_ranges_of_several_configs_merge() {
        let m = merged_buffer_range([r(64, 1024), r(256, 8192)].into_iter());
        assert_eq!(m, r(64, 8192));
        let m = merged_buffer_range([SupportedBufferSize::Unknown, r(128, 4096)].into_iter());
        assert_eq!(m, r(128, 4096));
        assert_eq!(
            merged_buffer_range([SupportedBufferSize::Unknown].into_iter()),
            SupportedBufferSize::Unknown
        );
    }

    #[test]
    fn the_default_request_is_what_orb_play_opens() {
        let d = Request::default();
        assert!(d.is_default());
        assert_eq!(d.buffer, 2048);
        let named = Request {
            device: Some(DeviceRef {
                id: "CoreAudio:x".into(),
                name: "X".into(),
            }),
            rate: Some(48000),
            buffer: 512,
            driver: Some("CoreAudio".into()),
        };
        assert!(!named.is_default());
        // a missing device keeps the driver, the rate and the buffer
        let back = named.on_system_default();
        assert_eq!(back.device, None);
        assert_eq!(back.rate, Some(48000));
        assert_eq!(back.buffer, 512);
        assert_eq!(back.driver.as_deref(), Some("CoreAudio"));
    }

    #[test]
    fn this_build_offers_a_driver_and_the_default_comes_first() {
        let v = drivers();
        assert!(!v.is_empty(), "no audio driver on this build");
        assert_eq!(v[0], cpal::default_host().id().name());
    }
}
