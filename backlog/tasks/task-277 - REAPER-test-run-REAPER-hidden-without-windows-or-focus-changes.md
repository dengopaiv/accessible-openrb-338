---
id: TASK-277
title: 'REAPER test: run REAPER hidden, without windows or focus changes'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-22T07:26:00Z'
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
- [ ] #1 On macOS the REAPER test runs all its renders with no REAPER window visible and without taking the focus from the user's front app; the method is documented in docs/plugin.md 7.1 and the reaper.rs header
- [ ] #2 The renders and comparisons give the same results as before (all REAPER test cases pass with ORB_REAPER set), and the plugin log check still names the fresh bundle
- [ ] #3 A failing or hanging render still fails the test: REAPER's failure is not hidden behind open's exit status, and on timeout the test kills only the REAPER it started (found by its scratch -cfgfile path); the timeout path is tested
- [ ] #4 No REAPER the user has open is used, focused or disturbed; clippy and nightly fmt clean, cargo test --workspace passes
<!-- AC:END -->
