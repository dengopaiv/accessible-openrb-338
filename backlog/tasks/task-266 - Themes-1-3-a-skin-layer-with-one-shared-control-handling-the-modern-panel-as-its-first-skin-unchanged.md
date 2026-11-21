---
id: TASK-266
title: >-
  Themes 1/3: a skin layer with one shared control handling; the modern panel as
  its first skin, unchanged
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T10:05:00Z'
labels:
  - ui
milestone: m-8
dependencies:
  - TASK-258
references:
  - crates/orb-ui/src/app.rs
  - crates/orb-ui/src/layout.rs
  - crates/orb-ui/src/behavior.rs
  - crates/orb-ui/src/theme_view.rs
  - assets/ui/layout.tsv
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The user (2026-11-21) wants the mod's classic panel, today the read-only Theme Viewer of task-187, to become a real skin of the standalone's main window that takes input (task 2 of 3), with an animated transition (task 3 of 3). The rule from the user: **no duplicated control handling.** A change to how a control behaves must not have to be made once for the modern theme and once for the classic one. The two layouts must also be free to differ: the modern theme may place controls elsewhere and may have controls the classic one lacks.

This task builds the skin layer and moves the modern panel onto it with **no visible or behavioral change.**

**Today.**
- The modern panel: `assets/ui/layout.tsv` (about 350 rows: id, section, kind, part, rect in the original's 624 x 510 panel pixels), `layout.rs`, and `app.rs`. app.rs draws the SVG parts and runs all input: `control`, `hit`, `gesture`, `repeat`, `knob`, `fader`, `drum_strips`, `instrument_selection`, `focus_click`, `mod_view`, the keyboard.
- `behavior.rs` holds the original's interaction math, with geometry constants of the original (FADER_TRACK 54, FADER_TRAVEL 43, knob spans per device, the 808/909 step-strip and instrument hit maths, `layout::TR808_ORIGIN`/`TR909_ORIGIN`).
- `theme_view.rs` draws the classic panel from the mod's 58 bitmaps at the original's geometry, with its positions hard-coded in its `ops`.

**What to build.**
- **A skin:** the description of one look. It gives:
  - its panel size in its own units. The modern theme's aspect stays 624:510 (the user's assumption; nothing has to handle another aspect);
  - its controls: id (the same ids as now, so the model, the keys and the status line are shared), kind, and rect in skin units, plus the kind's geometry where it varies: fader track and travel, knob drag span, the drum strips' step and instrument hit areas, the row focus areas, the logo and the mod-splash areas;
  - a renderer that draws the panel state at a given screen rect.
  A skin may omit controls the other has; the controls' keyboard shortcuts keep working whatever the skin shows (user, 2026-11-21).
- **One controller** for both skins. It runs every bit of input on whatever rects the current skin gives:
  - hit tests, gestures, knob and fader drags with Shift fine mode, jump clicks, repeat buttons;
  - the step strips and instrument selection, row focus;
  - hover, cursor and status line;
  - the keyboard.
  `behavior.rs` takes the geometry from the skin instead of its constants. The numbers of the original stay as the classic skin's data, and the modern skin's data equals them today.
- **The modern skin:** `layout.tsv` plus the SVG renderer, moved onto the layer. Its layout table is its own: later edits to it must not touch the classic skin.
- The classic skin is **not** in this task (task 2). Only prove that the layer can take it, e.g. with a test skin whose rects differ and extra or missing controls, driven through the shared controller in tests.
- **Unchanged:**
  - the plugin editor (embed.rs) keeps working on the modern skin;
  - the desk (the metallic background) and the panel shadows are drawn around any skin (the user: they stay the same with classic).

**Testing.** Every existing orb-ui test passes unchanged in meaning, along with the input and render tests. Add tests that drive the controller through a skin with moved, extra and missing controls.

The user is the first reviewer: they try the standalone and the plugin by hand before the agent review. Give them a build (`scripts/macos-dev-app.sh`, and `cargo xtask bundle orb-plugin --release` for the plugin) and say what to try.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A skin type gives the panel size, the controls (shared ids, kind, rect and per-kind geometry) and a renderer; behavior.rs takes geometry from the skin, not constants
- [ ] #2 One controller handles all input for any skin; no control's input code exists twice
- [ ] #3 The modern panel runs on the layer with no visible or behavioral change, in the standalone and the plugin; existing orb-ui and orb-plugin tests pass
- [ ] #4 Tests drive the controller through a skin with moved, extra and missing controls
- [ ] #5 docs/ui_spec.md describes the skin layer; the user reviewed the build before the agent review
- [ ] #6 The modern skin is the generic SVG renderer plus one layout table plus one part set: no part or file name is hard-coded outside that set, and the colors of what the renderer draws itself (focus bar, shape-drawn lights and the like) come from the theme (user, 2026-11-21: another SVG-based theme should be easy to add)
- [ ] #7 A test builds a second SVG skin from a tiny dummy part set and layout table and drives and draws it through the shared controller and renderer
<!-- AC:END -->
