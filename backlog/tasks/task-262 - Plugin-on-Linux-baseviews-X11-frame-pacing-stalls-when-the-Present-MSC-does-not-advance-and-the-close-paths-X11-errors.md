---
id: TASK-262
title: >-
  Plugin on Linux: baseview's X11 frame pacing stalls when the Present MSC does
  not advance, and the close path's X11 errors
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T09:18:00Z'
labels:
  - player
  - ui
milestone: m-7
dependencies:
  - TASK-261
references:
  - docs/plugin.md
  - crates/orb-plugin/src/plugin_log.rs
  - /Users/pingw33n/win_exchange/log/plugin.log
priority: medium
type: bug
ordinal: 255000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-260 found why the plugin editor stays black (VST3) or stale (CLAP) in Bitwig on the user's VM: baseview 0.4.2 on X11 draws a frame only on a Present CompleteNotify whose MSC is past the last one (src/platform/x11/present.rs 72, 134-158, 192-226; its 15 ms fallback timer, 160, runs only without the extension), and that X server answers every PresentNotifyMSC at once with MSC 0. The editor draws one frame and never again. docs/plugin.md section 10 ("Linux: baseview paces every frame on the X server's Present extension") has the mechanism and the evidence.

The log of the user's VM run (build 2026.1121.0857, dev-44c615c; /Users/pingw33n/win_exchange/log plugin.log, engine.log, info.txt):
- System: Ubuntu, kernel 7.0.0-30-generic, VMware SVGA II adapter, Xorg (vt2, gdm) on modesetting; "Refusing to try glamor on llvmpipe", glamor off; "Initializing extension Present"; xdpyinfo lists DRI2 and Present.
- Adapter: Vulkan llvmpipe (LLVM 21.1.8), device type Cpu, taken over GL llvmpipe (both listed, both present to the window), WGPU_BACKEND unset, power preference HighPerformance. Surface formats [Bgra8UnormSrgb, Bgra8Unorm], present modes [Immediate, Mailbox, Fifo, FifoRelaxed]; egui takes Bgra8Unorm, AutoVsync.
- Present probe: "Present 1.2; window 0x3800003, 656x532 at (391, 124), depth 32"; six NotifyMSC (target 0, then 2) each complete in 0-6 ms with MSC 0, UST 0, kind NOTIFY_MSC, mode COPY: the MSC never advances.
- Frames: frame 1 at 121 ms, then the count is 1 at 1, 2, 4, 8 and 16 s; "editor: closed after 1 frames". Same for the VST3 (parent None, wait for parent true) and the CLAP (parent Some(XlibWindow(58720256))).
- On closing: "WARN baseview::platform::x11::window_thread: Error while closing window: Request to X11 thread failed: Could not receive response (X11 thread disconnected)", then baseview X11 errors BadWindow (bad value 58720259 = the editor window) for Present SelectInput (major 148 minor 3) and core DestroyWindow (major 4).

What to do (after Decide task-261 picks the option: A vendored baseview with a fallback to the 15 ms timer when the MSC does not advance or no answer comes in about 100 ms, B vendored with Present pacing off on X11, C a git fork, D upstream only):
1. Apply the chosen patch to baseview 0.4.2 (nice-plug-egui 0.6 -> egui-baseview 0.9 -> baseview). Keep the Present pacing where the MSC advances (option A).
2. Log the switch to the fallback timer in the plugin log (ORB_PLUGIN_LOG or the plugin.log switch file, task-260).
3. Check the close path. Another user reported, on Debian 13 with Bitwig 6.1: "audio engine crash when I close the CLAP version" - every time the editor window is closed, not when it is minimized. The VM log shows, on every close, baseview's "Error while closing window: Request to X11 thread failed: Could not receive response (X11 thread disconnected)" and X11 BadWindow errors for Present SelectInput and DestroyWindow on the already destroyed window (baseview x11 window_thread.rs Drop, xcb_window.rs Drop). Find out whether that path (or something else on close) can crash the host process, fix it with the same patch if it is baseview's, and write the finding in docs/plugin.md.
4. The user checks in Bitwig on the VM (log on) and, if possible, the Debian user's close.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The VST3 and CLAP editors draw the full panel in Bitwig on the user's VM, confirmed by the user (moved from task-260 criterion 3 by the user on 2026-11-21)
- [ ] #2 The editor keeps the Present pacing on an X server whose MSC advances, or the Decide task chose otherwise; the plugin log shows which pacing a window uses
- [ ] #3 Whether baseview's close path (the X11 thread disconnected, BadWindow on Present SelectInput and DestroyWindow) causes the Debian 13 / Bitwig 6.1 crash on closing the CLAP editor is found and written in docs/plugin.md; if it does, it is fixed
- [ ] #4 The baseview change is documented in docs/plugin.md (what is patched and why, with file:line); macOS and Windows editors still work (the plugin tests pass)
<!-- AC:END -->
