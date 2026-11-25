---
id: TASK-309
title: '808 mod window: show the standard splash when the mod has none'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T15:15:00Z'
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
- [ ] #1 A mod without Splash.jpg (or with one that does not decode) shows the embedded standard mod's splash in the 808's mod window, in the app and the plugin editor
- [ ] #2 Mods > Info still says 'This mod has no splash picture.' for such a mod
- [ ] #3 A test covers the fallback and the unchanged Info text; docs/ui_spec.md describes it
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #5 The user has checked it in the app
<!-- AC:END -->
