---
id: TASK-277
title: 'REAPER test: run REAPER hidden, without windows or focus changes'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T07:26:00Z'
updated_date: '2026-11-22T09:39:00Z'
labels:
  - tests
milestone: m-7
dependencies:
  - TASK-275
references:
  - crates/orb-plugin/tests/reaper.rs
  - docs/plugin.md
priority: medium
type: enhancement
ordinal: 259750
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the opt-in REAPER render test (`ORB_REAPER`, crates/orb-plugin/tests/reaper.rs, docs/plugin.md 7.1) starts REAPER's executable directly with `-newinst -nosplash -ignoreerrors -cfgfile <scratch ini> -renderproject <rpp>`. Each of its 22 renders shows REAPER's main window and render dialog, and they can take the focus, while the user works on the same Mac. The user asked (2026-11-22) that the test run without REAPER's UI appearing.

REAPER has no headless mode. On macOS, launching through `open` should keep it out of sight:
- `open -n -g -j -W -a <REAPER.app or exe> --env ORB_PLUGIN_LOG=<log> --args -newinst -nosplash -ignoreerrors -cfgfile ... -renderproject ...`;
- `-n` starts a new instance, `-g` launches in the background without activating, `-j` launches hidden (as Cmd+H), and `-W` waits for exit;
- `--env` is needed because `open` does not pass the test's environment, and the plugin log is how the test checks which library ran.
This is not verified yet: check that the render dialog and any window REAPER opens stay hidden, and that the render result is unchanged.

**What has to keep working:**
- **The timeout path.** Today the test kills its own child on RENDER_TIMEOUT. With `open`, the child is `open`, not REAPER, so the test must find the REAPER it started, by its unique scratch `-cfgfile` path in the process list, and kill only that process.
  - Never touch any other REAPER: the user may have one open. Kill only the process this test started.
  - Test this path.
- **The rest unchanged:**
  - the exit status (REAPER's, not open's: check what `open -W` returns when REAPER fails, and keep a failed render failing);
  - the plugin log check;
  - the scratch configuration;
  - the `-newinst` isolation.
- **If `-j` does not hide a window,** document what shows. Try the fallback: the window positions in the scratch reaper.ini moved off-screen.
- **Windows:** if the test is made to run there, start REAPER hidden (STARTUPINFO wShowWindow = SW_HIDE). Otherwise note in docs/plugin.md 7.1 that only macOS is covered.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 REAPER never takes the focus from the user's front app, and on macOS the REAPER test's windows are hidden except for brief flashes (best effort; measured over 12 runs of the whole test with a 20 ms sampler: REAPER never frontmost; no REAPER window on screen in 8 runs, windows in 14, 58, 59 and 131 samples in the other 4, the longest about 3 s in one render); the method is documented in docs/plugin.md 7.1 and the reaper.rs header
- [x] #2 The renders and comparisons give the same results as before (all REAPER test cases pass with ORB_REAPER set), and the plugin log check still names the fresh bundle
- [x] #3 A failing or hanging render still fails the test: REAPER's failure is not hidden behind open's exit status, and on timeout the test kills only the REAPER it started (found by its scratch -cfgfile path); the timeout path is tested
- [x] #4 No REAPER the user has open is used, focused or disturbed; clippy and nightly fmt clean, cargo test --workspace passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Measure on REAPER 7.86 (open -n -g -j -W vs direct) with a CGWindowList/NSWorkspace sampler.
2. Run REAPER through open with a pid-fed JXA hider; find the REAPER by its scratch -cfgfile pair in ps for the kill path.
3. Unit test the ps matching, a real hang test, docs/plugin.md 7.1 and the reaper.rs header.
4. cargo test --workspace, clippy, nightly fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a22c37530b3e5692c

Measurements (REAPER 7.86, arm64, macOS 15.6; sampler = JXA loop every 20 ms of CGWindowListCopyWindowInfo on-screen windows owned by REAPER plus NSWorkspace front app; no REAPER of the user was open: pgrep -x REAPER empty before each run):
- direct start (old way), latch_48k: REAPER windows on screen in 52 of 155 samples (main window 1024x768 and the render dialog); REAPER never the front app.
- open -n -g -j -W, no hider: windows on screen in 15 of 117 samples. So -j does NOT keep the windows hidden (REAPER unhides itself; lsappinfo/NSRunningApplication isHidden flips false->true->false). -g works: front app stayed the user's.
- Fallback tried: reaper.ini wnd_x/wnd_y=-20000: macOS clamps (REAPER wrote back wnd_x=68, wnd_y=0), the render dialog is centered anyway. Not a fallback.
- Hider: NSRunningApplication.hide in a 5 ms JXA loop (no Accessibility needed; System Events fails with -25211 here), started before REAPER and fed the pid on stdin. Whole reaper test with it: 2250 samples in 54 s, 0 samples with a REAPER window on screen, 0 with REAPER frontmost.
- open -W exits 0 on success; with an unreadable project REAPER hangs (waits), so the timeout path catches it; a REAPER failure that exits is only visible as missing out/Master.wav, which the test now asserts explicitly.
- Timeout: test a_hung_reaper_is_killed_and_fails_the_run (garbage RPP, 15 s timeout): fails with 'did not finish', REAPER gone from ps afterwards. reapers_of unit test covers another REAPER, open's own command line and a path that only ends like the scratch path.
- Failed approach: per-iteration osascript hider (0.3 s start-up): windows seen in 31 of 292 samples.

Final: ORB_REAPER=... cargo test --workspace all ok (reaper.rs: 4 tests, 11 renders equal to in-process as before, activation log names the fresh bundle); clippy -D warnings and nightly fmt --check clean. No cfg(target_os) code added (the test skips off macOS), so no Windows-only lint; Windows is not covered (documented in docs/plugin.md 7.1).

Review round 1 (all fixed in reaper.rs, docs/plugin.md 7.1): Started Drop guard (hider, open, REAPERs by scratch -cfgfile, 3 re-scans over 1 s after a timeout); Ctrl-C not coverable, cleanup by scratch path documented; hider keeps looping on a nil lookup (nil is an object with undefined properties in JXA, not an exception) and ends on isTerminated, or nil after seen, or 60 s nil; ORB_PLUGIN_LOG_LEVEL and ORB_PLUGIN_SIZE_LOG passed by --env; reapers_of matches on the text before the exact pair (app path with ' -' unit-tested); render time measured (2.3 to 5 s each). Honest result: the hider is a race, not a guarantee. Over 12 runs REAPER was never frontmost; 8 runs had 0 REAPER window samples, 4 had 14, 58, 59 and 131 samples (one render 3 s). 4 later runs with hider logging: 0 windows each. Final runs: cargo test --workspace ok with ORB_REAPER, clippy and nightly fmt clean, no REAPER, open or hider of mine left.

User acceptance (2026-11-22): the original criterion 1 was 'On macOS the REAPER test runs all its renders with no REAPER window visible and without taking the focus from the user's front app; the method is documented in docs/plugin.md 7.1 and the reaper.rs header'. The hider is a race, so that is not guaranteed; the user accepted the result: REAPER never takes the focus, its windows are mostly hidden but can flash briefly. Criterion 1 was reworded to that, with the 12-run numbers.

Review round 2: (a) commit message rewritten to the 12-run numbers and the review-round-1 changes; (b) per-render times removed from docs (raw, one run, seconds, in test order: 3.0 2.3 2.4 3.0 2.4 2.3 5.0 2.4 2.4 5.0 2.4; whole test about 55 s); (c) Ctrl-C cleanup rewritten: scratch dir <target dir>/reaper/, fixed-string grep -F on ps, regex caveat of pkill -f; (d) several --env flags checked by a one-off open run with ORB_PLUGIN_LOG, ORB_PLUGIN_LOG_LEVEL, ORB_PLUGIN_SIZE_LOG, WGPU_BACKEND=metal and WGPU_POWER_PREF=low: the plugin log's first line (which prints the environment) shows 'WGPU_BACKEND metal, WGPU_POWER_PREF low' (unset otherwise) and the log itself was written, so all --env flags reach REAPER. ORB_PLUGIN_LOG_LEVEL and SIZE_LOG have no visible effect in a render without the editor (level only adds wgpu debug records, the size log only the editor's), so they were not observable themselves.

Raw run behind 'without the hider 953 of 2033 samples': the whole reaper test (all renders and the hang test) with the hider script made to exit at once (a temporary $.exit(0) in HIDER, since reverted), sampled by the 20 ms JXA sampler (mon.js): samples=2033 samples_with_reaper_windows_onscreen=953 max_windows=3 samples_with_reaper_frontmost=0 first_window_ms=7175 (test ok in 54.9 s). Other recorded no-hider figures: direct start 52 of 155, open -g -j alone 15 of 117 (single latch_48k renders).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
REAPER test runs through open -n -g -j -W plus a pid-fed JXA hider (NSRunningApplication.hide every 5 ms), since -j alone does not keep the windows hidden. Result (user accepted 2026-11-22): REAPER never takes the focus (never frontmost in 12 runs); windows are hidden except for brief flashes, best effort: 8 of 12 runs had no REAPER window on screen, 4 had windows in 14, 58, 59 and 131 samples (20 ms; the longest about 3 s in one render). Started Drop guard stops hider, open and the REAPER found by its scratch -cfgfile on every exit and re-scans after a timeout; Ctrl-C cleanup by scratch path is documented. open exits 0 whatever REAPER does, so the test asserts out/Master.wav. ORB_PLUGIN_LOG_LEVEL and ORB_PLUGIN_SIZE_LOG go through --env. Tests: cargo test --workspace with ORB_REAPER passes (11 renders equal to in-process, ps matching unit test, real hang/kill test); clippy and nightly fmt clean. Docs: docs/plugin.md 7.1, reaper.rs header. Branch worktree-agent-a22c37530b3e5692c, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
