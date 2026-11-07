//! The run log (task-214): every start of OpenRB-338 writes `run-YYYYMMDD-HHMMSS.log` beside the
//! crash logs ([`crate::crash::log_dir`]: `ORB_UI_LOG_DIR`, else `%LOCALAPPDATA%\OpenRB-338\logs`
//! on Windows), so that a start that fails leaves a trace that can be read afterwards as plain
//! text. Started from Explorer or the Start menu the Windows release build has no visible
//! stderr, and before this an error of `eframe::run_native` (no graphics adapter, no window), a
//! native crash in a driver or a hang during the start left nothing at all.
//!
//! What the log holds, each line flushed as it is written (a hang or a native crash keeps the
//! lines before it):
//! - a header ([`header`]): time, version, the system with the Windows build, the arguments, the
//!   executable, whether the system reports a screen reader, the `ORB_*` / `WGPU_*` / `RUST_*`
//!   variables that are set;
//! - a line per stage of the start ([`stage`]), with the milliseconds since the start: the song,
//!   the audio output, the window, the graphics adapter, the first frame drawn;
//! - the messages of the libraries (wgpu, winit, eframe, cpal, AccessKit) through the `log` crate:
//!   warnings and errors, or more with `ORB_LOG=info`, `debug` or `trace` ([`level`]); at most
//!   [`MAX_LIBRARY_LINES`] of them;
//! - a line from the watchdog when no frame has been drawn [`WATCHDOG`] after the start, naming the
//!   last stage reached (a hang);
//! - on Windows, a line for a native crash (an unhandled structured exception: code, address and
//!   the module of the address), written by an unhandled-exception filter that then leaves the
//!   crash to Windows Error Reporting (its LocalDumps still make a dump);
//! - the end ([`end`]): the window closed, a start error, a panic (naming its crash log).
//!
//! A run log without an end line ended in a way nothing could write: killed, a fail-fast abort
//! (`0xC0000409`, which no exception filter sees: Rust's `abort` uses it), or a hang.
//!
//! The newest [`KEEP`] run logs are kept, rotated apart from the crash logs. Everything here
//! ignores its own errors: the log is evidence, never a reason for the program to fail. Before
//! [`start`] (and in the library's tests, which never call it) every function does nothing.

use std::fmt::Write as _;
use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::version::utc;

/// How many run logs are kept (the newest).
pub const KEEP: usize = 20;

/// The prefix of the run logs' names.
pub const PREFIX: &str = "run";

/// How long after the start the watchdog looks for the first frame.
pub const WATCHDOG: Duration = Duration::from_secs(15);

/// The most library messages one run log takes (a library that warns on every frame must not
/// fill the disk); one more line says that the rest are left out.
pub const MAX_LIBRARY_LINES: usize = 2000;

/// The open run log.
struct Log {
    path: PathBuf,
    file: Mutex<File>,
    start: Instant,
    /// The text of the last [`stage`], for the watchdog.
    stage: Mutex<String>,
}

static LOG: OnceLock<Log> = OnceLock::new();
static FRAME: AtomicBool = AtomicBool::new(false);
static LIBRARY_LINES: AtomicUsize = AtomicUsize::new(0);

/// What the header says.
pub struct Header<'a> {
    pub version: &'a str,
    /// Unix time, seconds.
    pub secs: i64,
    /// The system and its version ([`system`]).
    pub system: &'a str,
    pub arch: &'a str,
    pub exe: &'a str,
    pub args: &'a [String],
    pub screen_reader: bool,
    /// The variables that are set ([`variables`]).
    pub vars: &'a [(String, String)],
}

/// The header's text.
pub fn header(h: &Header) -> String {
    let [y, mo, d, hh, mi, s] = utc(h.secs);
    let mut r = String::new();
    let _ = writeln!(r, "OpenRB-338 run log");
    let _ = writeln!(
        r,
        "time:          {y:04}-{mo:02}-{d:02} {hh:02}:{mi:02}:{s:02} UTC"
    );
    let _ = writeln!(r, "version:       {}", h.version);
    let _ = writeln!(r, "system:        {}", h.system);
    let _ = writeln!(r, "architecture:  {}", h.arch);
    let _ = writeln!(r, "executable:    {}", h.exe);
    let args = if h.args.is_empty() {
        "none".to_string()
    } else {
        h.args
            .iter()
            .map(|a| {
                if a.is_empty() || a.contains(' ') {
                    format!("\"{a}\"")
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let _ = writeln!(r, "arguments:     {args}");
    let sr = if h.screen_reader {
        "the system reports one running"
    } else {
        "the system reports none"
    };
    let _ = writeln!(r, "screen reader: {sr}");
    if h.vars.is_empty() {
        let _ = writeln!(r, "variables:     none");
    } else {
        let _ = writeln!(r, "variables:");
        for (k, v) in h.vars {
            let _ = writeln!(r, "  {k}={v}");
        }
    }
    let _ = writeln!(r);
    r
}

/// One line of the log: the milliseconds since the start, then the text; the further lines of a
/// text of several lines are indented under the first.
pub fn line(ms: u128, text: &str) -> String {
    let mut r = format!("{ms:>7} ms  ");
    for (i, l) in text.lines().enumerate() {
        if i > 0 {
            r.push_str("\n            ");
        }
        r.push_str(l);
    }
    r.push('\n');
    r
}

/// The level of the libraries' messages from `ORB_LOG`: `off`, `error`, `warn`, `info`, `debug`
/// or `trace` (any case); unset or anything else is `warn`.
pub fn level(v: Option<&str>) -> log::LevelFilter {
    use log::LevelFilter as L;
    match v.map(|s| s.trim().to_ascii_lowercase()).as_deref() {
        Some("off") => L::Off,
        Some("error") => L::Error,
        Some("info") => L::Info,
        Some("debug") => L::Debug,
        Some("trace") => L::Trace,
        _ => L::Warn,
    }
}

/// The variables the header lists, from `vars` (the environment): the names that start with
/// `ORB_`, `WGPU_` or `RUST_`, sorted.
pub fn variables(vars: impl Iterator<Item = (String, String)>) -> Vec<(String, String)> {
    let mut v: Vec<_> = vars
        .filter(|(k, _)| ["ORB_", "WGPU_", "RUST_"].iter().any(|p| k.starts_with(p)))
        .collect();
    v.sort();
    v
}

/// The name of a Windows version: `Windows 10 22H2 (10.0.19045.5737)`. Windows 11 reports
/// itself as 10.0 with a build of 22000 or more.
pub fn windows_name(
    major: u32,
    minor: u32,
    build: u32,
    ubr: Option<u32>,
    display: Option<&str>,
) -> String {
    let name = if major == 10 && build >= 22000 {
        "11".to_string()
    } else if major == 10 {
        "10".to_string()
    } else {
        format!("{major}.{minor}")
    };
    let display = display
        .filter(|d| !d.is_empty())
        .map(|d| format!(" {d}"))
        .unwrap_or_default();
    let ubr = ubr.map(|u| format!(".{u}")).unwrap_or_default();
    format!("Windows {name}{display} ({major}.{minor}.{build}{ubr})")
}

/// The name of a Windows exception code (an NTSTATUS), for the native crash line and the
/// exit codes of tools/win_diag.ps1.
pub fn exception_name(code: u32) -> &'static str {
    match code {
        0xC000_0005 => "access violation",
        0xC000_0006 => "in-page error",
        0xC000_001D => "illegal instruction",
        0xC000_0094 => "integer division by zero",
        0xC000_0096 => "privileged instruction",
        0xC000_00FD => "stack overflow",
        0xC000_0374 => "heap corruption",
        0xC000_0409 => "fail-fast abort (stack buffer overrun)",
        0x8000_0003 => "breakpoint",
        0xE06D_7363 => "C++ exception",
        _ => "unknown exception",
    }
}

/// The system and its version, for the header.
pub fn system() -> String {
    #[cfg(windows)]
    {
        win::system()
    }
    #[cfg(not(windows))]
    {
        std::env::consts::OS.to_string()
    }
}

/// Opens the run log of this start, writes the header, and sets up the libraries' logger, the
/// watchdog (after `watchdog`: [`WATCHDOG`], shorter only for `--debug-hang`) and (Windows) the
/// native crash line. Returns the log's path; `None` when there is no log folder or the file
/// cannot be made (the program runs on without a log). Only the first call opens a log.
pub fn start(version: &str, watchdog_after: Duration) -> Option<PathBuf> {
    let started = Instant::now();
    if let Some(l) = LOG.get() {
        return Some(l.path.clone());
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let dir = crate::crash::log_dir()?;
    let (path, mut file) = match crate::crash::create_log(&dir, PREFIX, secs) {
        Ok(f) => f,
        Err(e) => {
            let _ = writeln!(
                std::io::stderr(),
                "OpenRB-338: the run log could not be made in {}: {e}",
                dir.display()
            );
            return None;
        }
    };
    let _ = crate::crash::rotate(&dir, PREFIX, KEEP);
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let exe = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|e| format!("unknown ({e})"));
    let vars = variables(std::env::vars_os().map(|(k, v)| {
        (
            k.to_string_lossy().into_owned(),
            v.to_string_lossy().into_owned(),
        )
    }));
    let text = header(&Header {
        version,
        secs,
        system: &system(),
        arch: std::env::consts::ARCH,
        exe: &exe,
        args: &args,
        screen_reader: crate::screen_reader::system_reports_screen_reader(),
        vars: &vars,
    });
    let _ = file.write_all(text.as_bytes());
    let _ = file.flush();
    let log = LOG.get_or_init(|| Log {
        path,
        file: Mutex::new(file),
        start: started,
        stage: Mutex::new("the start".to_string()),
    });
    if log::set_logger(&LIBRARY).is_ok() {
        let v = std::env::var("ORB_LOG").ok();
        log::set_max_level(level(v.as_deref()));
    }
    watchdog(watchdog_after);
    #[cfg(windows)]
    win::install_native_crash_line();
    Some(log.path.clone())
}

/// The path of this start's run log, once [`start`] has opened it.
pub fn path() -> Option<&'static Path> {
    LOG.get().map(|l| l.path.as_path())
}

/// A stage of the start reached: a line, remembered as the last stage for the watchdog.
pub fn stage(text: &str) {
    if let Some(l) = LOG.get() {
        *l.stage.lock().unwrap_or_else(|e| e.into_inner()) = text.to_string();
        write(l, text);
    }
}

/// A line that is not a stage (a library message, the watchdog).
pub fn note(text: &str) {
    if let Some(l) = LOG.get() {
        write(l, text);
    }
}

/// The end of the run: `end: TEXT`.
pub fn end(text: &str) {
    stage(&format!("end: {text}"));
}

/// Called on every frame (`App::ui`): the first one is the stage "first frame drawn". After
/// the first, an atomic load.
pub fn frame() {
    if FRAME.load(Ordering::Relaxed) {
        return;
    }
    if !FRAME.swap(true, Ordering::Relaxed) {
        stage("first frame drawn");
    }
}

fn write(l: &Log, text: &str) {
    let s = line(l.start.elapsed().as_millis(), text);
    let mut f = l.file.lock().unwrap_or_else(|e| e.into_inner());
    let _ = f.write_all(s.as_bytes());
    let _ = f.flush();
}

/// [`note`] that never waits: when the file is in use (the crashing thread may hold it) the
/// line is lost. For the native crash filter.
#[cfg(windows)]
fn note_now(text: &str) {
    if let Some(l) = LOG.get()
        && let Ok(mut f) = l.file.try_lock()
    {
        let _ = f.write_all(line(l.start.elapsed().as_millis(), text).as_bytes());
        let _ = f.flush();
    }
}

/// After `after`, if no frame has been drawn and the run has not ended (a start error's or a
/// panic's box may still be open after the end line, which must stay the last line), a line
/// naming the last stage.
fn watchdog(after: Duration) {
    let _ = std::thread::Builder::new()
        .name("run-log-watchdog".into())
        .spawn(move || {
            std::thread::sleep(after);
            if !FRAME.load(Ordering::Relaxed)
                && let Some(l) = LOG.get()
            {
                let last = l.stage.lock().unwrap_or_else(|e| e.into_inner()).clone();
                if last.starts_with("end: ") {
                    return;
                }
                note(&format!(
                    "no frame drawn {} s after the start; the last stage reached: {last}",
                    after.as_secs()
                ));
            }
        });
}

/// The `log` crate's messages into the run log.
struct Library;

static LIBRARY: Library = Library;

impl log::Log for Library {
    fn enabled(&self, m: &log::Metadata) -> bool {
        m.level() <= log::max_level()
    }

    fn log(&self, r: &log::Record) {
        if !self.enabled(r.metadata()) {
            return;
        }
        let n = LIBRARY_LINES.fetch_add(1, Ordering::Relaxed);
        if n < MAX_LIBRARY_LINES {
            note(&format!("{} {}: {}", r.level(), r.target(), r.args()));
        } else if n == MAX_LIBRARY_LINES {
            note("further library messages are left out");
        }
    }

    fn flush(&self) {}
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    /// The system's name and real version: `RtlGetVersion` (GetVersionEx answers what the
    /// program's manifest allows), with the update revision (UBR) and the release name
    /// (DisplayVersion, `22H2`) from the registry.
    pub fn system() -> String {
        #[repr(C)]
        struct OsVersionInfoW {
            size: u32,
            major: u32,
            minor: u32,
            build: u32,
            platform: u32,
            csd: [u16; 128],
        }
        #[link(name = "ntdll")]
        unsafe extern "system" {
            fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
        }
        let mut v = OsVersionInfoW {
            size: size_of::<OsVersionInfoW>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform: 0,
            csd: [0; 128],
        };
        // SAFETY: RtlGetVersion fills the structure whose size is in `size`.
        if unsafe { RtlGetVersion(&mut v) } != 0 {
            return "Windows (version unknown)".to_string();
        }
        let ubr = reg_dword("UBR");
        let display = reg_string("DisplayVersion");
        super::windows_name(v.major, v.minor, v.build, ubr, display.as_deref())
    }

    const KEY: &str = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion";
    // HKEY_LOCAL_MACHINE: ((HKEY)(ULONG_PTR)((LONG)0x80000002)), sign-extended
    const HKLM: isize = 0x8000_0002_u32 as i32 as isize;
    const RRF_RT_REG_SZ: u32 = 0x2;
    const RRF_RT_REG_DWORD: u32 = 0x10;

    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn RegGetValueW(
            key: isize,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut c_void,
            size: *mut u32,
        ) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }

    fn reg_dword(value: &str) -> Option<u32> {
        let (k, n) = (wide(KEY), wide(value));
        let mut d: u32 = 0;
        let mut size = 4u32;
        // SAFETY: a DWORD value into a 4-byte buffer whose size is given.
        let r = unsafe {
            RegGetValueW(
                HKLM,
                k.as_ptr(),
                n.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut d as *mut u32).cast(),
                &mut size,
            )
        };
        (r == 0).then_some(d)
    }

    fn reg_string(value: &str) -> Option<String> {
        let (k, n) = (wide(KEY), wide(value));
        let mut buf = [0u16; 128];
        let mut size = (buf.len() * 2) as u32;
        // SAFETY: a string value into a buffer whose size in bytes is given; RegGetValueW
        // terminates it.
        let r = unsafe {
            RegGetValueW(
                HKLM,
                k.as_ptr(),
                n.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut size,
            )
        };
        if r != 0 {
            return None;
        }
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        Some(String::from_utf16_lossy(&buf[..len]))
    }

    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        record: *mut ExceptionRecord,
        address: *mut c_void,
        parameters: u32,
        information: [usize; 15],
    }

    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut c_void,
    }

    type Filter = unsafe extern "system" fn(*const ExceptionPointers) -> i32;

    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const EXCEPTION_EXECUTE_HANDLER: i32 = 1;
    const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x2;
    const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;

    unsafe extern "system" {
        fn SetUnhandledExceptionFilter(filter: Option<Filter>) -> Option<Filter>;
        fn GetModuleHandleExW(flags: u32, name: *const u16, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
    }

    /// Sets the filter that writes the native crash line.
    pub fn install_native_crash_line() {
        // SAFETY: sets a process-wide function pointer; the filter is a plain function.
        unsafe {
            SetUnhandledExceptionFilter(Some(filter));
        }
    }

    /// Writes the line, then hands the crash on: to Windows Error Reporting (its box, its
    /// LocalDumps), or with `ORB_UI_NO_CRASH_DIALOG` set (tests) straight to the end of the
    /// process, whose exit code is then the exception code. It allocates (the line's text):
    /// with a corrupt heap the line may be lost, and the crash goes on as before.
    unsafe extern "system" fn filter(p: *const ExceptionPointers) -> i32 {
        // SAFETY: Windows passes valid exception pointers to the filter.
        let text = unsafe { describe(p) };
        super::note_now(&format!("end: native crash: {text}"));
        if crate::crash::no_dialog() {
            EXCEPTION_EXECUTE_HANDLER
        } else {
            EXCEPTION_CONTINUE_SEARCH
        }
    }

    /// `access violation (0xC0000005) writing address 0x8, at 0x7FF6..., C:\...\X.exe+0x1234`.
    unsafe fn describe(p: *const ExceptionPointers) -> String {
        // SAFETY: the caller's pointers are valid or null.
        let Some(r) = (unsafe { p.as_ref().and_then(|p| p.record.as_ref()) }) else {
            return "no exception record".to_string();
        };
        let mut s = format!("{} (0x{:08X})", super::exception_name(r.code), r.code);
        if r.code == 0xC000_0005 && r.parameters >= 2 {
            let how = match r.information[0] {
                0 => "reading",
                1 => "writing",
                8 => "executing",
                _ => "using",
            };
            s.push_str(&format!(" {how} address 0x{:X}", r.information[1]));
        }
        let at = r.address as usize;
        s.push_str(&format!(", at 0x{at:X}"));
        let mut module: *mut c_void = std::ptr::null_mut();
        // SAFETY: FROM_ADDRESS takes the address as the name; UNCHANGED_REFCOUNT needs no
        // FreeLibrary. It takes the loader lock, which is recursive: the crashing thread gets
        // it again if it holds it, but if another thread holds it, this waits for that thread
        // and may hang instead of ending; rare, and the run log then has no end line, which
        // win_diag.ps1 reports as a hang.
        let found = unsafe {
            GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                    | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                r.address as *const u16,
                &mut module,
            )
        } != 0;
        if found {
            let mut buf = [0u16; 512];
            // SAFETY: the buffer and its length in characters.
            let n = unsafe { GetModuleFileNameW(module, buf.as_mut_ptr(), buf.len() as u32) };
            let name = String::from_utf16_lossy(&buf[..n as usize]);
            s.push_str(&format!(", {name}+0x{:X}", at - module as usize));
        } else {
            s.push_str(", in no module");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_has_every_field() {
        let args = ["--demo".to_string(), "C:\\My Songs\\a.rbs".to_string()];
        let vars = [("ORB_LOG".to_string(), "info".to_string())];
        // 2026-10-30 09:15:07 UTC
        let h = header(&Header {
            version: "2026.1107.1200 (abc1234)",
            secs: 1_793_351_707,
            system: "Windows 10 22H2 (10.0.19045.5737)",
            arch: "x86_64",
            exe: "C:\\OpenRB-338\\OpenRB-338.exe",
            args: &args,
            screen_reader: true,
            vars: &vars,
        });
        for want in [
            "OpenRB-338 run log\n",
            "time:          2026-10-30 09:15:07 UTC\n",
            "version:       2026.1107.1200 (abc1234)\n",
            "system:        Windows 10 22H2 (10.0.19045.5737)\n",
            "architecture:  x86_64\n",
            "executable:    C:\\OpenRB-338\\OpenRB-338.exe\n",
            "arguments:     --demo \"C:\\My Songs\\a.rbs\"\n",
            "screen reader: the system reports one running\n",
            "variables:\n  ORB_LOG=info\n",
        ] {
            assert!(h.contains(want), "{want}\n{h}");
        }
        let h = header(&Header {
            version: "v",
            secs: 0,
            system: "linux",
            arch: "aarch64",
            exe: "/x",
            args: &[],
            screen_reader: false,
            vars: &[],
        });
        assert!(h.contains("arguments:     none\n"), "{h}");
        assert!(
            h.contains("screen reader: the system reports none\n"),
            "{h}"
        );
        assert!(h.contains("variables:     none\n"), "{h}");
    }

    #[test]
    fn a_line_has_the_time_and_indents_further_lines() {
        assert_eq!(line(5, "audio output"), "      5 ms  audio output\n");
        assert_eq!(
            line(12345, "start error:\nno adapter"),
            "  12345 ms  start error:\n            no adapter\n"
        );
        assert_eq!(line(0, ""), "      0 ms  \n");
    }

    #[test]
    fn orb_log_sets_the_level_and_warn_is_the_default() {
        use log::LevelFilter as L;
        assert_eq!(level(None), L::Warn);
        assert_eq!(level(Some("")), L::Warn);
        assert_eq!(level(Some("nonsense")), L::Warn);
        assert_eq!(level(Some("Info")), L::Info);
        assert_eq!(level(Some(" debug ")), L::Debug);
        assert_eq!(level(Some("TRACE")), L::Trace);
        assert_eq!(level(Some("error")), L::Error);
        assert_eq!(level(Some("off")), L::Off);
    }

    #[test]
    fn the_header_lists_only_the_programs_variables() {
        let v = variables(
            [
                ("WGPU_BACKEND", "dx12"),
                ("PATH", "x"),
                ("ORB_LOG", "info"),
                ("RUST_BACKTRACE", "full"),
                ("ORBX", "no"),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string())),
        );
        let names: Vec<_> = v.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["ORB_LOG", "RUST_BACKTRACE", "WGPU_BACKEND"]);
    }

    #[test]
    fn windows_10_and_11_are_named_from_the_build() {
        assert_eq!(
            windows_name(10, 0, 19045, Some(5737), Some("22H2")),
            "Windows 10 22H2 (10.0.19045.5737)"
        );
        assert_eq!(
            windows_name(10, 0, 26100, Some(1), Some("24H2")),
            "Windows 11 24H2 (10.0.26100.1)"
        );
        assert_eq!(
            windows_name(10, 0, 22000, None, None),
            "Windows 11 (10.0.22000)"
        );
        assert_eq!(
            windows_name(6, 1, 7601, None, Some("")),
            "Windows 6.1 (6.1.7601)"
        );
    }

    #[test]
    fn exception_codes_have_names() {
        assert_eq!(exception_name(0xC000_0005), "access violation");
        assert_eq!(exception_name(0xC000_00FD), "stack overflow");
        assert_eq!(exception_name(0x1234), "unknown exception");
    }

    #[test]
    fn without_start_nothing_is_written_and_nothing_fails() {
        // the library's tests never call start
        stage("x");
        note("y");
        end("z");
        frame();
        assert!(path().is_none());
    }
}
