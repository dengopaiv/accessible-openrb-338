//! The run log end to end (task-214): every start writes `run-*.log` into the log folder, and
//! the failures of a start that the program can see are its last line. The runs use the hidden
//! debug options, which fail at the start before any window (no display needed), with the boxes
//! off (`ORB_UI_NO_CRASH_DIALOG`) and the log folder a temp folder (`ORB_UI_LOG_DIR`, so the
//! user's real log folder is never written).

use std::path::{Path, PathBuf};
use std::process::Command;

fn run(arg: &str, dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_OpenRB-338"))
        .arg(arg)
        .env("ORB_UI_LOG_DIR", dir.join("logs"))
        .env("ORB_UI_NO_CRASH_DIALOG", "1")
        .env("ORB_LOG", "info")
        // nothing of the user's: a settings file and a home that are temp
        .env("ORB_UI_CONFIG", dir.join("ui.toml"))
        .env("HOME", dir)
        .output()
        .unwrap()
}

/// The logs of `dir/logs` whose names start with `prefix`.
fn logs(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir.join("logs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_str().unwrap().starts_with(prefix))
        .collect();
    v.sort();
    v
}

/// The one run log of `dir`, as text.
fn run_log(dir: &Path) -> String {
    let runs = logs(dir, "run-");
    assert_eq!(runs.len(), 1, "{runs:?}");
    let name = runs[0].file_name().unwrap().to_str().unwrap();
    assert!(name.ends_with(".log"), "{name}");
    std::fs::read_to_string(&runs[0]).unwrap()
}

#[test]
fn a_start_error_exits_1_and_is_the_run_logs_end() {
    let t = tempfile::tempdir().unwrap();
    let o = run("--debug-start-error", t.path());
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("OpenRB-338: --debug-start-error: a deliberate start error"),
        "{err}"
    );
    let log = run_log(t.path());
    for want in [
        "OpenRB-338 run log\n",
        "version:       ",
        "system:        ",
        "architecture:  ",
        "executable:    ",
        "arguments:     --debug-start-error\n",
        "screen reader: the system reports ",
        "  ORB_LOG=info\n",
        "  ORB_UI_NO_CRASH_DIALOG=1\n",
        " ms  end: start error: --debug-start-error: a deliberate start error\n",
    ] {
        assert!(log.contains(want), "{want}\n{log}");
    }
    if cfg!(windows) {
        assert!(log.contains("system:        Windows "), "{log}");
    }
    // the end is the last line
    assert!(
        log.trim_end()
            .lines()
            .last()
            .unwrap()
            .contains("end: start error"),
        "{log}"
    );
}

#[test]
fn a_panic_names_its_crash_log_in_the_run_log() {
    let t = tempfile::tempdir().unwrap();
    let o = run("--debug-panic", t.path());
    assert_eq!(o.status.code(), Some(101), "{o:?}");
    let crash = logs(t.path(), "crash-");
    assert_eq!(crash.len(), 1, "{crash:?}");
    let log = run_log(t.path());
    let last = log.trim_end().lines().last().unwrap().to_string();
    assert!(
        last.contains("end: panic on thread main: --debug-panic: a deliberate panic (")
            && last.contains("main.rs:"),
        "{log}"
    );
    assert!(
        last.contains(&format!("crash log {}", crash[0].display())),
        "{log}"
    );
}

/// `--debug-hang` sets the watchdog to 1 s and waits 3 s before any window: the watchdog's line
/// names the last stage, then the start error ends the log.
#[test]
fn the_watchdog_names_the_last_stage_when_no_frame_comes() {
    let t = tempfile::tempdir().unwrap();
    let o = run("--debug-hang", t.path());
    assert_eq!(o.status.code(), Some(1), "{o:?}");
    let log = run_log(t.path());
    let lines: Vec<&str> = log.lines().collect();
    let at = |want: &str| {
        lines
            .iter()
            .position(|l| l.contains(want))
            .unwrap_or_else(|| panic!("{want}\n{log}"))
    };
    let stage = at(" ms  --debug-hang: waiting 3 s with no window");
    let dog = at(
        " ms  no frame drawn 1 s after the start; the last stage reached: --debug-hang: waiting 3 \
         s with no window",
    );
    let end = at(" ms  end: start error: --debug-hang: the wait is over");
    assert!(stage < dog && dog < end, "{log}");
}

#[test]
fn each_start_has_its_own_run_log() {
    let t = tempfile::tempdir().unwrap();
    for _ in 0..3 {
        run("--debug-start-error", t.path());
    }
    assert_eq!(logs(t.path(), "run-").len(), 3);
    assert!(logs(t.path(), "crash-").is_empty());
}

/// A native crash (an access violation) is written down by the exception filter; with the
/// boxes off the process ends with the exception code (no Windows Error Reporting box).
#[cfg(windows)]
#[test]
fn a_native_crash_is_the_run_logs_end() {
    let t = tempfile::tempdir().unwrap();
    let o = run("--debug-native-crash", t.path());
    assert_eq!(o.status.code(), Some(0xC000_0005_u32 as i32), "{o:?}");
    let log = run_log(t.path());
    let last = log.trim_end().lines().last().unwrap().to_string();
    for want in [
        "end: native crash: access violation (0xC0000005) writing address 0x8, at 0x",
        "OpenRB-338.exe+0x",
    ] {
        assert!(last.contains(want), "{want}\n{log}");
    }
    assert!(
        log.contains("--debug-native-crash: writing to address 0x8\n"),
        "{log}"
    );
    assert!(logs(t.path(), "crash-").is_empty());
}
