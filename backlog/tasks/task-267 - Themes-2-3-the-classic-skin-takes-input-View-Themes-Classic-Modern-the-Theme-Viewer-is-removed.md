---
id: TASK-267
title: >-
  Themes 2/3: the classic skin takes input; View > Classic / Modern; the Theme
  Viewer is removed
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T10:03:00Z'
updated_date: '2026-11-21T12:53:00Z'
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
- **The menu.** A new **View** menu with **Classic** and **Modern**, two radio items directly in View (the user's review of 2026-11-21: no Themes submenu; first planned as View > Themes > Classic / Modern). Modern is the default.
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
- [x] #1 The classic skin draws from the kit's bitmaps with nearest scaling and takes all input through the shared controller, with no classic-specific input code
- [x] #2 View > Classic / Modern, two radio items directly in View (the user's change of 2026-11-21: no Themes submenu; standalone only, default Modern, saved in ui.toml) switches the main window's skin; the desk and shadows stay
- [x] #3 The Theme Viewer window, its menu item, the splash click, the Space filter key and theme_viewer_filter are removed; an old ui.toml still loads
- [x] #4 Tests: classic hit areas against the original's rects, the same input giving the same state in both skins, the setting round-trip
- [x] #5 Docs updated (ui_spec section 21, menus, Help); the user reviewed the build before the agent review
- [x] #6 The classic composer is scale-ready (user, 2026-11-21: hi-res classic sets in the same format): it composes at an integer scale N found from the bitmap sizes (the background's size against its 1x size, every other bitmap checked to match, a clear error otherwise); strip frames, digit widths, LED offsets and positions scale by N while the control rects and hit areas stay in 624x510 panel units
- [x] #7 A test composes a 2x set (the Standard bitmaps upscaled by nearest) and checks it equals the 1x compose upscaled 2x; where hi-res sets come from (mod resource, sidecar folder, theme file) is left to a later decision and noted in docs/ui_spec.md
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Raster source skin/bitmaps.rs (from theme_view.rs, no input code): Bitmaps of a kit (each missing or broken bitmap falls back to the built-in one), the set's integer scale N (12057's size over 592 x 50; every other bitmap a multiple of N; fallbacks upscaled by N; a clear error otherwise, the built-in set drawn instead), Op and compose at N (positions, frame cells, digit pitch, LED offsets scaled by N; the original's geometry in 624 x 510 units).
2. Classic layout table assets/ui/classic.tsv in the skin table format: the original's view rects (ui_spec sections 3-9, the viewer's places, the IDA-read strips, selection areas and PCF faders), part = bitmap id, params n, v, frame, frames, src, digits, cap, of/base (new geometry kind pick for the grid and bank strips), mark rows for the 909 LED, the 808 selector as a knob (tr808.selector, span 40; the model maps it to inst808).
3. skin/classic.rs ClassicSkin: Skin over the table and a Bitmaps set: ops from the table and Panel::look, composed when they change into one texture, drawn with NEAREST. scale() for task-268.
4. Controller: remove the modview click and hand cursor (the Theme Viewer link); egui ids of strips and selects keyed by place.
5. Settings: theme = "modern" | "classic" in ui.toml (default modern); theme_viewer_filter dropped (an old file still loads). View > Themes > Classic / Modern radio (standalone only); App keeps the classic skin per kit (rebuilt on a kit change, built-in bitmaps for no mod).
6. Remove theme_view.rs, Mods > Theme Viewer, Cmd::ThemeViewer, the viewer's Help row and tests; plugin test updated.
7. Tests: classic hit areas against the original's (IDA) rects and maths; same input same state in both skins per control kind; settings round-trip and old file; 2x compose equals 1x upscaled; classic compose equals the old viewer's ops for many states; third-party mods compose.
8. Docs: ui_spec section 21 rewritten (classic skin), 17/18 menus and settings, Help; plugin.md; a Decide task for where hi-res sets come from.
9. cargo test, clippy, nightly fmt; dev app; user review list.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a996f643067ca61f5

IDA (a copy of Rebirth.exe.i64 in the scratchpad, closed with save=false), the original's hit areas the classic table takes:
- TR-808 step view: layout sub_442CDA ends sub_47E04C(16, 91, 256, 17) (section coords), step = (x - view left) / 16 in sub_44111F after PtInRect (sub_479AA2): panel strip 106 371 256 17, 16 cells of 16 [C]. The modern skin's strip is 104 370 256 20 (old constants x - 14): its own.
- TR-909 step view: sub_446E4E ends sub_47E04C(29, 86, 336, 16); sub_444FB3 step = (x - left) / 21: panel 119 481 336 16 [C]. Modern: 116 480 336 18 (x - 26).
- 808 selection sub_4433EE: legend rect (10, 71)..(366, 85), index (x - 10) / 22 < 12; upper labels (78, 43)..(184, 55), (x - 78) / 22 + 3. Same as the modern rows [C].
- 909 selection sub_447500: legend (4, 4)..(366, 18), name row (26, 70)..(366, 85), dx < 26 AC, < 236 (dx - 26) / 42 + 1, else (dx - 236) / 21 + 6, > 11 none; plus the AC box (4, 44)..(26, 62) giving AC. The modern rows (legend open to the left, names y 74..84, no AC box) differ.
- Faders: CBitmapFaderView ctor sub_469240(section, handle, track) sizes the view handle-wide and TRACK-high (sub_46E2BE on both bitmaps); the PCF ctor sub_41B390 passes [+0x15C] = 12031 (handle) and [+0x164] = 12030 (track), the mixer sub_413B55 [+0x160] = 12032 (CReBirthBitmaps sub_4287A0: +87 dwords 12031, +88 12032, +89 12030). The mouse-down 0x469708 takes k = 10000 / (view height - handle height). So the PCF faders are 15 x 45 with travel 34, k = 294 [C], not 54/43 (settles the [G] of ui_spec section 21).
- Pattern section layout sub_41A9A0: grid (1, 19, 68, 40), bank row (1, 58, 68, 30) [C]; grid hit sub_41A5B0 (vtable 0x4BB6FC + 92) col = 4 dx / 68, row = 2 dy / 40 (the non-obfuscated branch) [C]; the bank row's hit was not read [G: 4 cells of 17].

Design as built:
- skin/bitmaps.rs (raster source, no input code; from theme_view.rs): Bitmaps (per-bitmap fallback to the built-in Standard kit), find_scale, upscale, Op, compose at N, knob_frame, digits, handle_top(p, track, handle).
- assets/ui/classic.tsv (hand-written, skin table format): the original's view rects; part = bitmap id; params n, v, frame, frames (state to frame, - none), src, digits, pitch, cap; kind pick (new geometry kind in table.rs: a strip whose frame is base + the first on control of of=) for the grid and bank strips, whose buttons are input-only rows (part -); 909 LED as 12 mark rows; gain-reduction LEDs as nine 7 x 10 parts of 12570.
- skin/classic.rs ClassicSkin: Skin over the table and a Bitmaps set; ops() from Panel::look, composed only when the op list changes, one texture with TextureOptions::NEAREST; scale() for task-268; compose_panel() for tests.
- model: SELECTOR_808 = tr808.selector, the original's 808 instrument selector knob (span 40), shown by the classic skin only: knob() = knob_from_engine((127 i + 5) / 11), set_knob selects (11 v + 63) / 127 and sends nothing.
- controller: modview takes no input (the Theme Viewer link and its hand cursor are gone); strips' and selects' egui ids keyed by place (review notes 1 and 2 of task-266).
- settings: theme = modern | classic (Theme enum), theme_viewer_filter ignored on read and dropped on save. App::set_theme, App::theme (Modern in a plugin), App::apply_theme (classic skin per kit, rebuilt on a kit change, an unusable set reported once in the error box). Menu: View > Themes > Classic / Modern radio (standalone only), Cmd::SetTheme; Cmd::ThemeViewer and Mods > Theme Viewer removed; key_help's Space row removed.
- Removed: theme_view.rs, tests/theme_view.rs.

Failed hypothesis: an exact scale rule (12057 exactly 592N x 50N) refused 51 of the 320 third-party mods (1x backgrounds of 608 x 50, 600 x 50, 609 x 50, 593 x 51, 592 x 90). Changed to N = round(width / 592), at least 1, with every bitmap's sides multiples of N; all 320 compose at 1x (ORB_THIRDPARTY_MODS_DIR=~/rb-mods/mods cargo test -p orb-ui --test classic).

Evidence:
- classic draws what the viewer drew: the old theme_view::ops kept in tests/classic.rs as the reference; the classic compose equals it pixel for pixel for Panel::new and 40 random states.
- 2x: the Standard bitmaps upscaled by nearest compose exactly the 1x image upscaled 2x (4 states).
- hit areas: every whole pixel of both drum machine sections and 30 around against the IDA-read strip and selection maths; grid cells against behavior::pattern_grid_button; PCF faders 45/11/34/294; knob spans.
- same input same state in both skins: 26 control probes (trigger, toggle, radio, switch, hswitch, vswitch, repeat, wind, grid pattern and bank, knobs of 4 spans, faders), both strips, both selections, the row focus; each probe checked to change the state.
- Known, not changed: the modern skin's PCF faders keep track 54 (k 232) where the original's are 45 (k 294); the modern table is its own.

Checks: cargo test (workspace) 991 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. With ORB_THIRDPARTY_MODS_DIR=~/rb-mods/mods: tests/classic.rs 17 pass, 320 mods compose at 1x.
Screenshot: ORB_UI_CONFIG with theme = "classic", OpenRB-338 --demo --size 1000x700 --screenshot: the classic panel inside the modern desk and shadows, the View menu in the bar.
Dev app for the user's review: target/OpenRB-338.app (scripts/macos-dev-app.sh, version 2026.1121.1213).
Decide task created: task-271 (where hi-res classic sets come from), not blocking this task.
AC 5 (docs and the user's review): docs done (ui_spec 6, 9, 17, 18, 21, 20; plugin.md; building.md; assets/ui/README.md); waiting for the user's review.

User review round 1 (2026-11-21, relayed by the supervisor): flatten the menu to View > Classic and View > Modern, two radio items directly in View, no Themes submenu. Done in menu.rs (no submenu), the doc comments, docs/ui_spec.md (sections 17, 18, 21), docs/plugin.md, task-267's title, description and criterion 2, task-268's and task-271's descriptions. New test tests/menu.rs the_view_menu_has_classic_and_modern_directly (View between Edit and Mods, Classic and Modern drawn, no Themes). Added coverage in tests/interaction.rs: a knob drag with Shift held from before the press (one ModifiersChanged and the ShiftLeft key, as egui-winit sends them) is 5 times finer through the controller; it passes. The earlier mentions of View > Themes in these notes and the plan are the history before this change.

Checks after review round 1: cargo test 993 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt clean.

User review (2026-11-21, relayed by the supervisor): the user tried the 8c5365a dev app (target/OpenRB-338.app, 2026.1121.1241): "looks fine". Earlier in the review they found Shift fine knob adjustment working, and asked for the flat View menu (review round 1, done). No code change after the review.

Agent review round 1 (send back), fixed:
1. task-268's file restored to main's text exactly (the supervisor updates its menu name).
2. task-271's description: stray quotes removed; its scale rule updated (below).
3. task-267's description: the menu bullet unquoted.
4. Theme Viewer wording in code: link.rs (the meters' comment) and model.rs (Panel::meters) now name the modern and the classic skin.
5. tests/classic.rs: view_themes_switches_and_is_kept renamed view_menu_switches_and_is_kept.
6. docs/ui_spec.md section 17's settings sentence rewrapped under 100 columns; docs/building.md: the WSLg note says the second window was the Theme Viewer, which task-267 removed, so the app no longer opens two windows. The task file's name: the backlog CLI (1.x here) has no rename and keeps the file name when the title changes (edit -t), and backlog/ is not edited by hand, so the file keeps its old name; left to the supervisor.
7. The commit body rewrapped at 72 columns.
8. The scale rule tightened: N = round(w / 592) only when it equals round(h / 50), else 1 (then every bitmap's sides must be multiples of N). An 888 x 50 or 1184 x 52 background with even bitmaps is now 1x (unit test). All 320 third-party mods still read as 1x (ORB_THIRDPARTY_MODS_DIR=~/rb-mods/mods cargo test -p orb-ui --test classic: 320 mods composed). The remaining limit, a 1x background about N times larger both ways with every side a multiple of N, is documented in docs/ui_spec.md section 21 and in find_scale's comment.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The classic skin takes input and is chosen in View > Classic / Modern; the Theme Viewer is gone.
- skin/bitmaps.rs is the raster source: a kit's 58 bitmaps with a per-bitmap fallback to the built-in Standard ones. The composer is scale-ready: N is the 12057 background's width over 592 and height over 50, rounded, when the two agree (else 1); positions, frame cells, digit pitch and offsets are multiplied by N, and hit areas stay in 624x510 units. The sides are rounded because 51 of the user's 320 mods have 1x backgrounds that are not 592x50; all 320 read as 1x. The remaining limit (a 1x background about N times larger both ways) is documented.
- assets/ui/classic.tsv holds the original's view rects and the hit areas read in IDA: the 808 and 909 step views, the 909 selection with its AC box, and the PCF faders 45 high (k 294, which settles a [G]). skin/classic.rs draws from it with nearest scaling, as one texture recomposed only on a change, with no input code. The original's 808 selector knob is a classic-only control (tr808.selector).
- The controller: the mod window has no click action, and strips and selects are keyed by their place in the layout. Settings: `theme` in ui.toml (an old theme_viewer_filter file still loads). View > Classic / Modern are two radio items, standalone only. Removed: theme_view.rs, Mods > Theme Viewer, the Space key and its setting.
- Evidence (tests/classic.rs): the classic panel equals the old viewer's pixel for pixel for 41 states; a 2x set composes the 1x image upscaled 2x; the hit areas match the IDA maths at every pixel; the same input gives the same panel state in both skins; all 320 third-party mods compose.
- cargo test passes (count in the notes and the commit); clippy and nightly fmt clean. The user reviewed the dev app (2026-11-21): looks fine. Agent review round 1 fixed (notes).
- Branch worktree-agent-a996f643067ca61f5, one commit on main 7825b97. Decide task-271 opened (where hi-res sets come from), not blocking.
<!-- SECTION:FINAL_SUMMARY:END -->
