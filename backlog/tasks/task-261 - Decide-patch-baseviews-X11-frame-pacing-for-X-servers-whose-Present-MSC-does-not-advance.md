---
id: TASK-261
title: >-
  Decide: patch baseview's X11 frame pacing for X servers whose Present MSC does
  not advance
status: Done
assignee:
  - '@pingw33n'
created_date: '2026-11-21T08:51:00Z'
updated_date: '2026-11-24T15:55:00Z'
labels:
  - decision
  - ui
  - player
milestone: m-7
dependencies: []
references:
  - docs/plugin.md
  - crates/orb-plugin/src/plugin_log.rs
priority: medium
type: spike
ordinal: 254000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-260 (diagnosis) and task-262 (the fix, which depends on this decision). In Bitwig on the user's software-GL Ubuntu VM the plugin editor shows its first frame (menu bar, splash) and then never draws again (VST3: black panel; CLAP: stale screen), while the standalone app draws. task-260's log confirmed the cause on 2026-11-21 (docs/plugin.md section 10, "Linux: baseview paces every frame on the X server's Present extension"):

- baseview 0.4.2 on X11 draws a frame only when the X server's Present extension answers its PresentNotifyMSC with a CompleteNotify whose MSC is past the last one (src/platform/x11/present.rs 72, 134-158, 192-226). A completion whose MSC did not advance draws nothing, and the next request is sent only after the previous one was answered. Its 15 ms fallback timer (present.rs 160) runs only when the server has no Present extension or refuses the request.
- That X server answers every PresentNotifyMSC at once with MSC 0, so the editor draws one frame and stops.

The log of the user's VM run (build 2026.1121.0857, dev-44c615c; /Users/pingw33n/win_exchange/log plugin.log, engine.log, info.txt):
- System: Ubuntu, kernel 7.0.0-30-generic, VMware SVGA II adapter, Xorg (vt2, gdm) on modesetting; "Refusing to try glamor on llvmpipe", glamor off; "Initializing extension Present"; xdpyinfo lists DRI2 and Present.
- Adapter: Vulkan llvmpipe (LLVM 21.1.8), device type Cpu, taken over GL llvmpipe (both listed, both present to the window), WGPU_BACKEND unset, power preference HighPerformance. Surface formats [Bgra8UnormSrgb, Bgra8Unorm], present modes [Immediate, Mailbox, Fifo, FifoRelaxed]; egui takes Bgra8Unorm, AutoVsync.
- Present probe: "Present 1.2; window 0x3800003, 656x532 at (391, 124), depth 32"; six NotifyMSC (target 0, then 2) each complete in 0-6 ms with MSC 0, UST 0, kind NOTIFY_MSC, mode COPY: the MSC never advances.
- Frames: frame 1 at 121 ms, then the count is 1 at 1, 2, 4, 8 and 16 s; "editor: closed after 1 frames". Same for the VST3 (parent None, wait for parent true) and the CLAP (parent Some(XlibWindow(58720256))).
- On closing: "WARN baseview::platform::x11::window_thread: Error while closing window: Request to X11 thread failed: Could not receive response (X11 thread disconnected)", then baseview X11 errors BadWindow (bad value 58720259 = the editor window) for Present SelectInput (major 148 minor 3) and core DestroyWindow (major 4).

The fix is in baseview, a dependency (nice-plug-egui 0.6 -> egui-baseview 0.9 -> baseview 0.4.2, the latest on crates.io on 2026-11-21); the plugin cannot reach baseview's pacing from outside (draw_now is private, Present events cannot be faked with SendEvent).

Options:
A. Vendor baseview 0.4.2 in the repository (vendor/baseview, [patch.crates-io], as raw-window-metal is vendored: docs/plugin.md "The vendored raw-window-metal") with a small patch to present.rs: when a CompleteNotify does not advance the MSC, or none arrives within about 100 ms of the request, switch the window to the 15 ms fallback timer for the rest of its life (a log line says so). Real vblank keeps the Present pacing. Upstream the patch as a PR to baseview and drop the vendored copy when a release has it.
B. Vendor it with Present pacing off on X11 altogether (always the 15 ms timer): the smallest patch; frames are no longer aligned to vblank, wgpu's own Fifo present still paces them on real hardware.
C. A fork on GitHub as a git dependency instead of a vendored copy (same patch as A or B).
D. Only report it upstream and wait; the plugin stays blank on such X servers.

Recommendation: A (keeps vsync where it works, fixes the VM, follows the raw-window-metal precedent), with the upstream PR.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user chose an option (A, B, C or D); the outcome and the date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user on 2026-11-24: option A's patch (when a CompleteNotify does not advance the MSC, or none arrives within about 100 ms, the window switches to baseview's 15 ms fallback timer for the rest of its life, with a log line), shipped as option C: a GitHub fork of github.com/RustAudio/baseview on the user's account (as the nice-plug and egui-baseview forks), branch OpenRB-338 on the published 0.4.2 commit 4fe59b3, used through [patch.crates-io]. A [patch.crates-io] for baseview reaches egui-baseview's fork unchanged. No upstream pull request unless the user asks. The user does not schedule task-262 (the fix) yet.
<!-- SECTION:FINAL_SUMMARY:END -->
