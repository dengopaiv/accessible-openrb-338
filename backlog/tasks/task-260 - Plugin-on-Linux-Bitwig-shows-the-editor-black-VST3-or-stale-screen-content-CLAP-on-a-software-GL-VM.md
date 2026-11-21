---
id: TASK-260
title: >-
  Plugin on Linux: Bitwig shows the editor black (VST3) or stale screen content
  (CLAP) on a software-GL VM
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T08:29:00Z'
updated_date: '2026-11-21T09:59:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies: []
references:
  - /Users/pingw33n/win_exchange/log/engine.log
  - /Users/pingw33n/win_exchange/log/BitwigStudio.log
  - docs/plugin.md
  - docs/building.md
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/main.rs
priority: medium
type: bug
ordinal: 253000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21, the first test of the plugin in Bitwig on Linux.

**Setup**
- Ubuntu in a VirtualBox VM (Mesa 26.0.8, no GPU), opened over RDP from Windows.
- An Xfce X11 session: `XDG_SESSION_TYPE=x11`.
- OpenGL is llvmpipe (software): `glxinfo -B` reports llvmpipe (LLVM 21.1.8), core 4.5, not accelerated. Vulkan has a software ICD; Bitwig itself uses Vulkan "llvmpipe" through Skia.
- The plugin is from the package OpenRB-338-2026.1118.0919-x86_64-unknown-linux-gnu. The user thinks later builds changed nothing relevant.

**Symptoms**
- **VST3:** the editor window shows the menu bar (File, Mods, Options, Help) and the mod's splash image, but the panel area is black.
- **CLAP:** the window shows whatever was on screen when it opened. Sometimes it shows only the menu and splash, as the VST3 does; it varies between openings.
- Moving the mouse, clicking or resizing doesn't make the panel appear.
- **The standalone OpenRB-338 works in the same VM and session,** with the default backend and with `WGPU_BACKEND=gl` alike. So drawing on wgpu's GL backend on llvmpipe works in eframe's window; the problem is specific to the plugin's window path (nice-plug-egui 0.6 → egui-baseview 0.9 → wgpu 30, an X11 child window that Bitwig's plugin host embeds).

**Evidence**
- Bitwig's logs are at /Users/pingw33n/win_exchange/log (engine.log, BitwigStudio.log).
- When the editor opens, the plugin host (BitwigPluginHost-X64-AVX2) logs `libEGL warning: DRI3 error: Could not get DRI3 device` and `Ensure your X server supports DRI3 to get accelerated rendering`. So the plugin's wgpu took the GL (EGL) backend, not Vulkan.
- egui-baseview's renderer (`egui-baseview-0.9.0/src/renderer/wgpu/renderer.rs`) builds its instance with `InstanceDescriptor::new_with_display_handle`. That uses all backends and does not read `WGPU_BACKEND` (wgpu reads the environment only via `with_env()`). The standalone chooses its backends itself: `crates/orb-ui/src/main.rs` `wgpu_backends`, docs/building.md.
- `NICE_LOG=$HOME/orb-plugin.log` set on bitwig-studio produced no file. Bitwig's plugin host apparently doesn't pass it through. Logging needs another route: a file the plugin opens itself, gated by an `ORB_*` variable or a settings flag, listed in docs/building.md's variable table.
- The black panel with a visible splash suggests that some draws reach the screen but most frames are not presented, or are presented to the wrong surface. The panel's SVG parts are placeholders rasterized at the end of a frame (`orb-ui` `part_texture`/`flush_part_textures`), so they need later frames to be presented. CLAP's stale content says no frame was presented at all in that case.

**What to do**
1. Add a plugin log that works inside a host. Through it, report the wgpu adapter (backend, name, device type) and the surface format and present mode, plus every wgpu error and failed surface acquire or present. Make it available in release builds.
2. Find the cause: why frames are not presented in an embedded baseview X11 window on llvmpipe while eframe's window works. Candidates:
   - EGL on a child window of a foreign parent without DRI3 (the software present path);
   - the surface configured before the window is mapped or given its size;
   - surface acquire errors (Outdated, Timeout) being swallowed;
   - the present mode.
   Check egui-baseview's and baseview's X11 code.
3. Fix it. Likely options:
   - choose the backends the way the standalone does: honor `WGPU_BACKEND`, and on Linux prefer Vulkan when an adapter exists. This may need a patched or forked egui-baseview, or nice-plug-egui's options if they expose the instance descriptor;
   - handle surface errors with reconfigure and retry;
   - delay the surface configuration until the window is mapped.
   If the fix needs vendoring or forking a dependency, or a choice of default backend, create a `Decide:` task for the user first.
4. The agent cannot run Bitwig or the VM itself. Give the user a test build: CI on a pushed branch, or `scripts/package.sh x86_64-unknown-linux-gnu` with Docker. Write the exact steps for the user's check, including the log variable. The user checks in Bitwig in the VM before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The plugin can write a log file inside a host (release build), enabled by a documented ORB_* variable or setting; it records the wgpu adapter, surface configuration and surface/present errors
- [x] #2 The cause of the unpresented frames in Bitwig on the user's software-GL Linux VM is found and written in the task notes with the log evidence
- [x] #3 The plugin's wgpu backend choice is documented in docs/plugin.md (and follows WGPU_BACKEND like the standalone, unless a Decide task chose otherwise)
- [x] #4 macOS and Windows editors still work: the existing plugin tests pass and the user checks the macOS plugin in one host
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. A plugin log (crates/orb-plugin/src/plugin_log.rs): on when ORB_PLUGIN_LOG names a file (or is `stderr`: Bitwig copies the plugin host's stderr into engine.log), or when the file plugin.log exists in orb-ui's log folder (~/.local/state/OpenRB-338 on Linux), so it works even if a host drops the environment. Release builds. It installs a `log` logger (wgpu-core/hal, egui-wgpu and egui-baseview log through `log`: adapters, acquire/present errors), chains a panic hook (wgpu's uncaptured errors panic), and writes the editor's own lines.
2. The adapter: egui-baseview's GraphicsConfig gets a WgpuSetup::CreateNew whose native_adapter_selector logs every adapter, the surface capabilities and the chosen format/present mode, and picks the adapter as wgpu-core 30's request_adapter does (surface-compatible, stable sort by device type for the power preference), among the WGPU_BACKEND backends when that is set. Default choice unchanged.
3. Frame diagnostics: a frame counter in the editor's ui() and a probe thread per open editor (log on only) that logs the frame count at 1, 2, 4, 8, 16 s and, on X11, sends PresentNotifyMSC requests on baseview's window from a connection of its own and logs each completion's msc/ust and delay: baseview 0.4.2 paces every draw on Present CompleteNotify (platform/x11/present.rs) and ignores a completion whose msc did not advance.
4. Hypothesis to confirm with the log: on the VM the X server's Present MSC does not advance (no vblank in the VM's DRM driver), so after the first frame baseview never draws again; the first frame shows the menu and splash, the panel's textures need later frames. The fix would patch baseview (a dependency): a Decide task for the user.
5. docs/plugin.md (the log, the backend choice), docs/building.md variable table; unit tests for the log switch and the adapter ranking; full cargo test, clippy, nightly fmt; a Linux x86_64 package for the user's check.

6. (Round 2, the user's decision of 2026-11-21) Rescoped to the log and the diagnosis: the baseview fix and the close-path check are task-262, after Decide task-261.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac59c7502f94651b6

Research (2026-11-21):
- Bitwig logs (/Users/pingw33n/win_exchange/log, read only): engine.log 108-130 is the CLAP run: the plugin host BitwigPluginHost-X64-AVX2 starts at 3:20:49.387, the two `libEGL warning: DRI3 ...` lines are its stderr (so Bitwig copies the plugin host stderr into engine.log), then the plug-in closes at 3:20:52.448. No panic or wgpu line. BitwigStudio.log: Bitwig itself draws with Vulkan llvmpipe (mailbox). BitwigStudio-previous-run.log 403-410 is the VST3 insert (no plug-in host stderr kept there).
- Why NICE_LOG produced no file: not the host. nice-plug is built with default-features = false (no tracing-subscriber) and Plugin::setup_logger is the default None, so nice-plug sets up no logger at all (nice-plug-0.4.2/src/wrapper/util.rs setup_logger); NICE_LOG is never read. Every nice_warn!/nice_error! of the plugin goes nowhere too. So there is no evidence Bitwig drops the environment; the new log has a switch file anyway.
- The DRI3 warning does not prove the GL backend: egui-baseview makes the instance with all backends (renderer.rs 57), and wgpu initializes EGL for its GL backend at instance creation. egui-wgpu then asks wgpu request_adapter with HighPerformance (setup.rs 255); wgpu-core 30 instance.rs 570-600 sorts by device type, stable, so on the VM (Vulkan llvmpipe and GL llvmpipe, both Cpu) Vulkan comes first if it can present to the window. Which one was taken is what the new log shows.
- Leading hypothesis (H1): baseview 0.4.2 X11 paces every draw on X Present. present.rs: draw only when draw_now (72), set only by a CompleteNotify of the last serial whose MSC is past the last received one (134-158; an MSC that did not advance is recorded and draws nothing, 150-153); the next PresentNotifyMSC (last MSC + 2) is sent only after the previous one was answered (192-226); the 15 ms fallback timer (160) only without the extension or when the request fails. egui-baseview requests redraws only through baseview (window.rs 520, 938). With an X server whose MSC does not advance for the window (no vblank in the VM display driver: Xorg Present then completes at once with the same MSC) or that never answers, the editor draws one frame and stops for good, and input cannot restart it. That matches: VST3 first frame (menu + splash; the panel parts are rasterized at the end of a frame and need later frames) then black; CLAP first frame drawn before mapping/reparenting or never (stale content); nothing helps. The standalone (winit) has no such pacing, which is why it works with Vulkan and GL alike.
- Other hypotheses kept open (the log tells): H2 frames drawn but not presented (egui-baseview renderer.rs 207-232 drops Timeout/Occluded/Outdated/Lost silently; the log then shows frames counting up with nothing on screen); H3 a wgpu validation panic in the window thread (the log has a chained panic hook).
- No newer baseview, egui-baseview or nice-plug-egui on crates.io (cargo search, 2026-11-21: 0.4.2, 0.9.0, 0.6.0). The plugin cannot reach baseview pacing from outside (draw_now private; Present events cannot be sent with SendEvent; a NotifyMSC from another client does reach baseview but with the same non-advancing MSC). A fix patches baseview: Decide task-261 (options A-D, recommendation A: vendored baseview with a fallback to the 15 ms timer when the MSC does not advance, as raw-window-metal is vendored).

Implementation:
- crates/orb-plugin/src/plugin_log.rs: the log (ORB_PLUGIN_LOG=FILE or stderr; else the switch file plugin.log in orb_ui::crash::log_dir, i.e. ~/.local/state/OpenRB-338/plugin.log on Linux), a `log` logger (debug for wgpu*, egui_wgpu, egui_baseview, orb_plugin; info otherwise; 20000 lines max), a chained panic hook, the header line; graphics_config(): egui-wgpu WgpuSetup::CreateNew with backends = WGPU_BACKEND or all and a native_adapter_selector that logs every adapter (surface support), picks as wgpu-core 30 request_adapter (pick(), unit-tested) and logs the surface formats/present modes/alpha modes and egui format and present mode; Probe: frames (first five, gaps over 1 s), counts at 1/2/4/8/16/32 s, and on X11 the Present probe (own x11rb connection, select COMPLETE_NOTIFY on baseview window, six NotifyMSC serial 0x4f524200+n, target 0 then last+2, logs MSC/UST/kind/mode/delay or no answer in 2 s).
- editor.rs: spawn calls plugin_log::init and passes the graphics config; EditorApp.probe built in build(), counted in ui(), dropped in editor_closed.
- Cargo.toml: egui-wgpu 0.37 (default-features off), log 0.4; x11rb features dri3 + present (x11rb 0.13.2 present does not compile without dri3: checked in a scratch crate; baseview enables both on Linux).
- macOS check: ORB_PLUGIN_LOG=<scratch>/mac-plugin.log cargo test -p orb-plugin --test clap_gui: clap_gui ok; the log has the header, the Metal adapter (presents: true), surface formats [Bgra8UnormSrgb, Bgra8Unorm, Rgba16Float, Rgb10a2Unorm], present modes [Fifo, Immediate], egui takes Bgra8Unorm AutoVsync, wgpu_hal build swapchain lines, frames 1-5 and closed after 57 frames.
- The Linux x11 module was compile-checked with clippy -D warnings in a scratch crate (x11rb 0.13.2, dri3+present) on this Mac: no Linux sysroot here and Docker is not running, so the full Linux build is left to CI.

Checks (2026-11-21, round 1): cargo test 973 pass, 0 fail (98 test binaries); cargo clippy --all-targets -D warnings clean (macOS); cargo +nightly fmt --all --check clean. Docker is not running on this Mac, so no Linux package here: the supervisor gets a CI build of the branch for the user.

AC 1 evidence: the macOS clap_gui run above (the log in a test build; nothing in plugin_log.rs depends on debug_assertions). Limit, written in docs/plugin.md: egui-baseview drops Timeout/Occluded/Outdated/Suboptimal/Lost acquires without logging (renderer.rs 207-232); wgpu-core acquire/present errors and validation panics are logged.
AC 4 evidence: docs/plugin.md section 10 "The wgpu adapter (task-260)"; WGPU_BACKEND restricts the enumerated adapters (graphics_config), default unchanged (all backends, wgpu order); unit test the_adapter_is_wgpus_pick.
Open: AC 2 needs the user's log from the VM; AC 3 needs the fix, which waits on Decide task-261 if the log confirms H1; AC 5 needs the user's macOS host check. Task stays To Do.

The user's check in the VM (round 1, diagnosis):
1. Unpack the CI package OpenRB-338-*-x86_64-unknown-linux-gnu.tar.gz in place of the 2026.1118.0919 one (same plugins folder for Bitwig).
2. mkdir -p ~/.local/state/OpenRB-338 && touch ~/.local/state/OpenRB-338/plugin.log (the switch file; no environment needed).
3. Start Bitwig, insert the CLAP, open its editor, leave it open 40 s, move the mouse over it, close it. Then the same with the VST3.
4. Optional second run: start Bitwig from a terminal as WGPU_BACKEND=gl bitwig-studio (and once with vulkan) and repeat 3; the log header shows whether the variable reached the plugin.
5. Copy ~/.local/state/OpenRB-338/plugin.log and ~/.BitwigStudio/log/engine.log to win_exchange/log, plus the output of: uname -r; lspci | grep -i vga; xdpyinfo | grep -i -E "present|dri"; grep -i -E "driver|present|vblank|glamor" /var/log/Xorg.0.log ~/.local/share/xorg/Xorg.*.log; ps ax | grep -i -E "xrdp|Xorg|vrde".
6. rm ~/.local/state/OpenRB-338/plugin.log to turn the log off.
Reading it: "present probe: ... (the MSC did not advance ...)" or "no CompleteNotify" plus frame counts stuck at 1-2 confirms H1 (then task-261); frames counting up with a black window points to H2 (presentation); a "panic:" line to H3.

Round 2 (2026-11-21): the user ran build 2026.1121.0857 (dev-44c615c) in the VM; the log confirms H1. Files: /Users/pingw33n/win_exchange/log plugin.log, engine.log, info.txt.
- System (info.txt): kernel 7.0.0-30-generic; VMware SVGA II Adapter; Xorg on vt2 under gdm (not xrdp), modesetting; "Refusing to try glamor on llvmpipe", "glamor initialization failed"; "Initializing extension Present"; xdpyinfo lists DRI2 and Present.
- Adapter (plugin.log 320-326): adapters Vulkan llvmpipe (Cpu) and GL llvmpipe (Cpu), both present to the window; the editor takes Vulkan llvmpipe; WGPU_BACKEND unset, HighPerformance. Surface formats [Bgra8UnormSrgb, Bgra8Unorm], present modes [Immediate, Mailbox, Fifo, FifoRelaxed]; egui takes Bgra8Unorm, AutoVsync. So the GL premise of the task description was wrong: the DRI3 warning came from instance creation.
- Present probe (VST3, plugin.log 331-338): "Present 1.2; window 0x3800003, 656x532 at (391, 124) on the root, depth 32"; NotifyMSC 0 (target 0) and 1-5 (target 2) all complete in 2-6 ms with MSC 0, UST 0 us, kind NOTIFY_MSC, mode COPY: the MSC never advances (UST 0: the server has no time for the window CRTC either). CLAP (plugin.log ~676-690): the same, 0.0 ms each.
- Frames: frame 1 at 121 ms; the count is 1 at 1, 2, 4, 8, 16 s; "editor: closed after 1 frames"; VST3 (parent None, wait for parent true) and CLAP (parent Some(XlibWindow(58720256))) alike.
- Cause (criterion 2): baseview 0.4.2 draws only on a CompleteNotify whose MSC is past the last one (present.rs 134-158); the server answers every request with MSC 0, so after the first frame baseview never draws again, and input only asks for another notification that draws nothing. H2 and H3 refuted: no frames beyond the first were drawn, no panic line.
- On closing (plugin.log 343-346): WARN baseview::platform::x11::window_thread "Error while closing window: Request to X11 thread failed: Could not receive response (X11 thread disconnected)", then BadWindow (58720259, the editor window) for Present SelectInput (major 148, minor 3) and core DestroyWindow (major 4). Possibly related to another user's report (Debian 13, Bitwig 6.1): "audio engine crash when I close the CLAP version", every time the editor is closed, not minimized. Left to task-262.
- The user's decision (2026-11-21): no fix in this task. Follow-up task-262 (bug, player/ui, m-7, @agent-complex, depends on Decide task-261) holds the baseview fix and the close-path check. Criterion 3 ("The fix makes the VST3 and CLAP editors draw the full panel in Bitwig on the user's VM, confirmed by the user") was moved to task-262 by the user on 2026-11-21 and removed here; task-260 no longer depends on task-261. Task-261 stays To Do, undecided, its description updated with the evidence.
- docs/plugin.md section 10: the confirmed cause with this evidence.
- Criterion 5 (after renumbering: the macOS/Windows criterion) waits for the user's macOS check in one host; the task stays To Do.

Round 3 (2026-11-21): the user checked the dcb6a1d bundles on macOS (built in this worktree with cargo xtask bundle orb-plugin --release, installed in /Library): "plugin seems to work": the editor opens, draws and plays. The user had created ~/Library/Application Support/OpenRB-338/plugin.log, which stayed empty: that is the settings folder, not the switch file location (macOS: ~/Library/Logs/OpenRB-338/plugin.log); not a bug. docs/plugin.md section 10 now lists the switch file of each system in a table and says a plugin.log in the settings folder does nothing.
Criterion 4 (macOS and Windows editors still work): macOS by the user's check above; Windows by the existing plugin tests passing (cargo test 973 pass) and the CI release build of the branch compiling the Windows packages; no Windows host check was run.
Pending before Done: after task-244 merges, rebase and fold its ORB_PLUGIN_LOAD_LOG (crates/orb-plugin/src/load_log.rs) into ORB_PLUGIN_LOG (supervisor, 2026-11-21).

Round 4 (2026-11-21, after task-244 merged, main 6a93e04):
- Rebased onto main; the one conflict (docs/building.md variable table) resolved by hand keeping ORB_REAPER and ORB_PLUGIN_LOG.
- task-244's load log folded into ORB_PLUGIN_LOG (supervisor's request): crates/orb-plugin/src/load_log.rs and ORB_PLUGIN_LOAD_LOG removed; plugin_log::image_path (dladdr, moved with its unit test), plugin_log::activated(rate) called from Plugin::activate (main thread; with the log off only the switch lookup, once per process) writing "activate: loaded from <path> at <rate> Hz", and parse_loaded_from, which tests/reaper.rs uses with ORB_PLUGIN_LOG set to <scratch>/plugin.log. docs/plugin.md module table, 7.1 and 10, docs/building.md table (old name ORB_PLUGIN_LOAD_LOG), AGENTS.md "Not test inputs" updated.
- wgpu noise: wgpu, wgpu_core, wgpu_hal and egui_wgpu now log at INFO and above; ORB_PLUGIN_LOG_LEVEL=debug brings their DEBUG back (documented in docs/plugin.md 10 and docs/building.md). egui-baseview and the plugin stay at DEBUG, the probe unchanged. clap_gui with the log on: 61 lines for its seven editor opens, against 3,420 before.
- Checks: cargo test 977 pass (99 binaries); clippy --all-targets -D warnings clean; nightly fmt --check clean; ORB_REAPER=/Applications/REAPER.app/Contents/MacOS/REAPER cargo test -p orb-plugin --test reaper: 2 pass, all 20 renders (new instances with -newinst on scratch resource folders; the user's REAPER untouched); a scratch plugin.log has the header and three "activate: loaded from .../target/bundled/OpenRB-338.clap/Contents/MacOS/OpenRB-338 at 44100 Hz" lines.

Review round 1 (2026-11-21, send back, docs and notes only):
- docs/plugin.md section 10: the DEBUG volume now gives the measured figures (macOS clap_gui, seven editor opens: 3,420 lines with wgpu DEBUG, 61 at the default level) instead of "about 3,000 lines for four editor opens".
- Correction to round 3's criterion 4 note: the CI release build run 44887244806 that compiled the Windows (and Linux) packages built commit 44c615c (round 1), not the final commit. The final commit's Linux and Windows compile, clippy and tests are checked by ci.yml on main after the merge (the supervisor watches that run).
- Superseded: round 2's last bullet ("Criterion 5 ... waits for the user's macOS check in one host; the task stays To Do") is superseded by round 3 (the user's macOS check, criterion 4 checked) and round 4 (Done).
- AGENTS.md "Not test inputs": the paragraph rewrapped (no dangling "only). Cargo's own" line).
No code change; tests not rerun.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the plugin has a diagnostic log that works inside a host, and with it the cause of the black (VST3) or stale (CLAP) editor in Bitwig on the user's software-GL Ubuntu VM was found. The fix is task-262 (by the user's decision of 2026-11-21), after Decide task-261.

- Log (crates/orb-plugin/src/plugin_log.rs, release builds): ORB_PLUGIN_LOG=FILE|stderr, or the switch file plugin.log in orb-ui's log folder (~/.local/state/OpenRB-338, ~/Library/Logs/OpenRB-338, %LOCALAPPDATA%\OpenRB-338\logs). It records each activation's library and rate (task-244's ORB_PLUGIN_LOAD_LOG folded in; the REAPER test reads it), wgpu and egui-wgpu at info (debug with ORB_PLUGIN_LOG_LEVEL=debug), egui-baseview at debug, panics, the adapter choice with the surface capabilities, the editor's frames and, on X11, a Present probe. NICE_LOG never worked because the plugin has no nice-plug logger.
- Adapter: an egui-wgpu selector honors WGPU_BACKEND; the default is unchanged (all backends, wgpu-core 30's pick).
- Cause (the user's VM log, build 2026.1121.0857): baseview 0.4.2 on X11 draws a frame only on a Present CompleteNotify whose MSC advances; the VM's Xorg (VMware SVGA II, modesetting without glamor) answers every NotifyMSC at once with MSC 0 and UST 0, so the editor draws one frame and stops. The adapter was Vulkan llvmpipe, not GL. Close-path X11 errors are logged too, and task-262 checks them against another user's Debian crash on closing the CLAP editor.
- Criterion 3 (the editor draws in Bitwig) moved to task-262 by the user on 2026-11-21.
- Checks: macOS bundles checked by the user ("plugin seems to work"); Windows by the tests and the CI release build. cargo test 977 pass, clippy -D warnings and nightly fmt clean, the REAPER test 2 pass (20 renders).
- Branch worktree-agent-ac59c7502f94651b6, one commit; docs/plugin.md section 10 and 7.1, docs/building.md, AGENTS.md.
<!-- SECTION:FINAL_SUMMARY:END -->
