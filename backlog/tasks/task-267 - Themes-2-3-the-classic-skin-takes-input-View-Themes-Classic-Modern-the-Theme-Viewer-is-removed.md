---
id: TASK-267
title: >-
  Themes 2/3: the classic skin takes input; View > Themes > Classic / Modern;
  the Theme Viewer is removed
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T10:08:00Z'
labels:
  - ui
milestone: m-8
dependencies:
  - TASK-266
references:
  - crates/orb-ui/src/theme_view.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/settings.rs
  - crates/orb-ui/src/key_help.rs
  - docs/ui_spec.md
  - docs/rbm.md
priority: medium
type: feature
ordinal: 2000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Task 2 of the user's themes request (2026-11-21, milestone m-8). It depends on task 1, the skin layer with one shared controller.

**What to build.**
- **The classic skin.** It draws the panel from the current kit's 58 bitmaps, as `theme_view.rs` does now (docs/ui_spec.md section 21, docs/rbm.md 13.1), at the original's geometry.
  - Its controls are a table of the original's rects and per-kind geometry; theme_view.rs's hard-coded positions and docs/ui_spec.md sections 1-9 hold them.
  - It takes all input through task 1's shared controller. No input code is specific to it.
  - Where the original's hit area differs from the drawn bitmap, check docs/ui_spec.md section 2 and the original. Use the user's IDA databases only through copies, never saved.
  - Scaling is **always nearest** (user, 2026-11-21): no bilinear option and no key for it.
  - The desk (the metallic background) and the panel's shadows stay as in the modern theme (user, 2026-11-21).
  - The panel keeps the modern theme's aspect, 624:510, so switching changes nothing outside the panel.
- **The menu.** A new **View** menu with **Themes > Classic** and **Themes > Modern**, a radio choice. Modern is the default.
  - Kept in the UI settings (ui.toml).
  - Standalone only: the plugin has neither the menu nor the classic skin.
  - With no mod loaded, or a kit without its panel bitmaps, Classic draws the built-in Standard kit's bitmaps, which assets/Standard.rbm has.
- **Removed:**
  - the Theme Viewer: Mods > Theme Viewer, its window and viewport, the Space filter key and the `theme_viewer_filter` setting;
  - the main window's mod-splash click that opened it, and its hand cursor;
  - their tests, Help > Keyboard Shortcuts rows and docs.
  An old ui.toml with `theme_viewer_filter` must still load.
- **Docs.** docs/ui_spec.md (section 21 rewritten for the skin), the menu docs and Help, and docs/plugin.md if it mentions the viewer.

**Testing.**
- The classic skin's hit areas are tested against the original's rects.
- The same input sequence gives the same panel state in both skins: one test per control kind.
- The settings round-trip works, and an old settings file loads.

The user is the first reviewer: give them a dev app (`scripts/macos-dev-app.sh`) and a list of what to try per section, before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The classic skin draws from the kit's bitmaps with nearest scaling and takes all input through the shared controller, with no classic-specific input code
- [ ] #2 View > Themes > Classic / Modern (standalone only, default Modern, saved in ui.toml) switches the main window's skin; the desk and shadows stay
- [ ] #3 The Theme Viewer window, its menu item, the splash click, the Space filter key and theme_viewer_filter are removed; an old ui.toml still loads
- [ ] #4 Tests: classic hit areas against the original's rects, the same input giving the same state in both skins, the setting round-trip
- [ ] #5 Docs updated (ui_spec section 21, menus, Help); the user reviewed the build before the agent review
- [ ] #6 The classic composer is scale-ready (user, 2026-11-21: hi-res classic sets in the same format): it composes at an integer scale N found from the bitmap sizes (the background's size against its 1x size, every other bitmap checked to match, a clear error otherwise); strip frames, digit widths, LED offsets and positions scale by N while the control rects and hit areas stay in 624x510 panel units
- [ ] #7 A test composes a 2x set (the Standard bitmaps upscaled by nearest) and checks it equals the 1x compose upscaled 2x; where hi-res sets come from (mod resource, sidecar folder, theme file) is left to a later decision and noted in docs/ui_spec.md
<!-- AC:END -->
