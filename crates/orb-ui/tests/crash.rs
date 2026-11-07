//! The crash hook end to end (task-157): `OpenRB-338 --debug-panic` panics on purpose before any
//! window, with the box off (`ORB_UI_NO_CRASH_DIALOG`) and the log folder a temp folder
//! (`ORB_UI_LOG_DIR`, so the user's real log folder is never written). The process exits with
//! 101, stderr has the panic text, and one `crash-*.log` appears with the fields.

use std::process::Command;

fn run(log_dir: &std::path::Path, home: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_OpenRB-338"))
        .arg("--debug-panic")
        .env("ORB_UI_LOG_DIR", log_dir)
        .env("ORB_UI_NO_CRASH_DIALOG", "1")
        // nothing of the user's: a settings file and a home that are temp
        .env("ORB_UI_CONFIG", home.join("ui.toml"))
        .env("HOME", home)
        .output()
        .unwrap()
}

/// The crash logs of `dir` (it also holds the run logs, task-214).
fn crash_logs(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("crash-")
        })
        .collect()
}

#[test]
fn a_deliberate_panic_exits_101_and_writes_the_log() {
    let t = tempfile::tempdir().unwrap();
    let logs = t.path().join("logs");
    let o = run(&logs, t.path());
    assert_eq!(o.status.code(), Some(101), "{o:?}");
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("--debug-panic: a deliberate panic"), "{err}");
    assert!(err.contains("crash log saved to"), "{err}");

    // the crash log, beside the run log of the start (task-214)
    let files = crash_logs(&logs);
    assert_eq!(files.len(), 1, "{files:?}");
    let name = files[0].file_name().unwrap().to_str().unwrap();
    assert!(
        name.starts_with("crash-") && name.ends_with(".log"),
        "{name}"
    );
    let log = std::fs::read_to_string(&files[0]).unwrap();
    for want in [
        "OpenRB-338 crash log",
        "UTC",
        "version:  ",
        "thread:   main",
        "message:  --debug-panic: a deliberate panic",
        "main.rs:",
        "backtrace:",
        std::env::consts::OS,
        std::env::consts::ARCH,
    ] {
        assert!(log.contains(want), "{want}\n{log}");
    }
    // a backtrace with a frame of the hook, though RUST_BACKTRACE is not set
    let bt = log.split_once("\nbacktrace:\n").unwrap().1;
    assert!(bt.contains("   0: "), "{log}");
    // The frame of the hook is named only where symbols exist. A Windows (MSVC) release build
    // has no debug info (the profile, a decision of the user), so the frames come from the
    // exe's public symbols only: the hook's closure is named after a neighbouring function
    // (`default_read_to_end`, `TEXtChunk::encode`; task-200) and `orb_ui::crash` is absent. The
    // debug build, which CI runs, has the PDB and keeps the check.
    if cfg!(all(windows, not(debug_assertions))) {
        assert!(bt.contains("std::panicking"), "{log}");
    } else {
        assert!(bt.contains("orb_ui::crash"), "{log}");
    }
}

#[test]
fn a_second_crash_adds_a_second_log() {
    let t = tempfile::tempdir().unwrap();
    let logs = t.path().join("logs");
    for _ in 0..2 {
        assert_eq!(run(&logs, t.path()).status.code(), Some(101));
    }
    assert_eq!(crash_logs(&logs).len(), 2);
}
