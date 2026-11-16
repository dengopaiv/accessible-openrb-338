---
id: TASK-224
title: >-
  Plugin: Reaper crashes (stack overflow) when switching the FX window from the
  parameter list to the plugin UI
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T16:34:00Z'
updated_date: '2026-11-16T17:11:00Z'
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
- [x] #1 The cause of the resize recursion is identified, with evidence (code paths, a reproduction)
- [x] #2 The loop cannot occur: switching Reaper's FX window between the parameter list and the plugin UI, and the other FX-window transitions, no longer crash (a test where feasible; else the user's check)
- [x] #3 Any baseview change is a minimal documented vendored patch, with upstream issue text in the notes
- [x] #4 The user checks it in Reaper (CLAP and VST3) before merge
- [x] #5 cargo test, clippy -D warnings and nightly fmt pass; validators pass on the rebuilt bundles
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Cause: read the crash report and baseview 0.4.2 (macOS view.rs, window.rs), nice-plug 0.4.2's CLAP gui.create (spawn with no parent) and VST3 attached (spawn with the parent); reproduce in tests/clap_gui.rs with a host that refuses gui.request_resize and sizes the editor before set_parent.
2. Fix in our layer (no baseview patch): the editor's host-callback proxy (HostProxy) answers baseview's own resize requests Ok without asking the host while the editor has no parent yet, while the host's set_size runs, and while another request is in progress; the route knows set_parent and set_size through OrbHandle. Baseview's revert (the loop's fuel) then never runs where it re-enters.
3. Check the VST3 path and the other transitions (set_scale, parenting, the host window resized, backing change, hide/show, close/reopen) in the regression test.
4. docs/plugin.md section 10: the cause and the guard; upstream issue text in the notes.
5. cargo test, clippy, nightly fmt; bundles built and validated; AC #4 left for the user.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad2c1deec88795039 (worktree .claude/worktrees/agent-ad2c1deec88795039).

Evidence (cause):
- Crash reports REAPER-2026-11-16-183352.ips and -183449.ips: both EXC_BAD_ACCESS/SIGSEGV past the main thread's stack guard, image /Users/USER/Library/Audio/Plug-Ins/CLAP/OpenRB-338.clap (the CLAP bundle). 511 frames of the cycle BaseviewView::window_did_resize+132 -> ViewImpl::view_did_change_backing_properties+916 -> BaseviewView::resize+160 -> -[NSWindow _setFrameCommon:display:fromServer:]+2668 -> NSNotificationCenter post -> window_did_resize. The origin frames are cut off (511-frame limit).
- windowDidResize: reaches the view only as the NSWindow's delegate, which baseview 0.4.2 sets only for a window of its own (ViewParentingType::Windowed, platform/macos/view.rs 40-55 setup -> set_delegate). resize() touches an NSWindow only in Windowed mode (setContentSize, view.rs 264-267). So at the crash the editor was in a baseview-owned window, not yet in Reaper's view.
- Why Windowed: nice-plug 0.4.2 CLAP ext_gui_create calls Editor::spawn(None, wait_for_parent=true, ...) (wrapper/clap/wrapper.rs 2775); baseview 0.4.2 macOS create_window ignores wait_for_parent (platform/macos/window.rs 36-50: no parent -> create_window_standalone). The view stays in that hidden window until gui.set_parent (BaseviewView::set_parent tears it down).
- The loop (view.rs 362-405): host set_size -> Window::resize -> BaseviewView::resize(S1, notify=false, from_window=false) -> setContentSize(S1) -> nested window_did_resize -> resize(S1, notify_host=true) -> view_did_change_backing_properties: state S0 -> S1, host.request_resize(S1) -> host refuses -> resize(previous S0) -> setContentSize(S0) -> window_did_resize -> state S1 -> S0, request S0 refused -> resize(S1) -> ... (state.size is updated only after the nested notification returns, so each level sees a change). It needs a refused request (or a handler error; egui-baseview 0.9 resized always returns Ok). baseview's Windows backend has the missing check: request_host_resize returns Ok when resize_host_originated (platform/win/window.rs 306-311, window_state.rs 263).
- Reproduction: tests/clap_gui.rs the_host_sizes_the_editor, unfixed: a host whose gui.request_resize returns false; gui.create(cocoa, embedded), adjust_size+set_size(900x700) before set_parent -> request_resize called 101 times (the test aborts at 100), lldb bt: 1656 main-thread frames, 101 _setFrameCommon frames, origin clap_gui.rs:292 (the first set_size, before set_parent). With a host that accepts (temporary TMP_ACCEPT run): no loop, but the host was asked 4 times for sizes it had set itself.
- So in Reaper: switching to the UI does gui.create, then sizes the editor to the FX window (gui.set_size) before gui.set_parent, and refuses gui.request_resize at that point (the user's case; Reaper's own code not read).
- VST3: nice-plug's attached() spawns with Some(parent) (wrapper/vst3/view.rs 559): Parented from the start, the view is no window's delegate, no window_did_resize; and Vst3HostCallbacks::request_resize always returns Ok (view.rs 500-520). No loop possible. In Parented mode baseview's revert is setFrameSize on its view only (no notification), so after set_parent the CLAP path cannot loop either.

Fix (our layer, no baseview patch): crates/orb-plugin/src/editor.rs. HostProxy (the host callbacks baseview gets through nice-plug-egui) answers baseview's own request_resize Ok without asking the host (a) while the window has no parent (RouteState.parented, set by OrbHandle::set_parent on success, or at spawn when VST3 gives the parent), (b) while the host's set_size runs (OrbHandle::set_size inside HostRoute::host_sizing; the flag is restored by a drop guard, also on a panic), (c) when the callbacks are already borrowed (a request in progress; it was Err before, which made baseview revert). baseview then never reverts in its own window; in the host's view a revert is setFrameSize only. The corner's requests (HostRoute::flush) still go to the host directly. OrbHandle now holds the route.
Tests: tests/clap_gui.rs the_host_sizes_the_editor (macOS; host refuses requests and counts them; two rounds of create -> adjust/set_size 900x700, 500x300 -> set_scale 2 -> set_size 700x600 -> set_parent into an NSWindow's content view -> show -> set_size 800x650 -> viewDidChangeBackingProperties sent to the editor's view -> host window setContentSize 1000x800 -> set_size 1000x800 -> hide/show -> destroy; get_size equals each set size; 0 requests). Before the fix: abort after 101 refused requests; after: clap_gui ok. tests/editor.rs the_host_is_not_asked_for_sizes_before_the_parent_or_its_own (unit: unparented Ok/not asked; host_sizing Ok/not asked; after a panicking host_sizing, asked and the refusal passed on; a nested request during the corner's flush Ok/not asked). Dev-dependencies objc2 0.6.5, objc2-app-kit 0.3.2, objc2-foundation 0.3.2 (macOS only; already in Cargo.lock through baseview).
A real backing-scale change (moving between displays) cannot be made in a test; viewDidChangeBackingProperties with no change is sent instead. On a real change in the host's view, baseview asks the host for the same logical size; a refusal reverts with setFrameSize (no notification), so it terminates.

Upstream issue text for baseview (not posted):
Title: macOS: resize loop (stack overflow) when the host refuses request_resize while the view is in its own window
Body: baseview 0.4.2, macOS. With no parent, Window::create_with_host makes a standalone NSWindow and sets the view as its delegate, even with wait_for_parent set (wait_for_parent is used only on win32), so a plugin framework that creates the editor before the host's set_parent (nice-plug's CLAP gui.create) has the view in a window of its own until set_parent. When that window is resized (Window::resize from the host's gui.set_size), windowDidResize: calls BaseviewView::resize(size, notify_host = true, from_window = true), whose view_did_change_backing_properties calls HostCallbacks::request_resize; if the host refuses, it calls resize(previous, false, false), which for a Windowed view calls setContentSize, which posts NSWindowDidResizeNotification synchronously and re-enters window_did_resize with the previous size. state.size is only updated after the nested call returns, so every level sees a change and the two sizes alternate until the main thread's stack overflows (seen in REAPER 7 on macOS 15). The win32 backend avoids the echo with resize_host_originated (request_host_resize returns Ok for a host-originated resize). Suggested fixes: on macOS, do not call request_resize for a resize of baseview's own window that came from Window::resize (a resize_host_originated flag as on win32), do not revert with setContentSize from inside windowDidResize (or guard re-entry), and honor wait_for_parent by not creating a visible-capable standalone window.

AC evidence:
- #1: crash report cycle + baseview 0.4.2 code paths + nice-plug spawn(None) for CLAP (notes above); reproduced in tests/clap_gui.rs (101 refused requests, lldb origin = gui.set_size before gui.set_parent).
- #2: the loop needs a refused request inside baseview's own window; HostProxy no longer asks (nor errs) before the parent, during the host's set_size, or nested. clap_gui the_host_sizes_the_editor passes (0 requests, every transition listed above, two create/destroy rounds). VST3 cannot loop (Parented from spawn, request always Ok). The Reaper confirmation is AC #4.
- #3: no baseview change (guard in our layer); upstream issue text in the notes above.
- #5: cargo test: 814 passed, 0 failed, 5 ignored (clap_gui ok); cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean; cargo xtask bundle orb-plugin --release built target/bundled/OpenRB-338.clap and .vst3; scripts/validate-plugin.sh: validators: all passed (clap-validator, pluginval SUCCESS).

Cargo.lock: the macOS dev-dependencies take objc2-app-kit/objc2-foundation without default features (the features the test uses), so no new packages enter the lock (3 dependency lines). Re-run after that: clap_gui ok, orb-plugin 69 tests pass, clippy and fmt clean.

Review round 1 (send back; code sound, docs and test wording):
1. docs/plugin.md: corrected 'nice-plug's VST3 request always returns Ok' (false: wrapper/vst3/view.rs 508-519 returns Err on FailedToPostTask and PluginClosed; the earlier note 'Vst3HostCallbacks::request_resize always returns Ok' was wrong too). VST3 cannot loop because the view is parented from spawn (no baseview window, no windowDidResize), so baseview's revert there is setFrameSize on its view only, which posts nothing.
2. docs/plugin.md: 'the backing-properties notification with no change' instead of 'the backing properties changing'.
3. clap_gui.rs: the host-window resize step reaches no baseview code by itself (the view is not that window's delegate and does not autoresize); the test comment, the doc comment and docs/plugin.md now say it checks only that the parented editor survives it and the set_size after.
4. Count: the old cap (n = fetch_add before increment, n > 100) aborted on the 102nd call after 101 refused requests (lldb: 101 _setFrameCommon frames; its message said 'called 101 times'). The cap now counts calls (fetch_add + 1, n > 100): the 101st call aborts and the message gives the true count; docs say the requests repeat until the test aborts at the 101st.
5. docs/plugin.md: the in-progress branch (callbacks borrowed => Ok) accepts a baseview request nested in any host callback holding the callbacks, not only set_size; a host resizing the view inside a request by other means than set_size would leave baseview's size and the host's apart until the next set_size (no known host does).
6. Still To Do with the DRAFT summary until the user's Reaper check.
Only test and doc text changed (no plugin code): bundles not rebuilt. Re-run: clap_gui ok, clippy and nightly fmt clean.

Rebased onto main f84d164 (task-218 Gate mode merged): no conflicts. After it: cargo test 830 passed, 0 failed, 5 ignored (clap_gui ok); clippy -D warnings and nightly fmt clean; bundles rebuilt (target/bundled/OpenRB-338.clap, .vst3; not installed) and scripts/validate-plugin.sh: all passed.
AC #4: the user checked the crash-fix build (bundles of e9ce183, before this rebase) in Reaper on 2026-11-16: "crash fix looks fine". The rebase only brought in task-218 (no conflicts, no change to this task's code).
Reviewer nits in the same amend: docs/plugin.md rewrapped (the 101st-request line) and the unsourced '(no known host does)' softened to '(not seen in the hosts tried)'.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reaper (CLAP) crashed with a main-thread stack overflow when its FX window switched from the parameter list to the plugin UI.
Cause: nice-plug 0.4.2 spawns a CLAP editor in gui.create with no parent, and baseview 0.4.2 on macOS then keeps the view in a hidden window of its own (wait_for_parent ignored) until gui.set_parent. Reaper sizes the editor there (gui.set_size) and refuses gui.request_resize; baseview's windowDidResize asked the host for the size it had just set and, on the refusal, set the old size back with setContentSize, re-posting the resize notification with the sizes swapped until the stack ran out (the crash report's cycle).
Fix (crates/orb-plugin/src/editor.rs, no baseview patch): HostProxy answers baseview's own requests Ok without asking the host before the window is parented (HostRoute::set_parented), during the host's set_size (HostRoute::host_sizing) and while another request is in progress. VST3 cannot loop (parented from spawn; the revert there is setFrameSize only). docs/plugin.md section 10 documents it; upstream baseview issue text is in the notes.
Tests: tests/clap_gui.rs the_host_sizes_the_editor (reproduced the loop before the fix; passes after, across set_scale, parenting, a backing-properties notification, the host window resized, hide/show, close/reopen) and tests/editor.rs the_host_is_not_asked_for_sizes_before_the_parent_or_its_own. cargo test 830 pass (5 ignored), clippy and nightly fmt clean, bundles validated. The user confirmed the fix in Reaper on 2026-11-16 (pre-rebase build). One review round (docs and test wording).
Branch worktree-agent-ad2c1deec88795039, one commit on main f84d164.
<!-- SECTION:FINAL_SUMMARY:END -->
