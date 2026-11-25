---
id: TASK-309
title: '808 mod window: show the standard splash when the mod has none'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T15:15:00Z'
updated_date: '2026-11-25T15:32:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-ui/src/mod_info.rs
  - docs/ui_spec.md
  - docs/rbm.md
priority: medium
type: enhancement
ordinal: 280700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-25. Some mods have no Splash.jpg. Today the 808's mod window (docs/ui_spec.md, around line 559: "the knob's place right of the legend shows the mod's window, the splash.jpg of the kit that plays", `App::mod_view`, the cached `App::splash`) has nothing to show for such a mod. When a mod has no splash, the 808's mod window should show the standard splash instead: the embedded standard mod's splash.jpg, the one the built-in kit shows (assets/Standard.rbm, `builtin_file`, docs/rbm.md 13).

For reference, the original falls back to its own exe resource 905, "the splash of a mod that has no Splash.jpg" (docs/rbm.md, the table near line 979; 0x417188). orb does not carry resource 905. The user chose the standard splash.

**Keep:** the Mods > Info window (crates/orb-ui/src/mod_info.rs, around line 363) still says "This mod has no splash picture." for such a mod. Only the 808's panel window falls back.

Check what happens with:
- the classic skin (ui_spec section 21), whether the selector knob replaces the window there;
- the plugin's editor;
- a mod whose Splash.jpg exists but fails to decode. Treat it like a missing one in the panel. The Info window keeps saying what it says today.

Update docs/ui_spec.md where it describes the mod's window.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A mod without Splash.jpg (or with one that does not decode) shows the embedded standard mod's splash in the 808's mod window, in the app and the plugin editor
- [x] #2 Mods > Info still says 'This mod has no splash picture.' for such a mod
- [x] #3 A test covers the fallback and the unchanged Info text; docs/ui_spec.md describes it
- [x] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #5 The user has checked it in the app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. App::panel_splash_texture: the kit's splash, else the embedded standard mod's (cached once); the panel's mod window uses it, Mods > Info keeps splash_texture. 2. App::splash_in_frame_size for tests. 3. Test in orb-ui/tests/menu.rs: mod without splash, with undecodable splash, with own splash. 4. docs/ui_spec.md. Classic skin has no modview control (selector knob) so is unaffected; the plugin editor uses the same App panel.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a6e50c783a96fed89

Evidence: App::panel_splash_texture (app.rs) falls back to the embedded standard splash (cached once); Info still uses splash_texture. Classic skin has no modview control, unchanged; the plugin editor is the same panel. Test the_mod_window_falls_back_to_the_standard_splash_and_info_does_not (orb-ui/tests/menu.rs) covers no splash, undecodable splash, own splash. cargo test --workspace 1180 pass, clippy macOS and xwin Windows clean, nightly fmt clean. Mod to try: ~/rb-mods/mods/AFX 1.rbm (no splash; 28 of 316 have none). AC 5 is the user's check.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 808's mod window shows the embedded standard mod's splash for a mod with no Splash.jpg or an undecodable one (App::panel_splash_texture); Mods > Info is unchanged. Test the_mod_window_falls_back_to_the_standard_splash_and_info_does_not; cargo test --workspace 1180 pass, clippy clean on macOS and Windows, nightly fmt clean. The user checked it in the dev app on 2026-11-25. Branch worktree-agent-a6e50c783a96fed89, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
