//! The plugin's diagnostic log (task-260, `docs/plugin.md` section 10): a file the plugin writes
//! itself, in release builds, because a host's plugin process keeps no log of the plugin's own
//! (nice-plug has no logger here, so `NICE_LOG` does nothing).
//!
//! - It is on when `ORB_PLUGIN_LOG` names a file (appended to) or is `stderr` (Bitwig copies its
//!   plugin host's stderr into its `engine.log`), or, with the variable unset, when the file
//!   `plugin.log` exists in orb-ui's log folder (`~/.local/state/OpenRB-338` on Linux,
//!   `~/Library/Logs/OpenRB-338` on macOS, `%LOCALAPPDATA%\OpenRB-338\logs` on Windows, or `logs`
//!   next to the bundle in portable mode): that switch works in a host that does not pass the
//!   environment on. Read once per process.
//! - It takes the `log` records of wgpu (wgpu-core's acquire and present errors; info and above,
//!   debug with `ORB_PLUGIN_LOG_LEVEL=debug`), egui-wgpu and egui-baseview, and every panic (wgpu's
//!   uncaptured errors panic).
//! - [`activated`] logs each activation's library and rate (task-244's load log, folded in).
//! - [`graphics_config`] picks the editor's wgpu adapter as wgpu does (among the backends of
//!   `WGPU_BACKEND` when it is set) and logs the adapters, the surface's capabilities and the
//!   format and present mode the editor takes.
//! - [`Probe`] logs the editor's frames and, on X11, how the X server answers the Present requests
//!   baseview paces its frames on.

use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once, OnceLock};
use std::time::{Duration, Instant};

use egui_wgpu::wgpu::{self, Backends, DeviceType, PowerPreference};
use egui_wgpu::{WgpuSetup, WgpuSetupCreateNew};
use nice_plug_egui::GraphicsConfig;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

/// The variable that turns the log on: a file, or `stderr`.
pub const LOG_VAR: &str = "ORB_PLUGIN_LOG";
/// The file in orb-ui's log folder whose existence turns the log on (it is appended to).
pub const SWITCH_FILE: &str = "plugin.log";
/// The most lines one process writes (a driver that logs each frame cannot fill the disk).
const MAX_LINES: usize = 20_000;

/// Where the log goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Stderr,
    File(PathBuf),
}

/// The log's target for the variable's value `var` and orb-ui's log folder `folder`: the
/// variable when set and not empty (`stderr`, or a file), else the switch file when it exists,
/// else none.
pub fn target(var: Option<&OsStr>, folder: Option<&Path>) -> Option<Target> {
    if let Some(v) = var.filter(|v| !v.is_empty()) {
        return Some(if v == "stderr" {
            Target::Stderr
        } else {
            Target::File(PathBuf::from(v))
        });
    }
    let f = folder?.join(SWITCH_FILE);
    f.is_file().then_some(Target::File(f))
}

enum Sink {
    Stderr,
    File(File),
}

struct Log {
    sink: Mutex<Sink>,
    start: Instant,
    lines: AtomicUsize,
}

static LOG: OnceLock<Option<Log>> = OnceLock::new();
static INSTALL: Once = Once::new();

fn log() -> Option<&'static Log> {
    LOG.get_or_init(|| {
        let var = std::env::var_os(LOG_VAR);
        let sink = match target(var.as_deref(), orb_ui::crash::log_dir().as_deref())? {
            Target::Stderr => Sink::Stderr,
            Target::File(p) => {
                Sink::File(OpenOptions::new().create(true).append(true).open(p).ok()?)
            }
        };
        Some(Log {
            sink: Mutex::new(sink),
            start: Instant::now(),
            lines: AtomicUsize::new(0),
        })
    })
    .as_ref()
}

/// Whether the log is on.
pub fn enabled() -> bool {
    log().is_some()
}

/// Turns the log on if it is switched on (once per process): the `log` records and the panics
/// go to it from then on, after a first line on the plugin and its environment.
pub fn init() {
    if enabled() {
        INSTALL.call_once(install);
    }
}

fn install() {
    if log::set_boxed_logger(Box::new(Logger)).is_ok() {
        log::set_max_level(log::LevelFilter::Debug);
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        line(format_args!("panic: {info}"));
        previous(info);
    }));
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| "(unset)".into());
    line(format_args!(
        "OpenRB-338 plugin {} on {} {}, in {:?}; XDG_SESSION_TYPE {}, DISPLAY {}, \
         WAYLAND_DISPLAY {}, WGPU_BACKEND {}, WGPU_POWER_PREF {}",
        orb_ui::about::VERSION,
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::env::current_exe().unwrap_or_default(),
        var("XDG_SESSION_TYPE"),
        var("DISPLAY"),
        var("WAYLAND_DISPLAY"),
        var("WGPU_BACKEND"),
        var("WGPU_POWER_PREF"),
    ));
}

/// Writes a line to the log, if it is on, with the milliseconds since it was opened and the
/// thread.
pub fn line(args: std::fmt::Arguments) {
    let Some(l) = log() else { return };
    let n = l.lines.fetch_add(1, Ordering::Relaxed);
    if n > MAX_LINES {
        return;
    }
    let ms = l.start.elapsed().as_secs_f64() * 1000.0;
    let text = if n == MAX_LINES {
        format!("{ms:10.1} the log stops: {MAX_LINES} lines\n")
    } else {
        let t = std::thread::current();
        match t.name() {
            Some(name) => format!("{ms:10.1} [{name}] {args}\n"),
            None => format!("{ms:10.1} [{:?}] {args}\n", t.id()),
        }
    };
    let mut g = l.sink.lock().unwrap_or_else(|e| e.into_inner());
    let _ = match &mut *g {
        Sink::Stderr => std::io::stderr().write_all(text.as_bytes()),
        Sink::File(f) => f.write_all(text.as_bytes()),
    };
}

/// The most detailed level logged for a record of the module path `target`: debug for
/// egui-baseview and the plugin; for wgpu and egui-wgpu debug only with `wgpu_debug`
/// ([`LEVEL_VAR`] `debug`: their debug records are thousands of lines per editor, the
/// swapchain's per resize and the shaders), else info; info for the rest.
pub fn level_for(target: &str, wgpu_debug: bool) -> log::LevelFilter {
    let root = target.split("::").next().unwrap_or(target);
    match root {
        "egui_baseview" | "orb_plugin" => log::LevelFilter::Debug,
        "wgpu" | "wgpu_core" | "wgpu_hal" | "egui_wgpu" if wgpu_debug => log::LevelFilter::Debug,
        _ => log::LevelFilter::Info,
    }
}

/// The variable whose value `debug` adds wgpu's and egui-wgpu's debug records to the log.
pub const LEVEL_VAR: &str = "ORB_PLUGIN_LOG_LEVEL";

/// Whether [`LEVEL_VAR`] asks for wgpu's debug records (read once).
fn wgpu_debug() -> bool {
    static DEBUG: OnceLock<bool> = OnceLock::new();
    *DEBUG.get_or_init(|| {
        std::env::var(LEVEL_VAR).is_ok_and(|v| v.trim().eq_ignore_ascii_case("debug"))
    })
}

/// The path of the loaded library that holds this code (the bundle's binary), from the dynamic
/// loader; `None` where it is not known (Windows: not implemented). From task-244's load log.
pub fn image_path() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::ffi::{CStr, c_char, c_int, c_void};
        use std::os::unix::ffi::OsStrExt;
        #[repr(C)]
        struct DlInfo {
            fname: *const c_char,
            fbase: *mut c_void,
            sname: *const c_char,
            saddr: *mut c_void,
        }
        unsafe extern "C" {
            fn dladdr(addr: *const c_void, info: *mut DlInfo) -> c_int;
        }
        let mut info = DlInfo {
            fname: std::ptr::null(),
            fbase: std::ptr::null_mut(),
            sname: std::ptr::null(),
            saddr: std::ptr::null_mut(),
        };
        let addr = image_path as fn() -> Option<PathBuf> as *const c_void;
        if unsafe { dladdr(addr, &mut info) } == 0 || info.fname.is_null() {
            return None;
        }
        let name = unsafe { CStr::from_ptr(info.fname) };
        Some(PathBuf::from(OsStr::from_bytes(name.to_bytes())))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// What precedes the library's path in an activation's line ([`activated`]).
pub const LOADED_FROM: &str = "activate: loaded from ";

/// Logs an activation (main thread, `Plugin::activate`): the library that runs ([`image_path`],
/// `?` when unknown) and the host's rate, as `activate: loaded from <path> at <rate> Hz`, so
/// that a test driving a real host (`tests/reaper.rs`, task-244) can tell which copy of the
/// plugin the host loaded ([`parse_loaded_from`]). With the log off it only looks up the switch
/// (once per process).
pub fn activated(rate: u32) {
    if !enabled() {
        return;
    }
    init();
    let image = image_path().map_or_else(|| "?".to_string(), |p| p.display().to_string());
    line(format_args!("{LOADED_FROM}{image} at {rate} Hz"));
}

/// The library path and the rate of a log line written by [`activated`], or `None` for any
/// other line.
pub fn parse_loaded_from(line: &str) -> Option<(&str, u32)> {
    let (_, rest) = line.split_once(LOADED_FROM)?;
    let (path, rate) = rest.rsplit_once(" at ")?;
    Some((path, rate.strip_suffix(" Hz")?.parse().ok()?))
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= level_for(m.target(), wgpu_debug())
    }

    fn log(&self, r: &log::Record) {
        if self.enabled(r.metadata()) {
            line(format_args!("{} {}: {}", r.level(), r.target(), r.args()));
        }
    }

    fn flush(&self) {}
}

/// egui-baseview's graphics with the editor's adapter choice ([`select_adapter`]): the
/// adapters of the backends of `WGPU_BACKEND` (wgpu's names, comma-separated: `vulkan`, `gl`,
/// `metal`, `dx12`) when it is set, else of all backends, as before task-260. egui-baseview
/// makes the wgpu instance itself, with all backends; only the adapter is chosen here.
pub fn graphics_config() -> GraphicsConfig {
    let mut config = GraphicsConfig::default();
    let mut setup = WgpuSetupCreateNew::without_display_handle();
    let env = Backends::from_env();
    let backends = env.unwrap_or(Backends::all());
    setup.instance_descriptor.backends = backends;
    let power = setup.power_preference;
    let surface = config.wgpu_options.surface;
    setup.native_adapter_selector = Some(Arc::new(move |adapters, s| {
        line(format_args!(
            "wgpu: adapters of {backends:?} (WGPU_BACKEND {env:?}), power preference {power:?}"
        ));
        let r = select_adapter(adapters, s, power);
        match &r {
            Ok(a) => {
                let info = a.get_info();
                line(format_args!(
                    "wgpu: the editor takes {}",
                    egui_wgpu::adapter_info_summary(&info)
                ));
                if let Some(s) = s {
                    let caps = s.get_capabilities(a);
                    line(format_args!(
                        "wgpu: the surface's formats {:?}, present modes {:?}, alpha modes \
                         {:?}; egui takes {:?}, present mode {:?}, frame latency {:?}",
                        caps.formats,
                        caps.present_modes,
                        caps.alpha_modes,
                        egui_wgpu::preferred_framebuffer_format(&caps.formats).ok(),
                        surface.present_mode,
                        surface.desired_maximum_frame_latency,
                    ));
                }
            }
            Err(e) => line(format_args!("wgpu: {e}")),
        }
        r
    }));
    config.wgpu_options.wgpu_setup = WgpuSetup::CreateNew(setup);
    config
}

/// The adapter wgpu-core 30's `request_adapter` gives for the window's surface `surface` among
/// `adapters` (in wgpu's backend order): those that can present to it, the first of the best
/// device type for `power` (none: the first). Each adapter is logged.
fn select_adapter(
    adapters: &[wgpu::Adapter],
    surface: Option<&wgpu::Surface<'_>>,
    power: PowerPreference,
) -> Result<wgpu::Adapter, String> {
    let mut types = Vec::with_capacity(adapters.len());
    for (i, a) in adapters.iter().enumerate() {
        let info = a.get_info();
        let ok = surface.is_none_or(|s| a.is_surface_supported(s));
        line(format_args!(
            "wgpu: adapter {i}: {}; presents to the window: {ok}",
            egui_wgpu::adapter_info_summary(&info)
        ));
        types.push(ok.then_some(info.device_type));
    }
    pick(&types, power)
        .map(|i| adapters[i].clone())
        .ok_or_else(|| "no wgpu adapter can draw to the editor's window".into())
}

/// The index wgpu-core 30's `request_adapter` picks among adapters of the device types `types`
/// (`None`: cannot present to the surface): a stable sort by its order for `power`.
pub fn pick(types: &[Option<DeviceType>], power: PowerPreference) -> Option<usize> {
    let mut v: Vec<(usize, DeviceType)> = types
        .iter()
        .enumerate()
        .filter_map(|(i, t)| t.map(|t| (i, t)))
        .collect();
    match power {
        PowerPreference::LowPower => v.sort_by_key(|&(_, t)| order(t, true)),
        PowerPreference::HighPerformance => v.sort_by_key(|&(_, t)| order(t, false)),
        _ => {}
    }
    v.first().map(|&(i, _)| i)
}

/// wgpu-core 30's `get_order` (`instance.rs`).
fn order(t: DeviceType, prefer_integrated: bool) -> u8 {
    match t {
        DeviceType::DiscreteGpu if prefer_integrated => 2,
        DeviceType::IntegratedGpu if prefer_integrated => 1,
        DeviceType::DiscreteGpu => 1,
        DeviceType::IntegratedGpu => 2,
        DeviceType::Other => 3,
        DeviceType::VirtualGpu => 4,
        DeviceType::Cpu => 5,
    }
}

/// The times (seconds after the window was built) the probe logs the frame count.
const MARKS: [u64; 6] = [1, 2, 4, 8, 16, 32];

/// One editor window's diagnostics, while the log is on: its frames ([`Probe::frame`]) and a
/// thread that logs their count at [`MARKS`] and probes the X server's Present extension.
pub struct Probe {
    frames: Arc<AtomicU64>,
    closed: Arc<AtomicBool>,
    last: Option<Instant>,
}

impl Probe {
    /// The probe of the window `window`, or `None` with the log off.
    pub fn start(window: &impl HasWindowHandle) -> Option<Probe> {
        if !enabled() {
            return None;
        }
        let handle = window.window_handle().map(|h| h.as_raw());
        // Xlib's window id is a c_ulong: u64 on Linux and macOS but u32 on Windows, where the
        // conversion is to the same type (clippy's useless_conversion failed CI there)
        #[allow(clippy::useless_conversion)]
        let x11 = match handle {
            Ok(RawWindowHandle::Xlib(h)) => u32::try_from(h.window).ok(),
            Ok(RawWindowHandle::Xcb(h)) => Some(h.window.get()),
            _ => None,
        };
        line(format_args!("editor: window built, {handle:?}"));
        let frames = Arc::new(AtomicU64::new(0));
        let closed = Arc::new(AtomicBool::new(false));
        let (f, c) = (frames.clone(), closed.clone());
        let spawned = std::thread::Builder::new()
            .name("orb editor probe".into())
            .spawn(move || run(&f, &c, x11));
        if let Err(e) = spawned {
            line(format_args!("editor: no probe thread: {e}"));
        }
        Some(Probe {
            frames,
            closed,
            last: None,
        })
    }

    /// One frame of the editor: its size in pixels and points per pixel. The first five and each
    /// one after a pause over a second are logged.
    pub fn frame(&mut self, size: egui::Vec2, ppp: f32) {
        let n = self.frames.fetch_add(1, Ordering::Relaxed) + 1;
        let now = Instant::now();
        let gap = self.last.map(|l| now - l);
        self.last = Some(now);
        if n <= 5 || gap.is_some_and(|g| g > Duration::from_secs(1)) {
            line(format_args!(
                "editor: frame {n}, {:.0}x{:.0} px at {ppp} px/pt, {:.1} ms after the last",
                size.x * ppp,
                size.y * ppp,
                gap.map_or(0.0, |g| g.as_secs_f64() * 1000.0)
            ));
        }
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        self.closed.store(true, Ordering::Relaxed);
        line(format_args!(
            "editor: closed after {} frames",
            self.frames.load(Ordering::Relaxed)
        ));
    }
}

/// The probe's thread: the frame count at each mark until the window closes, and the Present
/// probe of the X11 window `x11` after the first.
fn run(frames: &AtomicU64, closed: &AtomicBool, x11: Option<u32>) {
    let start = Instant::now();
    for t in MARKS {
        let at = start + Duration::from_secs(t);
        while Instant::now() < at {
            if closed.load(Ordering::Relaxed) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if closed.load(Ordering::Relaxed) {
            return;
        }
        let n = frames.load(Ordering::Relaxed);
        line(format_args!(
            "editor: {t} s after the window was built, {n} frames"
        ));
        if t == 1
            && let Some(w) = x11
        {
            present_probe(w);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn present_probe(window: u32) {
    if let Err(e) = x11::present_probe(window) {
        line(format_args!("present probe: failed: {e}"));
    }
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn present_probe(_window: u32) {}

/// The Present probe: baseview 0.4.2 on X11 draws a frame only when the X server's Present
/// extension answers its `PresentNotifyMSC` with a `CompleteNotify` whose MSC is past the last
/// one (`platform/x11/present.rs`); its fallback timer (15 ms) runs only without the
/// extension. The probe asks for the same notifications on baseview's window from a connection
/// of its own and logs the answers: the MSC, the UST and the delay.
#[cfg(all(unix, not(target_os = "macos")))]
mod x11 {
    use std::error::Error;
    use std::time::{Duration, Instant};

    use x11rb::connection::{Connection, RequestConnection as _};
    use x11rb::protocol::Event;
    use x11rb::protocol::present::{self, CompleteNotifyEvent, ConnectionExt as _};
    use x11rb::protocol::xproto::ConnectionExt as _;
    use x11rb::rust_connection::RustConnection;

    use super::line;

    /// The probe's serials ("ORB" and the request), apart from baseview's (0, 1, 2, ...).
    const SERIAL: u32 = 0x4f52_4200;
    /// How many notifications the probe asks for.
    const COUNT: u32 = 6;
    /// How long it waits for each.
    const WAIT: Duration = Duration::from_secs(2);

    pub fn present_probe(window: u32) -> Result<(), Box<dyn Error>> {
        let (conn, _) = RustConnection::connect(None)?;
        if conn
            .extension_information(present::X11_EXTENSION_NAME)?
            .is_none()
        {
            line(format_args!(
                "present probe: the X server has no Present extension (baseview draws on its \
                 15 ms timer)"
            ));
            return Ok(());
        }
        let v = conn.present_query_version(1, 2)?.reply()?;
        let g = conn.get_geometry(window)?.reply()?;
        let at = conn.translate_coordinates(window, g.root, 0, 0)?.reply()?;
        line(format_args!(
            "present probe: Present {}.{}; window 0x{window:x}, {}x{} at ({}, {}) on the root, \
             depth {}",
            v.major_version, v.minor_version, g.width, g.height, at.dst_x, at.dst_y, g.depth
        ));
        let eid = conn.generate_id()?;
        conn.present_select_input(eid, window, present::EventMask::COMPLETE_NOTIFY)?
            .check()?;
        let mut last: Option<u64> = None;
        for i in 0..COUNT {
            // baseview asks for the last MSC + 2
            let target = last.map_or(0, |m| m + 2);
            let t0 = Instant::now();
            conn.present_notify_msc(window, SERIAL + i, target, 0, 0)?;
            conn.flush()?;
            match wait(&conn, SERIAL + i)? {
                Some(e) => {
                    line(format_args!(
                        "present probe: NotifyMSC {i}, target MSC {target}: complete, MSC {}, \
                         UST {} us, kind {:?}, mode {:?}, after {:.1} ms{}",
                        e.msc,
                        e.ust,
                        e.kind,
                        e.mode,
                        t0.elapsed().as_secs_f64() * 1000.0,
                        if last.is_some_and(|l| e.msc <= l) {
                            " (the MSC did not advance: baseview draws no frame on it)"
                        } else {
                            ""
                        }
                    ));
                    last = Some(e.msc);
                }
                None => {
                    line(format_args!(
                        "present probe: NotifyMSC {i}, target MSC {target}: no CompleteNotify in \
                         {} s (baseview waits for it before its next frame)",
                        WAIT.as_secs()
                    ));
                    break;
                }
            }
        }
        conn.present_select_input(eid, window, present::EventMask::NO_EVENT)?
            .check()?;
        line(format_args!("present probe: done"));
        Ok(())
    }

    /// The `CompleteNotify` of `serial`, or `None` after [`WAIT`].
    fn wait(
        conn: &RustConnection,
        serial: u32,
    ) -> Result<Option<CompleteNotifyEvent>, Box<dyn Error>> {
        let end = Instant::now() + WAIT;
        loop {
            while let Some(ev) = conn.poll_for_event()? {
                if let Event::PresentCompleteNotify(e) = ev
                    && e.serial == serial
                {
                    return Ok(Some(e));
                }
            }
            if Instant::now() >= end {
                return Ok(None);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_variable_wins_and_the_switch_file_turns_the_log_on() {
        let dir = tempfile::tempdir().unwrap();
        let d = Some(dir.path());
        assert_eq!(target(None, d), None);
        assert_eq!(target(Some(OsStr::new("")), d), None);
        assert_eq!(target(None, None), None);
        assert_eq!(target(Some(OsStr::new("stderr")), d), Some(Target::Stderr));
        assert_eq!(
            target(Some(OsStr::new("/x/y.log")), d),
            Some(Target::File(PathBuf::from("/x/y.log")))
        );
        let f = dir.path().join(SWITCH_FILE);
        std::fs::write(&f, b"").unwrap();
        assert_eq!(target(None, d), Some(Target::File(f.clone())));
        assert_eq!(target(Some(OsStr::new("")), d), Some(Target::File(f)));
        assert_eq!(target(Some(OsStr::new("stderr")), d), Some(Target::Stderr));
    }

    #[test]
    fn wgpu_logs_debug_only_when_asked_the_editor_always() {
        use log::LevelFilter::{Debug, Info};
        for t in [
            "wgpu_core::present",
            "wgpu_hal::gl::egl",
            "wgpu",
            "egui_wgpu",
        ] {
            assert_eq!(level_for(t, false), Info, "{t}");
            assert_eq!(level_for(t, true), Debug, "{t}");
        }
        for d in [false, true] {
            assert_eq!(level_for("egui_baseview::window", d), Debug);
            assert_eq!(level_for("orb_plugin::editor", d), Debug);
            assert_eq!(level_for("naga::front", d), Info);
            assert_eq!(level_for("wgpu_core_x", d), Info);
        }
    }

    #[test]
    fn the_activation_line_gives_the_library_and_the_rate() {
        let l = format!("    12.5 [main] {LOADED_FROM}/a b/at x/OpenRB-338 at 48000 Hz");
        assert_eq!(parse_loaded_from(&l), Some(("/a b/at x/OpenRB-338", 48000)));
        assert_eq!(parse_loaded_from("    1.0 [main] editor: spawn"), None);
        assert_eq!(
            parse_loaded_from(&format!("{LOADED_FROM}/x at fast Hz")),
            None
        );
    }

    #[test]
    fn the_image_is_this_library() {
        if cfg!(unix) {
            // in a unit test the code is in the test binary itself
            let p = image_path().expect("dladdr finds the image");
            let exe = std::env::current_exe().unwrap();
            assert_eq!(
                p.canonicalize().unwrap(),
                exe.canonicalize().unwrap(),
                "{p:?}"
            );
        }
    }

    #[test]
    fn the_adapter_is_wgpus_pick() {
        use DeviceType::*;
        let hp = PowerPreference::HighPerformance;
        let lp = PowerPreference::LowPower;
        // the software Vulkan and GL of a VM: the first, Vulkan (wgpu's backend order)
        assert_eq!(pick(&[Some(Cpu), Some(Cpu)], hp), Some(0));
        // one that cannot present is skipped
        assert_eq!(pick(&[None, Some(Cpu)], hp), Some(1));
        assert_eq!(pick(&[None, None], hp), None);
        assert_eq!(pick(&[], hp), None);
        // the device type before the backend order
        assert_eq!(
            pick(&[Some(Cpu), Some(Other), Some(DiscreteGpu)], hp),
            Some(2)
        );
        let gpus = [Some(Other), Some(DiscreteGpu), Some(IntegratedGpu)];
        assert_eq!(pick(&gpus, hp), Some(1));
        assert_eq!(pick(&gpus, lp), Some(2));
        assert_eq!(pick(&gpus, PowerPreference::None), Some(0));
        assert_eq!(pick(&[Some(Cpu), Some(VirtualGpu)], hp), Some(1));
    }
}
