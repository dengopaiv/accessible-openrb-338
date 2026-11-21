---
id: TASK-266
title: >-
  Themes 1/3: a skin layer with one shared control handling; the modern panel as
  its first skin, unchanged
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T12:02:00Z'
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
- [x] #1 A skin type gives the panel size, the controls (shared ids, kind, rect and per-kind geometry) and a renderer; behavior.rs takes geometry from the skin, not constants
- [x] #2 One controller handles all input for any skin; no control's input code exists twice
- [x] #3 The modern panel runs on the layer with no visible or behavioral change, in the standalone and the plugin; existing orb-ui and orb-plugin tests pass
- [x] #4 Tests drive the controller through a skin with moved, extra and missing controls
- [x] #5 docs/ui_spec.md describes the skin layer; the user reviewed the build before the agent review
- [x] #6 The modern skin is the generic SVG renderer plus one layout table plus one part set: no part or file name is hard-coded outside that set, and the colors of what the renderer draws itself (focus bar, shape-drawn lights and the like) come from the theme (user, 2026-11-21: another SVG-based theme should be easy to add)
- [x] #7 A test builds a second SVG skin from a tiny dummy part set and layout table and drives and draws it through the shared controller and renderer
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Part sets (art.rs): PartSet = the embedded parts or a set built at run time from SVG files (viewBox parsed as build.rs does); every part painter takes a set; art::part stays the embedded set's.
2. The layout table format (assets/ui/layout.tsv, written by orb-ui-art): a 9th optional column of key=value params, and rows of new kinds besides the controls: panel (the size), bg (with plugin= variant), logo (drawn and the click area), focus (bar, on=), fg, row (focus click areas), strip (drum step strips, cells=), select (instrument hit areas, cells=x0:w:first segments, count=, open=), mark (selection marks, lit=/litsel=, slot=, upper=). Controls carry their geometry as params: knob span=, fader handle=, meter map= or on=/off=, modview glass=/picture=/name_top=/font=/ink=. The generator emits them; layout::controls() keeps returning only the 350 controls.
3. skin/mod.rs: trait Skin { layout() -> &Layout (size, controls, strips, selects, rows, logo: all in skin units), paint(ui, view, &Frame) }; Frame = the panel state to draw (panel, pressed per control, now, note-mode lights, plugin, hidden, logo time, mod name, splash); the mark state (selected/lit) shared by all skins.
4. skin/svg.rs: SvgSkin = the generic renderer over one table + one part set; modern() = layout.tsv + the embedded set. All part naming (family + state, params overrides, _pressed variants found in the set) moves from model::part_name to the skin; the model gives a Look per control (OnOff, Push, Lit, Value, Meter byte, Knob, Fader, Digits, Plain).
5. controller.rs: one controller (impl App) running all input on the current skin's layout: keyboard and holds, row focus and logo click, hit tests and gestures per kind (buttons, repeat/wind, knobs with Shift fine, faders with jump, double click), the drum strips and instrument selection, the plugin's drag-out; returns the pressed flags the renderer shows. Same order of egui interact calls as before; all input runs before painting.
6. behavior.rs: geometry from the skin (knob span, Fader {track, handle}, strip cells, select segments) instead of constants; the original's numbers are the modern table's (and theme_view's) data.
7. app.rs: App holds Arc<dyn Skin> (modern by default, set_skin for tests and task-267); View fits the skin's size; the desk and shadows are drawn around the skin's rect in panel proportions; the plugin overlay stays in embed.rs over the modern skin.
8. Tests: existing tests unchanged in meaning (part_name call sites move to the skin); new tests drive the controller through a skin with moved, extra and missing controls, and through a second SVG skin from a tiny dummy part set and table (drive and draw).
9. Docs: docs/ui_spec.md new section on the skin layer, assets/ui/README.md table format; build the dev app and plugin bundles for the user's review.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-adbc943ce6bdae5e3 (rebased onto main 4ad0ac6 before starting).

Design as built:
- skin/mod.rs: trait Skin { layout() -> &Layout; paint(ui, view, &Frame) } (Send + Sync, held by App as Arc<dyn Skin>; App::set_skin, App::skin). Layout = size + controls + strips + selects + rows + logo, all in skin units, built from a layout table. Frame = panel, pressed per control, now, note-mode Lit, plugin, hidden, logo time, mod name, splash. skin::mark() computes a drum mark's selected/lit state for any skin.
- skin/table.rs: the table format (id section kind part x y w h [params]); Ctl moved here from layout.rs (layout re-exports it) with params and the geometry accessors knob_span(), fader(), cell(), segments(), select_area(), row().
- skin/svg.rs: SvgSkin::new(table, Arc<PartSet>) = the generic renderer; modern() = layout.tsv + PartSet::embedded(). part_name() moved here from the model; Panel::look() gives a model::Look (Plain, OnOff, Push, Lit, Value, Meter byte); Panel::part_name stays as a one-line delegate to the modern skin (the tests use it).
- controller.rs: the one controller (impl App): keyboard + holds, focus/logo click, per-kind input, repeat, knob, fader, drum strips, instrument selection; returns the pressed flags; egui ids carry (kind, id, place in the layout) so a skin may show a control twice. Input runs before the skin paints; the order of ui.interact calls is unchanged (controls in table order, strips, selects, then the plugin overlay).
- behavior.rs: knob_span and FADER_TRACK/HANDLE/TRAVEL/K and tr808_hit/tr909_hit/tr808_step/tr909_step removed; Fader {track, handle} (travel, k = 10000/travel), select_hit(areas) with Segment/SelectArea, strip_step(x0, cell, x); digit_frames gives Option<u8> digits (no part names).
- art.rs: PartSet (embedded, or from_svgs reading each viewBox as build.rs does; names and bytes leaked once per set). app::paint_part_in(ui, set, ...) paints from a set; logo_anim::paint_logo_in(set, base) animates when the set has <base>_open.
- orb-ui-art: the generator writes the table's geometry rows (panel, bg in the app's old drawing order with plugin=bg_transport_plugin, logo, focus, row, fg, strip, select, mark) and the controls' params (knob span by section, fader handle=11, header meter map, gr on/off, the mod window's glass/picture/name/font/ink). layout.tsv regenerated (cargo run --release -p orb-ui-art -- gen): 289 lines changed, every SVG byte-identical.
- backdrop::paint(ui, avail, view, skin size): the desk and shadows in panel proportions around any skin; View::fit_size for a skin's size.

Evidence of no change:
- Pixel comparison against main 4ad0ac6 (built in a scratch worktree, CARGO_TARGET_DIR outside the repo): OpenRB-338 --demo --screenshot with ORB_LOGO_T=10 at --size 624x510 --ppp 1 and 2, 1000x600 ppp 2, 700x900 ppp 1, 1300x1062 ppp 1.5, and tests/songs/shipped/G-Spot.rbs at 900x700: 0 pixels differ in every pair (script shots.sh in the scratchpad: decodes both PNGs and counts differing pixels).
- behavior::tests::the_modern_skin_hits_as_the_constants_did: the old constant functions (kept in the test as before::*) against the table-driven select_hit/strip_step at every quarter pixel of both drum machine sections and 100 px around them (about 1.6 million points each): equal everywhere, including the 909's rows taking everything left of section x 26 as AC (open=1).
- Failed first try: the moved skin test placed transport.play twice and the second placement did not work: both shared the egui ids (hit, kind, id) and the gesture state. Fixed by keying by the place in the layout (Placed {c, n}).

Checks: cargo test (workspace) 990 passed, 0 failed, 5 ignored; cargo clippy --all-targets -- -D warnings clean; cargo +nightly fmt --all --check clean. New tests: tests/skin.rs (2), skin::table, skin::mark, skin::svg (the modern table names only parts of its set), layout (focus areas, logo, marks against the art generator), behavior (the sweep).
Builds for the user's review: target/OpenRB-338.app (scripts/macos-dev-app.sh, version 2026.1121.1115), target/bundled/OpenRB-338.vst3 and .clap (cargo xtask bundle orb-plugin --release).
Open point for the user (not changed): the plugin's overlay (Sync/MIDI LEDs, Bar display, mode selector, format logo, resize grip; embed.rs) still names embedded parts in code: it is plugin chrome over the modern skin, the only skin the plugin has (task-267). The About window's art likewise.

User review (2026-11-21, relayed by the supervisor): the user tried the 2bb014f builds (the dev app target/OpenRB-338.app and the plugin bundles, rebuilt from 2bb014f and installed by the supervisor) in a DAW: "looks good". No code change after the review; main still 14f4aff, no rebase needed.

User, 2026-11-21 (after merge): the plugin overlay (embed.rs), the About window and the desk tile naming their SVG parts in code is accepted as is for now; moving them into a theme's data is to be handled when a new SVG theme is added.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The main window's panel now runs on a skin layer with one shared controller; the modern panel is its first skin, visibly and behaviorally unchanged.
- skin/ (trait Skin: a Layout in skin units plus a renderer; Frame; shared drum mark state), skin/table.rs (the layout table format), skin/svg.rs (SvgSkin: the generic SVG renderer over one table and one part set; modern() = assets/ui/layout.tsv + the embedded parts), controller.rs (all panel input, once, on any skin's layout), behavior.rs on skin geometry (Fader, select_hit, strip_step), art::PartSet (embedded or from SVG files), model::Look (the model names no parts). orb-ui-art writes the table's geometry rows and parameters; every SVG byte-identical.
- Evidence: screenshots of the demo panel at five sizes and densities and of a shipped song are pixel-identical to main's; the table-driven hit maths equal the old constants at every quarter pixel of both drum machines; tests/skin.rs drives a skin with moved, doubled, missing and unknown controls and a dummy SVG skin (table + part set only) through the same controller and renderer.
- cargo test 990 pass, 0 fail, 5 ignored; clippy -D warnings and nightly fmt clean. The user reviewed the standalone and plugin builds (2026-11-21): looks good.
- Branch worktree-agent-adbc943ce6bdae5e3, one commit on main 14f4aff. Docs: docs/ui_spec.md section 22, assets/ui/README.md (the table format).
- Open point noted for the user: the plugin overlay (embed.rs) and the About window still name embedded parts in code (chrome over the modern skin, the plugin's only skin).
<!-- SECTION:FINAL_SUMMARY:END -->
