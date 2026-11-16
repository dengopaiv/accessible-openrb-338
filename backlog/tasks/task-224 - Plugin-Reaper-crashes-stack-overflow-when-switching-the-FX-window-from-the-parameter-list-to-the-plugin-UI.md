---
id: TASK-224
title: >-
  Plugin: Reaper crashes (stack overflow) when switching the FX window from the
  parameter list to the plugin UI
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-16T16:34:00Z'
labels:
  - ui
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/editor.rs
  - crates/orb-ui/src/embed.rs
  - docs/plugin.md
  - ~/Library/Logs/DiagnosticReports/REAPER-2026-11-16-183352.ips
priority: high
type: bug
ordinal: 229000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Report (the user, 2026-11-16, macOS 15.7.5 arm64, Reaper 7.86, CLAP bundle built from task-218's branch, which includes the merged task-201 editor):** in Reaper's FX window, switching from the generic parameter list to the plugin's UI (the "UI" button) crashed Reaper. The crash report is in ~/Library/Logs/DiagnosticReports/; find the REAPER-2026-11-16-1833*.ips file.

**Crash:** EXC_BAD_ACCESS on the main thread, past the stack guard: a stack overflow from unbounded recursion. The stack repeats this cycle hundreds of times:
`baseview::platform::macos::view::BaseviewView::window_did_resize` → `view_did_change_backing_properties` → `BaseviewView::resize` → `-[NSWindow _setFrameCommon:display:fromServer:]` → NSNotificationCenter posts NSWindowDidResize → `window_did_resize` → ...

baseview's view observes the window's resize notification. Its backing-properties handler calls `resize`, which sets the NSWindow's frame (the host's window!), and that posts another resize notification. A frame change that doesn't converge, or setting the same frame repeatedly, loops forever. Likely factors to check:
- our size rule: height = round(width × 510/624) + 22, enforced through adjust_size and the corner's host requests;
- the scale factor: logical vs physical points, rounding;
- baseview resizing the parent/host NSWindow at all, when it's a child view of a window it doesn't own;
- the state Reaper's FX window is in when it switches modes, possibly a different size or scale.

**To do:**
1. Reproduce it: Reaper if possible through the user, otherwise a test host or the clap_gui harness. Try a parent window whose size the host changes after set_parent, and a change of backing scale. Find the exact cause.
2. Fix it so the loop can't happen. For example:
   - a re-entrancy guard in our layer;
   - make sure no editor path asks to resize the host window from inside a resize notification;
   - if the bug is in baseview itself (resizing a window it doesn't own, or no guard), a minimal vendored patch under vendor/ like raw-window-metal, documented in docs/plugin.md with upstream issue text in the notes (not posted).
3. Check the VST3 path for the same loop, and Reaper's other FX-window transitions: float/dock, close and reopen, moving between displays with different scale.
4. A regression test where feasible (macOS), for example clap_gui opening the editor in a window that is then resized and has its backing scale changed.
5. Rebuild the bundles into the worktree's target and validate them with scripts/validate-plugin.sh. The user checks the fix in Reaper before merge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The cause of the resize recursion is identified, with evidence (code paths, a reproduction)
- [ ] #2 The loop cannot occur: switching Reaper's FX window between the parameter list and the plugin UI, and the other FX-window transitions, no longer crash (a test where feasible; else the user's check)
- [ ] #3 Any baseview change is a minimal documented vendored patch, with upstream issue text in the notes
- [ ] #4 The user checks it in Reaper (CLAP and VST3) before merge
- [ ] #5 cargo test, clippy -D warnings and nightly fmt pass; validators pass on the rebuilt bundles
<!-- AC:END -->
