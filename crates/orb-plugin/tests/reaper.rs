//! task-244: the plugin in REAPER. The committed projects of `tests/reaper/` (made by
//! `examples/reaper_fixtures.rs` from `tests/reaper/template.RPP`) are rendered through
//! REAPER's command line and compared with the plugin's process side driven in-process the way
//! REAPER drives it, and, for the note-mode drag-outs, with orb-player's pattern-mode render of
//! the same song through the same process side (`docs/plugin.md` section 7.1).
//!
//! Opt-in: the render test runs only when `ORB_REAPER` names the REAPER executable (on macOS
//! `/Applications/REAPER.app/Contents/MacOS/REAPER`) and skips with a message otherwise. CI does
//! not run it (REAPER is licensed software). The fixtures' freshness test always runs.
//!
//! REAPER runs as a new instance (`-newinst`, so a REAPER the user has open is neither used nor
//! disturbed) on a scratch resource folder of its own under `target/reaper/` (`-cfgfile
//! <scratch>/reaper.ini` moves the whole resource folder, the plugin caches included). Its
//! plugin paths list only `target/bundled`, where `cargo xtask bundle orb-plugin --release` has
//! just put the bundles; nothing is installed, and the user's REAPER configuration and caches
//! are never read or written. The plugin log (`ORB_PLUGIN_LOG`, set to a scratch file) has a
//! line per activation naming the library that runs; they must name only the fresh bundle's
//! library, so a test that ran some other copy fails.
//!
//! REAPER has no headless mode; the test keeps it out of the user's sight (task-277, macOS):
//! it starts it through `open -n -g -j -W` (a new instance, not activated, hidden, waited for;
//! `--env` passes `ORB_PLUGIN_LOG`), and a hider (`osascript` running a JXA loop, [`HIDER`])
//! hides the app again whenever REAPER unhides itself to open its windows, which `-j` alone
//! does not prevent. `open` exits 0 whatever REAPER's status, so a render counts when it wrote
//! its files. A REAPER that hangs is found by its scratch `-cfgfile` path in `ps` and killed,
//! and only that one ([`reapers_of`]). The measurements are in `docs/plugin.md` section 7.1.

#[path = "reaper/cases.rs"]
mod cases;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use cases::{CASES, Case, DEVICE_TRACKS, Format, Timing};
use orb_player::midi::Message;
use orb_plugin::core::{AuxOut, DEVICE_OUTPUTS, HostEvent, HostTransport};
use orb_plugin::instance::{self, Mode, OrbParams};
use orb_plugin::plugin_log::{LEVEL_VAR, LOG_VAR, parse_loaded_from};

/// One LSB of the int16 output.
const LSB: f32 = 1.0 / 32768.0;

/// How long one render may take before the test gives up and stops the REAPER it started.
const RENDER_TIMEOUT: Duration = Duration::from_secs(120);

/// REAPER 7.86's VST3 host writes 0 for an output sample below about 1e-16 in magnitude
/// (-320 dB; seen on the 909's device output, whose decays reach 3e-17): a device output's sample
/// counts as equal when REAPER's is 0 and the plugin's is below this.
const FLUSHED: f32 = 1e-16;

/// Ceilings, not a model: the largest differences measured (REAPER 7.86 and in-process alike;
/// the drums' equal `tests/note_clock.rs`'s at the same tempo) of a note-mode render at the
/// clip's times from the pattern's render. task-227's timing rule (a note sounds in the first
/// block at or after its frame, a step in the block whose position after its advance reaches
/// it) puts most hits one block, 64 frames, later, so the same loud hits a block apart differ
/// by about their peak, and the 303's waveform depends on when a note starts. These bounds
/// catch only gross changes; the real check against the pattern is the aligned variant
/// ([`Timing::Aligned`]: within one LSB), the clip's times are checked exactly against the
/// in-process host.
const CLIP_CEILING: [(&str, f32); 3] = [
    ("tb303_slide_clip", 0.14968872),
    ("tr808_pcf_clip", 1.5874329),
    ("tr909_pcf_clip", 1.7430725),
];

#[test]
fn the_fixtures_are_current() {
    let template = cases::template();
    for c in &CASES {
        for f in Format::ALL {
            let path = cases::fixture_path(c, f);
            let have = std::fs::read_to_string(&path).unwrap_or_default();
            assert!(
                have == cases::project(c, f, &template),
                "{} is stale: regenerate the fixtures with `cargo run -p orb-plugin --example \
                 reaper_fixtures`",
                path.display()
            );
        }
    }
}

/// The workspace's target folder.
fn target_dir() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| cases::crate_dir().join("../../target"))
}

/// Builds the bundles (`cargo xtask bundle orb-plugin --release`); their folder.
fn bundles() -> PathBuf {
    let root = cases::crate_dir().join("../..");
    let status = Command::new(env!("CARGO"))
        .args(["xtask", "bundle", "orb-plugin", "--release"])
        .current_dir(&root)
        .status()
        .expect("cargo runs");
    assert!(status.success(), "cargo xtask bundle orb-plugin --release");
    let dir = target_dir().join("bundled");
    for f in Format::ALL {
        let b = dir.join(format!("OpenRB-338.{}", f.ext()));
        assert!(b.is_dir(), "no bundle {}", b.display());
    }
    dir.canonicalize().unwrap()
}

/// The `reaper.ini` key of the VST plugin path of REAPER 7.86 on macOS (a string of its
/// executable; `docs/plugin.md` section 7), and the plugin cache's file name.
struct Keys {
    vst_path: &'static str,
    vst_cache: &'static str,
}

fn keys() -> Option<Keys> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    Some(if cfg!(target_arch = "aarch64") {
        Keys {
            vst_path: "vstpath_arm64",
            vst_cache: "reaper-vstplugins_arm64.ini",
        }
    } else {
        Keys {
            vst_path: "vstpath64",
            vst_cache: "reaper-vstplugins64.ini",
        }
    })
}

/// The scratch `reaper.ini`: the plugin path lists only `bundles`; `vstfullstate` keeps REAPER
/// from adding the default VST3 folders to them on its first start; the audio device is
/// REAPER's dummy (no CoreAudio device in or out, after `coreaudiobs` marks the audio as
/// configured), so REAPER neither asks for a device nor opens one.
fn reaper_ini(bundles: &Path, k: &Keys) -> String {
    let b = bundles.display();
    format!(
        "[REAPER]\n{}={b}\nvstfullstate=33605\ncoreaudiobs=512\n\
         coreaudioindevnew=<none>\ncoreaudiooutdevnew=<none>\n",
        k.vst_path
    )
}

/// A float WAV REAPER rendered: its rate and its frames, interleaved stereo.
fn read_wav(path: &Path) -> (u32, Vec<f32>) {
    let d = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(&d[..4], b"RIFF");
    assert_eq!(&d[8..12], b"WAVE");
    let u16_at = |i: usize| u16::from_le_bytes([d[i], d[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);
    let mut i = 12;
    let mut rate = 0;
    while i + 8 <= d.len() {
        let len = u32_at(i + 4) as usize;
        let body = i + 8;
        match &d[i..i + 4] {
            b"fmt " => {
                assert_eq!(u16_at(body), 3, "{}: IEEE float", path.display());
                assert_eq!(u16_at(body + 2), 2, "{}: stereo", path.display());
                assert_eq!(u16_at(body + 14), 32, "{}: 32 bits", path.display());
                rate = u32_at(body + 4);
            }
            b"data" => {
                let s = d[body..body + len]
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|&b| f32::from_le_bytes(b))
                    .collect();
                return (rate, s);
            }
            _ => {}
        }
        i = body + len + (len & 1);
    }
    panic!("{}: no data chunk", path.display())
}

/// What one REAPER render gave: the rendered files by name (without `.wav`).
struct Render {
    dir: PathBuf,
    files: BTreeMap<String, Vec<f32>>,
}

impl Render {
    fn get(&self, name: &str) -> &[f32] {
        self.files
            .get(name)
            .unwrap_or_else(|| panic!("{}: no {name}.wav", self.dir.display()))
    }
}

/// The pids of the REAPERs `ps` lists (`ps -axww -o pid=,command=`) that run with `ini` as
/// their `-cfgfile` and `-renderproject` after it. The match is the exact argument pair, and
/// the process must be an app's executable (`.../Contents/MacOS/...`), so neither `open` (whose
/// command line carries the same arguments) nor a REAPER on another configuration, the user's
/// included, can match.
fn reapers_of(ps: &str, ini: &Path) -> Vec<u32> {
    let pair = format!(" -cfgfile {} -renderproject ", ini.display());
    ps.lines()
        .filter_map(|l| {
            let (pid, cmd) = l.trim_start().split_once(' ')?;
            // what precedes the pair: the executable path (it may contain " -") and the
            // arguments before `-cfgfile`
            let before = &cmd[..cmd.find(&pair)?];
            before
                .contains(".app/Contents/MacOS/")
                .then(|| pid.parse().ok())
                .flatten()
        })
        .collect()
}

/// The pids of the REAPERs this test started on `ini` (see [`reapers_of`]); none when `ps`
/// cannot run (this is also called while unwinding, so it must not panic).
fn find_reapers(ini: &Path) -> Vec<u32> {
    Command::new("ps")
        .args(["-axww", "-o", "pid=,command="])
        .output()
        .map(|o| reapers_of(&String::from_utf8_lossy(&o.stdout), ini))
        .unwrap_or_default()
}

/// Everything one hidden run starts, stopped when it goes out of scope, on every exit path
/// including a panic: the hider, `open`, and the REAPERs found by the scratch `ini` (REAPER is
/// started by LaunchServices, not by this process, so it would outlive both). A Ctrl-C kills
/// the test process without running this; see docs/plugin.md 7.1 for the cleanup by hand.
struct Started<'a> {
    ini: &'a Path,
    hider: std::process::Child,
    open: Option<std::process::Child>,
    /// `open` was killed before REAPER may have been started: look again after a while.
    settle: bool,
}

impl Started<'_> {
    fn kill_reapers(&self) {
        for pid in find_reapers(self.ini) {
            let _ = Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status();
        }
    }
}

impl Drop for Started<'_> {
    fn drop(&mut self) {
        if let Some(open) = &mut self.open {
            let _ = open.kill();
            let _ = open.wait();
        }
        self.kill_reapers();
        if self.settle {
            // LaunchServices may still start the REAPER `open` asked for
            for _ in 0..3 {
                std::thread::sleep(Duration::from_millis(300));
                self.kill_reapers();
            }
        }
        let _ = self.hider.kill();
        let _ = self.hider.wait();
    }
}

/// The hider: a JXA script that waits for a pid on its standard input (started before REAPER,
/// so that its start-up, 0.3 s, is not spent after the first window) and then keeps that app
/// hidden until it is gone: `open -j` starts REAPER hidden, but REAPER unhides itself when it
/// opens its windows, so the loop (5 ms) hides it again whenever it is not hidden.
const HIDER: &str = "\
ObjC.import('AppKit');
const text = $.NSString.alloc.initWithDataEncoding(
  $.NSFileHandle.fileHandleWithStandardInput.readDataToEndOfFile, $.NSUTF8StringEncoding);
const pid = parseInt(ObjC.unwrap(text));
// The lookup is nil until the process is known to NSWorkspace: keep looping then; the end is
// isTerminated, or nil again after it was seen (a gone process), or 60 s of nil.
// (a nil lookup is an object whose properties are undefined).
let seen = false;
const t0 = Date.now();
for (;;) {
  const a = $.NSRunningApplication.runningApplicationWithProcessIdentifier(pid);
  const nil = a.isTerminated === undefined;
  if (!nil && a.isTerminated) break;
  if (nil && (seen || Date.now() - t0 > 60000)) break;
  if (!nil) {
    seen = true;
    if (!a.isHidden) a.hide;
  }
  $.NSThread.sleepForTimeInterval(0.005);
}
";

/// Starts the hider ([`HIDER`]); the pid goes to its standard input.
fn start_hider() -> std::process::Child {
    Command::new("osascript")
        .args(["-l", "JavaScript", "-e", HIDER])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("osascript runs")
}

/// Runs REAPER on macOS through `open`, so that it neither shows a window nor takes the focus:
/// `-n` a new instance, `-g` not activated, `-j` hidden, `-W` waits for it to exit; `--env`
/// because `open` does not pass this process's environment; [`HIDER`] keeps its windows
/// hidden. `open` exits 0 whatever REAPER's status, so the caller checks the render. A REAPER
/// that does not finish in `timeout` is killed, found by its scratch `ini` ([`find_reapers`]):
/// the child here is `open`, and no other REAPER is touched.
fn run_hidden(
    exe: &Path,
    ini: &Path,
    rpp: &Path,
    log: &Path,
    timeout: Duration,
) -> Result<(), String> {
    let app = exe
        .ancestors()
        .nth(3)
        .filter(|a| a.extension().is_some_and(|e| e == "app"))
        .ok_or_else(|| format!("{} is not inside a .app bundle", exe.display()))?;
    let mut st = Started {
        ini,
        hider: start_hider(),
        open: None,
        settle: false,
    };
    // let it get to its read before REAPER starts
    std::thread::sleep(Duration::from_millis(500));
    let mut cmd = Command::new("open");
    cmd.args(["-n", "-g", "-j", "-W", "-a"]).arg(app);
    // `open` does not pass this process's environment: the plugin's variables go by `--env`
    cmd.arg("--env").arg(format!("{LOG_VAR}={}", log.display()));
    for var in [LEVEL_VAR, orb_plugin::editor::SIZE_LOG_VAR] {
        if let Some(v) = std::env::var_os(var) {
            let mut kv = std::ffi::OsString::from(format!("{var}="));
            kv.push(v);
            cmd.arg("--env").arg(kv);
        }
    }
    cmd.args([
        "--args",
        "-newinst",
        "-nosplash",
        "-ignoreerrors",
        "-cfgfile",
    ])
    .arg(ini)
    .arg("-renderproject")
    .arg(rpp)
    .stdout(Stdio::null())
    .stderr(Stdio::null());
    st.open = Some(cmd.spawn().map_err(|e| format!("open: {e}"))?);
    let t0 = Instant::now();
    let mut found = false;
    loop {
        match st.open.as_mut().unwrap().try_wait() {
            Ok(Some(s)) if s.success() => return Ok(()),
            Ok(Some(s)) => return Err(format!("open {s}")),
            Ok(None) => {}
            Err(e) => return Err(format!("waiting for open: {e}")),
        }
        if !found && let Some(&pid) = find_reapers(ini).first() {
            // closing the pipe ends the pid for the hider
            let mut stdin = st.hider.stdin.take().unwrap();
            let _ = write!(stdin, "{pid}");
            found = true;
        }
        if t0.elapsed() > timeout {
            // `Started` kills `open` and the REAPER, looking again for a late one
            st.settle = true;
            return Err(format!("REAPER did not finish in {timeout:?} (a dialog?)"));
        }
        // fast until REAPER is found, so that the hider has it before its first window
        std::thread::sleep(Duration::from_millis(if found { 100 } else { 10 }));
    }
}

/// [`reapers_of`] matches only the REAPER on the given scratch configuration.
#[test]
fn only_the_reaper_of_the_scratch_configuration_matches() {
    let ini = Path::new("/w/target/reaper/a.vst3/resources/reaper.ini");
    let ps = "\
  101 /Applications/REAPER.app/Contents/MacOS/REAPER
  102 /Applications/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash -ignoreerrors \\
 -cfgfile /w/target/reaper/b.vst3/resources/reaper.ini -renderproject /w/b.RPP
  103 open -n -g -j -W -a /Applications/REAPER.app --args -newinst -nosplash -ignoreerrors \\
 -cfgfile /w/target/reaper/a.vst3/resources/reaper.ini -renderproject /w/a.RPP
  104 /Applications/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash -ignoreerrors \\
 -cfgfile /x/w/target/reaper/a.vst3/resources/reaper.ini -renderproject /w/a.RPP
  105 /Applications/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash -ignoreerrors \\
 -cfgfile /w/target/reaper/a.vst3/resources/reaper.ini -renderproject /w/a.RPP
  106 /Apps/My -x/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash -ignoreerrors \\
 -cfgfile /w/target/reaper/a.vst3/resources/reaper.ini -renderproject /w/a.RPP
";
    // the two-line entries above are one line each in the real `ps` output
    let ps = ps.replace("\\\n", "");
    assert_eq!(reapers_of(&ps, ini), [105, 106]);
}

/// Renders the fixture of `c` for `f` with REAPER (`exe`) on a scratch resource folder,
/// `target/reaper/<case>.<format>/`, and checks that it ran the fresh bundle.
fn render(exe: &Path, bundles: &Path, k: &Keys, c: &Case, f: Format) -> Render {
    let id = format!("{}.{}", c.name, f.ext());
    let dir = target_dir().join("reaper").join(&id);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("resources")).unwrap();
    std::fs::create_dir_all(dir.join("project")).unwrap();
    let dir = dir.canonicalize().unwrap();
    let (res, proj) = (dir.join("resources"), dir.join("project"));
    let ini = res.join("reaper.ini");
    std::fs::write(&ini, reaper_ini(bundles, k)).unwrap();
    let rpp = proj.join(format!("{}.RPP", c.name));
    std::fs::copy(cases::fixture_path(c, f), &rpp).unwrap();
    let log = dir.join("plugin.log");
    let t0 = Instant::now();
    if let Err(e) = run_hidden(exe, &ini, &rpp, &log, RENDER_TIMEOUT) {
        panic!("{id}: {e}; {}", dir.display());
    }
    let out = proj.join("out");
    assert!(
        out.join("Master.wav").is_file(),
        "{id}: REAPER exited without rendering (no {}; `open` reports 0 whatever REAPER's status)",
        out.join("Master.wav").display()
    );
    eprintln!("{id}: rendered in {:.1} s", t0.elapsed().as_secs_f64());
    // the copy that ran: every activation is the fresh bundle's library, at the project's rate
    let lib = bundles
        .join(format!("OpenRB-338.{}", f.ext()))
        .join("Contents/MacOS/OpenRB-338");
    let text = std::fs::read_to_string(&log).unwrap_or_default();
    let activations: Vec<(&str, u32)> = text.lines().filter_map(parse_loaded_from).collect();
    assert!(
        !activations.is_empty(),
        "{id}: the plugin was never activated (no activation line in {})",
        log.display()
    );
    for (path, rate) in activations {
        assert_eq!(
            Path::new(path).canonicalize().ok().as_deref(),
            Some(lib.as_path()),
            "{id}: REAPER ran another copy of the plugin: {path}"
        );
        assert_eq!(rate, c.rate, "{id}: the activation's rate");
    }
    // REAPER kept the scratch configuration's path, and its scan found one OpenRB-338 (the cache
    // names the file, not its folder)
    let kept = std::fs::read_to_string(&ini).unwrap();
    let key = k.vst_path;
    let line = kept.lines().find(|l| l.starts_with(&format!("{key}=")));
    let want = format!("{key}={}", bundles.display());
    assert_eq!(line, Some(want.as_str()), "{}", ini.display());
    // an entry: a VST cache line `<file>=<time>,<id>,<name>`
    let (cache, entry) = (k.vst_cache, "OpenRB_338.vst3=");
    let text = std::fs::read_to_string(res.join(cache)).unwrap_or_default();
    let found = text
        .lines()
        .filter(|l| l.starts_with("OpenRB") || l.starts_with("[OpenRB"))
        .count();
    assert!(
        found == 1 && text.contains(entry),
        "{id}: {cache} lists {found} OpenRB-338 entries"
    );
    let mut files = BTreeMap::new();
    for e in std::fs::read_dir(proj.join("out")).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_stem().unwrap().to_string_lossy().into_owned();
        let (rate, s) = read_wav(&p);
        assert_eq!(rate, c.rate, "{}", p.display());
        assert_eq!(
            s.len(),
            2 * (c.seconds * c.rate as f64).round() as usize,
            "{}: the rendered length",
            p.display()
        );
        files.insert(name, s);
    }
    Render { dir, files }
}

/// The in-process host's call size, and another one the output must not depend on (at 48 kHz
/// and 125 BPM these two give different renders, see `cases::TEMPO`).
const CALL: usize = 64;
const OTHER_CALL: usize = 1024;

/// The host rolling from beat 0 at the case's tempo: its transport at frame `f`.
fn rolling(c: &Case, f: usize) -> HostTransport {
    HostTransport {
        playing: true,
        tempo: Some(c.tempo),
        pos_beats: Some(f as f64 * c.tempo / 60.0 / c.rate as f64),
    }
}

/// What the in-process host got: the Mix and the four buses, interleaved stereo each, and the
/// latency the plugin reported.
struct Played {
    mix: Vec<f32>,
    buses: Vec<Vec<f32>>,
    latency: usize,
}

/// The in-process host as REAPER drives the plugin: `p` activated at the case's rate, rolling
/// from beat 0 ([`rolling`]) in calls of `call` frames, with the MIDI messages `ev` (frame,
/// message), for `frames` frames after the latency. The outputs start at the latency, where
/// REAPER's render starts (it compensates the reported latency).
fn play(c: &Case, p: &OrbParams, call: usize, frames: usize, ev: &[(usize, Message)]) -> Played {
    let rate = c.rate;
    let mut core = instance::core(p, rate);
    let latency = core.latency() as usize;
    let total = frames + latency;
    let values = p.values();
    let (mut l, mut r) = (vec![0f32; total], vec![0f32; total]);
    let mut buses = vec![vec![0f32; total]; 2 * DEVICE_OUTPUTS];
    let mut pos = 0;
    while pos < total {
        let end = (pos + call).min(total);
        let events: Vec<HostEvent> = ev
            .iter()
            .filter(|e| e.0 >= pos && e.0 < end)
            .map(|e| HostEvent {
                frame: (e.0 - pos) as u32,
                message: e.1,
            })
            .collect();
        let mut it = buses.iter_mut();
        let mut aux: AuxOut = std::array::from_fn(|_| {
            let (bl, br) = (it.next().unwrap(), it.next().unwrap());
            Some([&mut bl[pos..end], &mut br[pos..end]])
        });
        core.process_aux(
            &mut l[pos..end],
            &mut r[pos..end],
            &mut aux,
            &values,
            &rolling(c, pos),
            &events,
        );
        pos = end;
    }
    let inter = |a: &[f32], b: &[f32]| {
        a[latency..]
            .iter()
            .zip(&b[latency..])
            .flat_map(|(&x, &y)| [x, y])
            .collect()
    };
    Played {
        mix: inter(&l, &r),
        buses: buses.chunks(2).map(|p| inter(&p[0], &p[1])).collect(),
        latency,
    }
}

/// The first frame where `a` and `b` differ by more than `tol`, the count of such frames and
/// the largest difference.
fn diff(a: &[f32], b: &[f32], tol: f32) -> (Option<usize>, usize, f32) {
    assert_eq!(a.len(), b.len());
    let mut first = None;
    let mut count = 0;
    let mut max = 0f32;
    for i in (0..a.len()).step_by(2) {
        let d = (a[i] - b[i]).abs().max((a[i + 1] - b[i + 1]).abs());
        max = max.max(d);
        if d > tol {
            count += 1;
            first.get_or_insert(i / 2);
        }
    }
    (first, count, max)
}

/// What a comparison lets differ from bit-identical; only the device outputs' tracks need it.
#[derive(Clone, Copy)]
struct Allow {
    /// REAPER's 0 for a sample of the plugin's below [`FLUSHED`] (REAPER 7.86's VST3 host).
    flush: bool,
}

/// Bit for bit.
const EXACT: Allow = Allow { flush: false };

/// Asserts `got` (REAPER's) is `want` (the in-process host's) bit for bit, except what `allow`
/// lets differ.
fn same(what: &str, got: &[f32], want: &[f32], allow: Allow) {
    assert_eq!(got.len(), want.len(), "{what}: the length");
    let bad = got.iter().zip(want).position(|(&g, &w)| {
        g.to_bits() != w.to_bits() && !(allow.flush && g.to_bits() == 0 && w.abs() < FLUSHED)
    });
    if let Some(i) = bad {
        panic!(
            "{what}: first difference at frame {} ({} against {}); {:?}",
            i / 2,
            got[i],
            want[i],
            diff(got, want, 0.0)
        );
    }
}

fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |m, v| m.max(v.abs()))
}

/// Checks one render against the in-process host: the same samples, then the case's own
/// comparison.
fn check(c: &Case, f: Format, r: &Render) {
    let id = format!("{}.{}", c.name, f.ext());
    let frames = (c.seconds * c.rate as f64).round() as usize;
    let p = cases::instance(c);
    let (ev, end) = cases::notes(c, &p);
    let host = play(c, &p, CALL, frames, &ev);
    // the in-process render does not depend on the host's call size, so neither does the
    // comparison on how REAPER splits its calls ([`cases::TEMPO`])
    let other = play(c, &p, OTHER_CALL, frames, &ev);
    same(
        &format!("{id} in-process calls"),
        &other.mix,
        &host.mix,
        EXACT,
    );
    let master = r.get("Master");
    assert!(peak(master) > 0.05, "{id}: the render plays");
    // REAPER drives the plugin as the in-process host: the state, the activation, the MIDI at
    // its frames, the transport, the latency compensation
    same(&format!("{id} Master"), master, &host.mix, EXACT);
    eprintln!(
        "{id}: equal to the in-process host (latency {})",
        host.latency
    );
    if c.orphan.is_some() {
        // task-258: the orphan note holds the gate open (every clip note slides) until the
        // clip lets its key go; after that the 303 closes the gate at each note-off, so the
        // item's tail is silent once the last note's release is over (before the fix the
        // orphan held the gate to the end of the render)
        let clips = end - (cases::TAIL * c.rate as f64).round() as usize;
        let from = clips + c.rate as usize / 2;
        let tail = peak(&master[2 * from..2 * end]);
        eprintln!("{id}: the tail's peak, frames {from}..{end}: {tail}");
        assert!(
            tail < 1e-3,
            "{id}: the 303 still sounds after the clips ({tail})"
        );
        let during = peak(&master[..2 * clips]);
        assert!(during > 0.05, "{id}: the clips play ({during})");
    } else if let Some((_, timing)) = c.clip {
        // orb-player's pattern-mode render through the same process side, the host rolling
        // from beat 0; compared up to the block where the pattern starts its third loop, which
        // the clip does not have
        let pattern = cases::instance(c);
        pattern.set_mode(Mode::Latch);
        let pat = play(c, &pattern, CALL, frames, &[]);
        let ticks = (end as u64 * 192 / cases::ppq(c) as u64) as u32;
        let until = cases::pattern_frame(c, ticks);
        let d = diff(&master[..2 * until], &pat.mix[..2 * until], LSB);
        eprintln!("{id}: against the pattern, frames 0..{until}: {d:?}");
        let tol = match timing {
            Timing::Clip => CLIP_CEILING.iter().find(|g| g.0 == c.name).unwrap().1,
            Timing::Aligned => LSB,
        };
        assert!(
            d.2 <= tol,
            "{id}: against the pattern, frames 0..{until}: {d:?}, tolerance {tol}"
        );
    }
    if c.routed {
        for (k, (name, _)) in DEVICE_TRACKS.iter().enumerate() {
            let stem = r.get(name);
            assert!(peak(stem) > 0.1, "{id}: the {name} track plays");
            // the VST3 host flushes tiny samples ([`Allow`])
            let allow = Allow {
                flush: f == Format::Vst3,
            };
            same(&format!("{id} {name}"), stem, &host.buses[k], allow);
        }
        eprintln!("{id}: each device track equal to its bus");
    }
}

#[test]
fn reaper_renders_as_the_plugin_plays_in_process() {
    let Some(exe) = std::env::var_os("ORB_REAPER").filter(|v| !v.is_empty()) else {
        eprintln!("skipped: set ORB_REAPER to the REAPER executable to run the REAPER renders");
        return;
    };
    let Some(k) = keys() else {
        eprintln!("skipped: the REAPER render test knows REAPER's settings on macOS only");
        return;
    };
    let exe = PathBuf::from(exe);
    let bundles = bundles();
    let mut masters = BTreeMap::new();
    for c in &CASES {
        for f in Format::ALL {
            let r = render(&exe, &bundles, &k, c, f);
            check(c, f, &r);
            masters.insert((c.name, f.ext()), r.get("Master").to_vec());
        }
    }
    // the Mix does not change when the device outputs are routed (task-256)
    for f in Format::ALL {
        same(
            &format!("outputs_routed.{} Master against unrouted", f.ext()),
            &masters[&("outputs_routed", f.ext())],
            &masters[&("outputs_unrouted", f.ext())],
            EXACT,
        );
    }
    // a limitation pin, not a matching case: REAPER 7.86's command-line render ignores the
    // looped selection and renders the linear timeline, bit-identical to the unlooped project.
    // This fails once REAPER loops; then a looped host belongs here (task-264)
    for f in Format::ALL {
        same(
            &format!(
                "loop_ignored.{} against the unlooped project (does REAPER loop now? task-264)",
                f.ext()
            ),
            &masters[&("loop_ignored", f.ext())][..2 * 4 * 44_100],
            &masters[&("outputs_unrouted", f.ext())],
            EXACT,
        );
    }
}

/// The timeout path: a REAPER that does not finish (an unreadable project leaves it waiting)
/// fails the run, and the REAPER it started, found by its scratch configuration, is gone.
#[test]
fn a_hung_reaper_is_killed_and_fails_the_run() {
    let Some(exe) = std::env::var_os("ORB_REAPER").filter(|v| !v.is_empty()) else {
        eprintln!("skipped: set ORB_REAPER to the REAPER executable to run the REAPER renders");
        return;
    };
    let Some(k) = keys() else {
        eprintln!("skipped: the REAPER render test knows REAPER's settings on macOS only");
        return;
    };
    let dir = target_dir().join("reaper/timeout");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("resources")).unwrap();
    let dir = dir.canonicalize().unwrap();
    let ini = dir.join("resources/reaper.ini");
    std::fs::write(&ini, reaper_ini(&dir, &k)).unwrap();
    let rpp = dir.join("garbage.RPP");
    std::fs::write(&rpp, "not a project\n").unwrap();
    let t0 = Instant::now();
    let err = run_hidden(
        Path::new(&exe),
        &ini,
        &rpp,
        &dir.join("plugin.log"),
        Duration::from_secs(15),
    )
    .expect_err("the garbage project does not render");
    eprintln!(
        "hung REAPER: {err} after {:.1} s",
        t0.elapsed().as_secs_f64()
    );
    assert!(err.contains("did not finish"), "{err}");
    assert!(!dir.join("out").exists(), "nothing rendered");
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        find_reapers(&ini).is_empty(),
        "the REAPER of {} is still running",
        ini.display()
    );
}
