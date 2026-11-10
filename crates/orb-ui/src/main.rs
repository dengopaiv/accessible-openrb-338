//! `OpenRB-338`: the panel window, wired to the real-time player (task-045). The binary has this
//! name (crate `orb-ui`) because the macOS Dock and Activity Monitor show the executable's name.
//!
//! ```text
//! OpenRB-338 [SONG]
//! ```
//! `--help` shows only this. The options below are for developers, tests and screenshots
//! (task-137): hidden from `--help` and the usage text, but they all work (docs/building.md,
//! "OpenRB-338 developer options"):
//!
//! ```text
//! --demo  --size WxH  --ppp S  --browse open|save|export|mod  --about  --info
//! --screenshot OUT.png  --play
//! ```
//!
//! `-h` is `--help`; an argument `-psn_...` (Finder on old macOS) is ignored; a bad option prints
//! the usage and exits 2.
//!
//! With a song it opens that song, with the mod it names (looked up in the Mods folder of the
//! settings, `ORB_MOD_DIR`, or the default places of `orb render`); without one the panel starts on
//! a new empty song. The menu bar (task-047, docs/ui_spec.md section 18) opens, saves and exports
//! songs, chooses the mod and sets the Vintage flag and the keyboard mode. The drum machines play
//! the built-in kit "Standard ReBirth", the standard mod orb embeds (`assets/Standard.rbm`), or
//! the song's mod laid over it. `--demo`
//! starts the panel in a lively state with no player (a picture). The window starts maximized;
//! `--size` opens it, not maximized, with that inner size of the panel in points (624x510 is
//! the native size; the window is as high as the menu bar more, for screenshots), `--ppp`
//! overrides the pixel density. `--screenshot` saves the window to a PNG after a few frames and
//! quits. Ctrl/Cmd+O opens the in-app file browser (task-046) to open a song, Ctrl/Cmd+S saves;
//! `--about` starts with the About window open (for screenshots, task-115), `--info` with File >
//! Info's Song Information window (task-117). `--play` presses Play once the song is open (CPU
//! measurements, task-153).
//! `--browse MODE` starts with the browser open for a mode. The browser's folders are kept in the
//! settings file (`$ORB_UI_CONFIG`, else in portable mode `data/ui.toml` beside the program,
//! task-223, else `OpenRB-338/ui.toml` in the configuration folder).

// The Windows release build is a GUI-subsystem program: no console window opens behind the
// panel (task-111). `orb` is not affected (this attribute is in the UI binary only). A debug
// build keeps the console, for `dbg!` and panics.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use eframe::egui;
use orb_engine::tables::Tables;
use orb_ui::app::{App, Shot};
use orb_ui::browser::Mode;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::link::Link;
use orb_ui::menu::MENU_H;
use orb_ui::model::Panel;
use orb_ui::runlog;
use orb_ui::session;
use orb_ui::settings::Settings;

/// The visible help: only the song. The developer options are hidden (task-137).
const USAGE: &str = "usage: OpenRB-338 [SONG]\n\
                     SONG is a ReBirth song (.rbs, RBS 4.x or ReBirth 1.x) to open";

/// The command line, parsed.
#[derive(Debug, PartialEq)]
struct Args {
    demo: bool,
    about: bool,
    info: bool,
    /// `--play`: press Play once the song is open (CPU measurements, task-153).
    play: bool,
    size: [f32; 2],
    sized: bool,
    ppp: Option<f32>,
    shot: Option<PathBuf>,
    song: Option<PathBuf>,
    browse: Option<Mode>,
    debug_panic: Option<DebugPanic>,
    debug_fail: Option<DebugFail>,
}

/// The failures of a start that are not panics, on purpose (task-214): tests of the run log.
/// Both happen at the start, before any window (they run with no display).
#[derive(Debug, PartialEq, Clone, Copy)]
enum DebugFail {
    /// `--debug-start-error`: the path of an error of `eframe::run_native`.
    StartError,
    /// `--debug-native-crash`: a write to an invalid address (on Windows an access violation,
    /// which the run log's exception filter writes down).
    NativeCrash,
    /// `--debug-hang`: the watchdog set to 1 s, then 3 s with no frame (its line), then the
    /// start error path.
    Hang,
}

/// Where `--debug-panic` panics on purpose (task-157): tests of the crash hook.
#[derive(Debug, PartialEq, Clone, Copy)]
enum DebugPanic {
    /// On the main thread at the start, before any window (runs with no display).
    Start,
    /// On the main thread, once the window is up.
    Window,
    /// On another thread, half a second after the window is up.
    Thread,
}

/// What the command line asks instead of running the app.
#[derive(Debug, PartialEq)]
enum Exit {
    /// `--help`: print the usage, exit 0.
    Help,
    /// A bad option or value: print the usage, exit 2.
    Bad,
}

/// Parses the arguments after the program name. `-psn_...` (Finder on old macOS) is ignored.
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, Exit> {
    let mut r = Args {
        demo: false,
        about: false,
        info: false,
        play: false,
        size: [PANEL_W, PANEL_H],
        sized: false,
        ppp: None,
        shot: None,
        song: None,
        browse: None,
        debug_panic: None,
        debug_fail: None,
    };
    let mut args = args.into_iter();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--help" | "-h" => return Err(Exit::Help),
            "--demo" => r.demo = true,
            "--about" => r.about = true,
            "--info" => r.info = true,
            "--play" => r.play = true,
            "--debug-panic" => r.debug_panic = Some(DebugPanic::Start),
            "--debug-panic-window" => r.debug_panic = Some(DebugPanic::Window),
            "--debug-panic-thread" => r.debug_panic = Some(DebugPanic::Thread),
            "--debug-start-error" => r.debug_fail = Some(DebugFail::StartError),
            "--debug-native-crash" => r.debug_fail = Some(DebugFail::NativeCrash),
            "--debug-hang" => r.debug_fail = Some(DebugFail::Hang),
            "--size" => {
                let v = args.next().ok_or(Exit::Bad)?;
                let (w, h) = v.split_once('x').ok_or(Exit::Bad)?;
                r.size = [
                    w.parse().map_err(|_| Exit::Bad)?,
                    h.parse().map_err(|_| Exit::Bad)?,
                ];
                r.sized = true;
            }
            "--ppp" => r.ppp = Some(args.next().and_then(|v| v.parse().ok()).ok_or(Exit::Bad)?),
            "--screenshot" => r.shot = Some(args.next().ok_or(Exit::Bad)?.into()),
            "--browse" => {
                r.browse = Some(match args.next().as_deref() {
                    Some("open") => Mode::OpenSong,
                    Some("save") => Mode::SaveSong,
                    Some("export") => Mode::Export,
                    Some("mod") => Mode::ChooseMod,
                    _ => return Err(Exit::Bad),
                })
            }
            s if s.starts_with("-psn_") => {}
            s if !s.starts_with('-') && r.song.is_none() => r.song = Some(s.into()),
            _ => return Err(Exit::Bad),
        }
    }
    Ok(r)
}

fn usage(e: Exit) -> ! {
    match e {
        Exit::Help => {
            println!("{USAGE}");
            std::process::exit(0);
        }
        Exit::Bad => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}

/// Gives a GUI-subsystem process the console of the program that started it (task-111).
///
/// Such a process has no console, so `eprintln!` (the usage text, a song that does not open, the
/// audio error) goes nowhere. When it is started from `cmd` or PowerShell, this attaches to that
/// console and points the standard output and error at it, unless they were redirected (a file
/// or a pipe stays as it is). Started from the Start menu or Explorer there is no parent
/// console and nothing happens. What stays lost: `cmd` does not wait for a GUI program, so the
/// prompt comes back at once and the text appears over it; `start /wait OpenRB-338.exe` waits.
#[cfg(all(windows, not(debug_assertions)))]
fn attach_parent_console() {
    use std::fs::OpenOptions;
    use std::os::windows::io::IntoRawHandle;

    type Handle = *mut core::ffi::c_void;
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const STD_ERROR_HANDLE: u32 = -12i32 as u32;
    unsafe extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
        fn GetStdHandle(std_handle: u32) -> Handle;
        fn SetStdHandle(std_handle: u32, handle: Handle) -> i32;
    }
    // SAFETY: plain Win32 calls; the handle given to SetStdHandle is leaked on purpose, it must
    // live as long as the process.
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            return;
        }
        for id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let h = GetStdHandle(id);
            // none (null) or invalid (-1): not redirected, so use the console
            if (h.is_null() || h as isize == -1)
                && let Ok(f) = OpenOptions::new().write(true).open("CONOUT$")
            {
                SetStdHandle(id, f.into_raw_handle());
            }
        }
    }
}

/// A start that failed without a panic (an error of `eframe::run_native`, task-214): the error
/// goes to stderr, the run log and the native box (unless `ORB_UI_NO_CRASH_DIALOG`), and the
/// exit code is 1. Started from Explorer or the Start menu the release build has no visible
/// stderr, so before this such a start showed nothing at all.
fn start_failed(error: &str) -> ExitCode {
    let _ = writeln!(std::io::stderr(), "OpenRB-338: {error}");
    runlog::end(&format!("start error: {error}"));
    if !orb_ui::crash::no_dialog() {
        let log = match runlog::path() {
            Some(p) => format!("The run log is {}.", p.display()),
            None => "No run log could be written.".to_string(),
        };
        let _ = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("OpenRB-338")
            .set_description(format!("OpenRB-338 could not start.\n\n{error}\n\n{log}"))
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
    ExitCode::from(1)
}

fn main() -> ExitCode {
    #[cfg(all(windows, not(debug_assertions)))]
    attach_parent_console();
    // before anything that can panic: a panic writes a crash log and shows a box (task-157)
    orb_ui::crash::install(orb_ui::about::VERSION);
    let Args {
        demo,
        about,
        info,
        play,
        size,
        sized,
        ppp,
        shot,
        song,
        browse,
        debug_panic,
        debug_fail,
    } = parse_args(std::env::args().skip(1)).unwrap_or_else(|e| usage(e));
    // the run log of this start (task-214), before anything that can fail
    let watchdog = match debug_fail {
        Some(DebugFail::Hang) => std::time::Duration::from_secs(1),
        _ => runlog::WATCHDOG,
    };
    runlog::start(orb_ui::about::VERSION, watchdog);
    // portable mode (task-223) and where the settings and the logs really are (ORB_UI_CONFIG
    // and ORB_UI_LOG_DIR win over it)
    let shown = |p: Option<std::path::PathBuf>| match p {
        Some(p) => p.display().to_string(),
        None => "none".to_string(),
    };
    let mode = match orb_ui::portable::root() {
        Some(_) => format!("portable ({} found)", orb_ui::portable::MARKER),
        None => "not portable".to_string(),
    };
    runlog::note(&format!(
        "{mode}: settings file {}; logs in {}",
        shown(orb_ui::settings::default_file()),
        shown(orb_ui::crash::log_dir())
    ));
    if debug_panic == Some(DebugPanic::Start) {
        panic!("--debug-panic: a deliberate panic");
    }
    match debug_fail {
        Some(DebugFail::StartError) => {
            return start_failed("--debug-start-error: a deliberate start error");
        }
        Some(DebugFail::Hang) => {
            runlog::stage("--debug-hang: waiting 3 s with no window");
            std::thread::sleep(std::time::Duration::from_secs(3));
            return start_failed("--debug-hang: the wait is over");
        }
        Some(DebugFail::NativeCrash) => {
            runlog::stage("--debug-native-crash: writing to address 0x8");
            // SAFETY: not safe: undefined behavior on purpose, a write that faults to test the
            // native crash line. 8 is aligned and not null, so the pointer
            // checks of a debug build pass and the write itself faults.
            unsafe { std::ptr::without_provenance_mut::<u32>(8).write_volatile(1) };
        }
        None => {}
    }

    let panel = if demo { Panel::demo() } else { Panel::new() };
    let mut link = None;
    let mut app_samples = None;
    let mut startup = None;
    // errors of the start, shown in the message box once the app is up (task-130)
    let mut start_errors: Vec<String> = Vec::new();
    // keeps the audio stream alive for the whole run
    let mut _stream = None;
    let mut audio_health = None;
    if !demo {
        let samples = Some(rbm::standard::builtin_samples());
        app_samples = samples.clone();
        let (l, audio) = Link::new(Arc::new(Tables::new()), samples.clone());
        // The song named on the command line is opened by the app (with its mod).
        if let Some(path) = &song {
            runlog::stage(&format!("opening the song {}", path.display()));
            match session::open_song(path) {
                Ok(o) => {
                    runlog::stage("song opened");
                    startup = Some((path.clone(), o));
                }
                // the app starts on the new song, with the error in its message box
                Err(e) => {
                    eprintln!("OpenRB-338: {e}");
                    runlog::stage(&format!("the song did not open: {e}"));
                    start_errors.push(e);
                }
            }
        }
        runlog::stage("starting the audio output");
        match orb_player::output::start(audio) {
            Ok(s) => {
                runlog::stage(&format!(
                    "audio output: \"{}\", {} Hz, {} channels, {}{}",
                    s.device,
                    s.config.sample_rate,
                    s.config.channels,
                    s.format,
                    if s.resampled { ", resampled" } else { "" }
                ));
                audio_health = Some(s.health());
                _stream = Some(s);
            }
            Err(e) => {
                eprintln!("OpenRB-338: audio output: {e}; the panel runs without sound");
                runlog::stage(&format!("audio output failed: {e}"));
                // the one failure at the launch is an error the user expects to hear of: the
                // box. Later audio errors repeat: `App::notify`.
                start_errors.push(format!("Audio output: {e}; the panel runs without sound"));
            }
        }
        link = Some(l);
    }

    // The window starts maximized; an explicit --size opens at that size instead.
    let viewport = egui::ViewportBuilder::default()
        .with_title("OpenRB-338")
        .with_app_id("OpenRB-338")
        .with_icon(Arc::new(orb_ui::art::app_icon(256)))
        // the menu bar is above the panel
        .with_inner_size([size[0], size[1] + MENU_H])
        // The minimum is half the native panel size (decision task-040).
        .with_min_inner_size([PANEL_W / 2.0, PANEL_H / 2.0 + MENU_H])
        .with_maximized(!sized);
    // the graphics adapter: on Windows DirectX 12 first, Vulkan not loaded (task-217)
    let graphics = orb_ui::graphics::Rule::from_env();
    runlog::stage(&graphics.describe());
    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions {
        viewport,
        wgpu_options: orb_ui::graphics::wgpu_options(&graphics),
        ..Default::default()
    };
    // Windows: the numpad's keys apart from the main keyboard's (task-225)
    #[cfg(windows)]
    {
        use winit::platform::windows::EventLoopBuilderExtWindows;
        options.event_loop_builder = Some(Box::new(|builder| {
            builder.with_msg_hook(orb_ui::numpad::hook);
        }));
    }
    runlog::stage("creating the window and the graphics (eframe::run_native)");
    let run = eframe::run_native(
        "OpenRB-338",
        options,
        Box::new(move |cc| {
            // always dark, whatever the system theme (task-127; `App::show` keeps it)
            cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
            match &cc.wgpu_render_state {
                Some(rs) => {
                    let i = rs.adapter.get_info();
                    runlog::stage(&format!(
                        "window created; graphics: {:?}, {:?} \"{}\", driver \"{}\" {}",
                        i.backend, i.device_type, i.name, i.driver, i.driver_info
                    ));
                    // ORB_GPU_INFO=1 prints the adapter wgpu chose (scripts/appimage-smoke.sh)
                    if std::env::var_os("ORB_GPU_INFO").is_some_and(|v| v == "1") {
                        eprintln!("gpu: {:?} {:?} {}", i.backend, i.device_type, i.name);
                    }
                }
                None => runlog::stage("window created; graphics: no wgpu state"),
            }
            if let Some(ppp) = ppp {
                cc.egui_ctx.set_pixels_per_point(ppp);
            }
            match debug_panic {
                Some(DebugPanic::Window) => panic!("--debug-panic-window: a deliberate panic"),
                Some(DebugPanic::Thread) => {
                    std::thread::Builder::new()
                        .name("debug-panic".into())
                        .spawn(|| {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            panic!("--debug-panic-thread: a deliberate panic");
                        })
                        .expect("spawn the debug thread");
                }
                Some(DebugPanic::Start) | None => {}
            }
            let shot = shot.map(Shot::new);
            let mut app = match link {
                Some(l) => App::with_link(panel, l, shot),
                None => App::new(panel, shot),
            };
            app.samples = app_samples;
            app.audio_health = audio_health;
            app.menu = true;
            app.enable_logo_anim();
            app.settings = Settings::load();
            // the real screen reader check and the system's file dialogs (task-194)
            app.use_system_services();
            // the song (or the new empty song) and its mod's startup sound; nothing in the demo,
            // which has no player
            // (the errors of the start are one box)
            app.action(|app| {
                app.start(startup);
                for e in start_errors {
                    app.fail(e);
                }
            });
            if let Some(mode) = browse {
                app.open_browser(mode);
            }
            if about {
                app.show_about();
            }
            if info {
                app.show_info();
            }
            if play {
                app.panel.press("transport.play", 0.0);
            }
            runlog::stage("the app is ready; waiting for the first frame");
            Ok(Box::new(app))
        }),
    );
    match run {
        Ok(()) => {
            runlog::end("the window was closed");
            ExitCode::SUCCESS
        }
        Err(e) => start_failed(&e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(a: &[&str]) -> Result<Args, Exit> {
        parse_args(a.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_arguments_is_the_new_song() {
        let a = parse(&[]).unwrap();
        assert_eq!(a.song, None);
        assert!(!a.demo && !a.about && !a.info && !a.sized && !a.play);
        assert_eq!(a.size, [PANEL_W, PANEL_H]);
    }

    #[test]
    fn help_is_exit_0_and_a_bad_option_is_exit_2() {
        assert_eq!(parse(&["--help"]), Err(Exit::Help));
        assert_eq!(parse(&["-h"]), Err(Exit::Help));
        assert_eq!(parse(&["song.rbs", "--help"]), Err(Exit::Help));
        for bad in [
            &["--nope"][..],
            &["-x"],
            &["--size"],
            &["--size", "10"],
            &["--size", "axb"],
            &["--ppp", "x"],
            &["--screenshot"],
            &["--browse", "other"],
            &["a.rbs", "b.rbs"],
        ] {
            assert_eq!(parse(bad), Err(Exit::Bad), "{bad:?}");
        }
    }

    #[test]
    fn the_hidden_options_still_parse() {
        let a = parse(&[
            "--demo",
            "--size",
            "300x200",
            "--ppp",
            "2",
            "--browse",
            "export",
            "--about",
            "--info",
            "--screenshot",
            "out.png",
            "--play",
        ])
        .unwrap();
        assert!(a.demo && a.about && a.info && a.sized && a.play);
        assert_eq!(a.size, [300.0, 200.0]);
        assert_eq!(a.ppp, Some(2.0));
        assert_eq!(a.browse, Some(Mode::Export));
        assert_eq!(a.shot, Some("out.png".into()));
        assert_eq!(a.debug_panic, None);
        assert_eq!(a.debug_fail, None);
    }

    #[test]
    fn the_debug_failures_parse() {
        let a = parse(&["--debug-panic-window"]).unwrap();
        assert_eq!(a.debug_panic, Some(DebugPanic::Window));
        let a = parse(&["--debug-start-error"]).unwrap();
        assert_eq!(a.debug_fail, Some(DebugFail::StartError));
        let a = parse(&["--debug-native-crash"]).unwrap();
        assert_eq!(a.debug_fail, Some(DebugFail::NativeCrash));
        let a = parse(&["--debug-hang"]).unwrap();
        assert_eq!(a.debug_fail, Some(DebugFail::Hang));
        for hidden in [
            "--debug-start-error",
            "--debug-native-crash",
            "--debug-hang",
        ] {
            assert!(!USAGE.contains(hidden), "{hidden}");
        }
    }

    #[test]
    fn the_song_is_a_path_with_spaces_and_psn_is_ignored() {
        let a = parse(&["-psn_0_12345", "/my songs/a b.rbs"]).unwrap();
        assert_eq!(a.song, Some("/my songs/a b.rbs".into()));
        let a = parse(&["-psn_0_12345"]).unwrap();
        assert_eq!(a.song, None);
    }

    #[test]
    fn the_usage_shows_only_the_song() {
        assert!(USAGE.contains("[SONG]"));
        for hidden in [
            "--demo",
            "--size",
            "--ppp",
            "--browse",
            "--screenshot",
            "--about",
            "--info",
            "--play",
        ] {
            assert!(!USAGE.contains(hidden), "{hidden}");
        }
    }
}
