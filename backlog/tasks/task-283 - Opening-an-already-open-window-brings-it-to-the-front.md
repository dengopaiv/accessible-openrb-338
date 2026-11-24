---
id: TASK-283
title: Opening an already open window brings it to the front
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-22T16:27:00Z'
updated_date: '2026-11-24T07:33:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/info.rs
  - crates/orb-ui/src/mod_info.rs
  - crates/orb-ui/src/about.rs
  - crates/orb-ui/src/key_help.rs
  - crates/orb-ui/src/plugin_usage.rs
  - crates/orb-ui/src/editor_frame.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 261700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-22. Choosing a menu item or shortcut for a window that is already open does nothing visible today when the window is behind another one: e.g. File > Info with Song Information already open should bring Song Information to the front and focus it. The same goes for every window the app opens.

**The windows** (all egui::Window inside the one app viewport, so "focus" means raising the window's layer to the top, e.g. egui's Context::move_to_top on the window's LayerId, and giving it keyboard focus where it has a focusable field):
- Song Information (crates/orb-ui/src/info.rs, File > Info)
- Mod Information (crates/orb-ui/src/mod_info.rs)
- About (crates/orb-ui/src/about.rs)
- Keyboard Shortcuts (crates/orb-ui/src/key_help.rs)
- Plugin Usage (crates/orb-ui/src/plugin_usage.rs)
- the 303, 808 and 909 pattern editors (crates/orb-ui/src/editor_frame.rs; opened from the Edit menu and Cmd/Ctrl+E)
- any other window found in crates/orb-ui and crates/orb-plugin (search for egui::Window); list them all in the notes.

Opening a window that is not open keeps today's behavior (its saved position, if any). Raising an open window does not move it, resize it or reset its state (e.g. a typed value in a pattern editor stays).

Works in the standalone app and the plugin (the plugin opens windows from the menus only; it takes no keys, task-201).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every window opened from a menu or shortcut, listed in the notes, comes to the front when its command is chosen while it is already open, including when another window covers it
- [x] #2 Raising an open window keeps its position, size and contents
- [x] #3 A test per window (or one table-driven test) opens it, opens another window over it, repeats the first window's command and checks the first window's layer is on top
- [x] #4 docs/ui_spec.md states the rule once, in the section that describes the app's windows
- [x] #5 cargo test --workspace, clippy -D warnings and nightly fmt --check pass
- [x] #6 The user has checked it in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. One shared mechanism: App::to_front (ids) + App::raise_window, drained by raise_windows (Context::move_to_top, Middle order) after all window ui fns in App::show. 2. Each show_* (info, mod_info, about, key_help, plugin_usage) calls raise_window when already open. The three pattern editors already raise through Placing::front (editor_frame.rs). 3. Test tests/raise_windows.rs: every ordered pair of the 8 windows. 4. docs/ui_spec.md section 1 states the rule.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a1c1516a5a4577421

Windows found (grep egui::Window in crates/orb-ui and crates/orb-plugin; orb-plugin has none of its own, it uses App): Song Information (info.rs, File > Info), Mod Information (mod_info.rs, Mods > Info), About (about.rs), Keyboard Shortcuts (key_help.rs), Plugin Usage (plugin_usage.rs, plugin menu only), and the 303/808/909 pattern editors (editor_frame.rs show_window, already brought to the front by Placing::open). Others (file browser, prompts, error box, export dialog) are not egui::Window instances opened by a menu command toggle, so not covered; not changed. Design choices: no keyboard focus is moved on a raise (nothing to focus: Song Information's fields keep whatever focus they have); the raise happens at the end of the frame, so it applies from the next frame.

Evidence: tests/raise_windows.rs runs every ordered pair of the 8 windows (open A, open B over it, repeat A, check A is top of the Middle order and its rect is unchanged): passes. cargo test --workspace 1066 pass 0 fail; clippy -D warnings and nightly fmt --check clean. Dev app target/OpenRB-338.app and VST3 target/bundled/OpenRB-338.vst3 built. AC 6 awaits the user.

User sign-off 2026-11-24: looks good (a raise does not move keyboard focus; it takes effect from the next frame).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Opening an already open window brings it to the front. App::raise_window queues the id and App::raise_windows moves the layer to the top at the end of the frame; Song Information, Mod Information, About, Keyboard Shortcuts and Plugin Usage use it, the three pattern editors already did via Placing::front. tests/raise_windows.rs checks every pair of the 8 windows; docs/ui_spec.md section 1 states the rule. Branch worktree-agent-a1c1516a5a4577421, one commit (amended after rebase onto a5f953a). cargo test --workspace, clippy -D warnings and nightly fmt --check pass (counts in the report). Decisions, signed off by the user: no keyboard focus move on a raise; effect from the next frame.
<!-- SECTION:FINAL_SUMMARY:END -->
