---
id: TASK-235
title: >-
  UI: About window title like the other windows; closing a window over the logo
  plays the startup sound
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-17T11:09:00Z'
updated_date: '2026-11-17T11:39:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src
  - crates/orb-plugin/src/editor.rs
  - task-221
priority: medium
type: bug
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two small UI bugs the user found in the plugin (2026-11-17); check the standalone app too, since the windows come from orb-ui.

1. The About window's title bar looks different from the other windows (File > Song Info, Mods > Info, Help > Plugin Usage, task-221). Make About's title bar match theirs (style, font, close button, layout), in the plugin and the standalone.

2. Closing a window whose X (close) button lies over the panel's logo also clicks the logo: the click reaches the panel under the window, and the mod's startup sound plays. A click on a window's close button (and, check, anywhere in a window's frame or body) must never reach the panel underneath. Fix it once for all orb-ui windows, not only About, and look for the same click-through on other panel controls under a window.

Help > Plugin Usage is on task-221's branch, not yet on main; match the windows on main and, once task-221 lands, the supervisor rebases. Visual change: the user checks it in the plugin before merge.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The About window's title bar matches the other windows (side-by-side screenshots of About and Song Info, plugin and standalone).
- [x] #2 Closing any orb-ui window with its close button over the logo (or another panel control) does not activate the control underneath (test with a headless egui click at the close button's position over the logo).
- [x] #3 The standalone's --screenshot changes only if About is in it; say which in the notes.
- [x] #4 cargo test, clippy -D warnings and nightly fmt pass; the user's check in the plugin.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. About: drop the zero-margin custom frame so it gets egui's default window frame like Song/Mod Information and Keyboard Shortcuts. 2. Click-through: App::focus_click (panel row focus and the logo's startup sound) read the raw press, so a press on any layer over the panel counted; skip it when ctx.layer_id_at(press) is not the panel's layer (covers every window, menu and box, and Plugin Usage when it lands). The other controls use ui.interact, which already respects layers. 3. Test with a headless drag of About over the logo and clicks on its title and close button. 4. Screenshots, bundles, tests.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa0c8bc17e709fca0

Cause 1: about.rs overrode the window frame with inner_margin(0); egui applies the frame margin to the content only (the title bar keeps the window frame), so the title bar differed: now the default frame (task-221 Plugin Usage uses the default too). Measured before: About title band 41 px vs Song Info 65 px at the same scale; after: same style. Cause 2: app.rs focus_click used the raw press origin. Fix: layer_id_at(press) must be the panel's layer. Other panel controls use ui.interact (hit-tested by layer), checked by reading app.rs. Test a_click_on_a_window_over_the_logo_does_not_reach_the_panel (tests/menu.rs) fails without the fix (preview_position Some(4096)) and passes with it. Screenshots (standalone --screenshot shows About only with --about, so the normal --screenshot is unchanged; --about changes): /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/shots/sa_before.png sa_after.png si_before.png si_after.png (standalone), /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/shots/pl_about.png pl_info.png (plugin editor_preview; both caught mid fade-in, translucent, the preview captures early). Bundles built in the worktree: target/bundled/OpenRB-338.clap and .vst3, scripts/validate-plugin.sh: all passed. cargo test 878 pass, clippy -D warnings and nightly fmt clean. The user's plugin check is pending.

Review round 1: status_bar (app.rs) cleared the status line on any click, also one on a window or menu over the bar. It now needs the click's layer (interact_pos, layer_id_at) to be the panel's. New test a_click_on_a_window_over_the_status_bar_keeps_the_status (fails without the fix, checked). Re-run: cargo test 879 pass, clippy and nightly fmt clean, bundles rebuilt (not installed) and scripts/validate-plugin.sh all passed. User's check still pending.

The user's check in the plugin passed (2026-11-17): About's title bar matches the other windows; closing About over the logo plays no sound.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
About uses the default window frame (title bar as in Song Information). A press on any layer over the panel (window, menu, box) no longer reaches the panel: focus_click (row focus, logo startup sound) and status_bar (status line dismissal) check that the layer under the pointer is the panel's. Tests: a_click_on_a_window_over_the_logo_does_not_reach_the_panel and a_click_on_a_window_over_the_status_bar_keeps_the_status (crates/orb-ui/tests/menu.rs), both fail without their fix. Branch worktree-agent-aa0c8bc17e709fca0, one commit. cargo test 879 pass, clippy and nightly fmt clean, bundles validated. The user's check in the plugin passed 2026-11-17.
<!-- SECTION:FINAL_SUMMARY:END -->
