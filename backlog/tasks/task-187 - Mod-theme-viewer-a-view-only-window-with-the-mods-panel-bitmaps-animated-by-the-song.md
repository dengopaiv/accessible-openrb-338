---
id: TASK-187
title: >-
  Mod theme viewer: a view-only window with the mod's panel bitmaps, animated by
  the song
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-14T16:46:00Z'
updated_date: '2026-11-19T12:36:00Z'
labels:
  - ui
  - re
  - docs
milestone: m-5
dependencies: []
references:
  - docs/rbm.md
  - docs/ui_spec.md
  - crates/rbm/src/standard.rs
  - crates/orb-ui/src/menu.rs
  - crates/orb-ui/src/mod_info.rs
  - task-078
  - task-130
priority: medium
type: feature
ordinal: 203000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Requested by the user on 2026-11-14. A mod (`.rbm`) carries a theme: 58 numbered panel bitmaps (`12015.jpg` .. `12571.jpg`, the exe's JPEG resources of the same ids; a missing one falls back to the built-in bitmap, docs/rbm.md 13.1, loader `sub_41703A`). The original redraws its whole song window with them. task-078 (2026-10-27) decided that OpenRB-338's main panel keeps its own SVG art and a mod changes only the sound. That stays. This task adds a separate way to see a mod's look: a viewer window that draws the panel as the original would with the current mod's bitmaps, and moves with the song (knobs turn, LEDs and step lights run, displays count) as the main panel does.

The viewer is view only. It ignores every input that would change something: no click, drag, scroll or key on it changes the song, the transport or the main panel, and no control in it reacts (no pressed look, no hover). Over the panel area the mouse cursor is the "not allowed" (stop) cursor (egui `CursorIcon::NotAllowed`), so it is plain that nothing there is clickable. Nothing is shown in a status bar or as a notification. The window itself (moving, resizing, closing) works as usual. (Changed by the user on 2026-11-19; earlier text asked for a notification on click.) The one key it takes (the user's review of 2026-11-19): Space, pressed while the viewer window has the focus, switches the viewer's scaling between nearest pixel (the default with no saved choice, the user's review round 4) and bilinear; the choice is kept in the UI settings across runs, and the key never reaches the main panel (no play or stop). The letterbox around the panel is black.

The viewer is the standalone app's only (the user's review of 2026-11-19): a plugin's Mods menu has no Theme Viewer item. Choosing the menu item while the viewer is open brings its window to the front with the focus. In the standalone app the 808's mod window on the main panel (the kit's splash) is a link to the viewer: the pointing-hand cursor over it, and a click opens the viewer or brings the open one to the front; in a plugin it does neither.

The user reviews the viewer in the app before the agent review.

What is known: the panel's geometry, widget classes and film-strip frame formulas are in docs/ui_spec.md (sections 1 to 9: e.g. knob frame `((n - 1) * p + 500) / 1000` of an n-frame strip, the fader's handle 12031 on track 12032, the seven-segment digits 12015 with 11 frames). Only a few bitmap ids are mapped to their controls so far. The rest of the mapping (which id is which section background, strip or button, its frame count and where it sits) is in Rebirth.exe's panel construction (e.g. the transport's layout `sub_4298B9` and ctor `sub_4291E9`, and the other sections' layout functions) and needs reading in IDA (read only). The built-in kit's bitmaps are in the embedded `assets/Standard.rbm` (`rbm::standard::builtin_file`), so the viewer works with no mod and the tests need no file of the original. Mod files stay out of git; third-party mods may have bitmaps of unexpected size or broken JPEGs and must not crash the app.

In the standalone app the window should be a native window (an egui viewport).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/ui_spec.md gains a table of all 58 panel bitmap ids: what each draws (section background, film strip, button, display), its frame count and layout, and its place on the panel, each [C] with addresses or [G]
- [x] #2 A menu item (in the Mods menu) opens the viewer in its own native window in the standalone app; it can be closed and reopened, and it is resizable with the panel's aspect kept
- [x] #3 The viewer draws the current mod's bitmaps, falling back bitmap by bitmap to the built-in ones, and redraws with the new mod's when the mod or the song changes
- [x] #4 Every control of the panel follows the live state as the main panel does: knobs, faders, switches, toggles, LEDs, step and running lights, the pattern grid, the seven-segment displays and the meters
- [x] #5 Tests with the embedded standard mod check the composed frame for known states (knob frames by the ui_spec formula, a lit LED, a display value), with no MIDI, audio device or file of the original
- [x] #6 Every mod of the user's third-party collection (ORB_THIRDPARTY_MODS_DIR) opens in the viewer without a panic, checked by a gated test that skips without the variable
- [x] #7 A side-by-side check of the viewer against a screenshot of the original with the same mod and song is recorded in the notes, with any difference explained
- [x] #8 No input to the viewer changes the song, the transport or the main panel and no control in it reacts; the cursor over the panel is the not-allowed (stop) cursor; nothing appears in a status bar or as a notification; the one exception is Space in the viewer window, which switches its scaling (bilinear / nearest, kept in the UI settings) and starts no playback; tests check all of these
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Map the 58 bitmaps: sizes from the embedded Standard.rbm, frame layouts from the CBitmapSequence rule (fw = (w + 1) / n - 1), places by template matching every frame against the original's screenshot docs/ui_spec/panel.png, controls from docs/ui_spec.md sections 1-9. The idalib MCP tools are not loaded in this session, so code addresses come only from what docs/ui_spec.md already cites; the rest is [C: template match] or [G]. Write the table into docs/ui_spec.md (new section 21).
2. orb-ui: new module theme_view.rs. ThemeBitmaps: the 58 JPEGs of the kit decoded (image crate), each falling back to the embedded standard mod's on a missing file, a decode error or a size other than the built-in one; never panics. A static table of every drawn element (bitmap, frame cell, place) in the original's geometry; compose(bitmaps, panel, lights) -> 624x510 ColorImage by blitting frames chosen from the live Panel (knob frame ((n-1)p+500)/1000, fader handle, toggles, LEDs, displays right aligned with blank leading zeros, meters, grids, step LEDs with the running light, 808 selector). The frame list is cached so the texture is rebuilt only when it changes.
3. App: Mods > Theme Viewer... opens an immediate egui viewport (native window; embedded in a plugin) with the panel letterboxed at its aspect; closed by its close button, reopened by the menu. The viewer allocates its area with Sense::hover only, sets CursorIcon::NotAllowed over the panel, and takes no input; nothing goes to the status line.
4. Tests (orb-ui/tests/theme_view.rs): composed frames of the embedded kit against the bitmaps' frames (knob frame by formula, lit LED, display digits, fallback for a broken JPEG); a headless App run with the viewer open: clicks, drags, scroll and keys over the viewer change neither the panel nor the events nor the status, and the cursor is NotAllowed; the gated ORB_THIRDPARTY_MODS_DIR test composes every mod without a panic.
5. cargo test, clippy, nightly fmt; build the dev app (scripts/macos-dev-app.sh) for the user's review; AC 7 (side by side with the original running) is left to the user / test machine if it cannot be done here.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a6ffc02acfb4902f5

Research (2026-11-19):
- The idalib MCP tools are not loaded in this session (ToolSearch for ida/idalib finds nothing), so no new code was read in Rebirth.exe. The bitmap table is built from (a) the sizes of the 58 JPEGs of the embedded assets/Standard.rbm (orb mod unpack), (b) the control rectangles of docs/ui_spec.md sections 3-9, which carry their addresses, (c) template matching of every frame of every strip against the original's screenshot docs/ui_spec/panel.png at those places (+-3 px, mean abs RGB difference). Scripts: scratchpad wt187/match.py (full search) and check.py (places), run with a venv numpy.
- Found: 12015's frame 0 is blank and frame d+1 the digit d (the first viewer drew tempo 140 as '03' and the disabled bar as '999'; fixed; ui_spec section 2 corrected). The five transport buttons are 12559 Play, 12560 Stop, 12561 Rewind, 12562 FF, 12563 Record (each matches only its own place, 2.4-3.1). 12017 is the PCF mode / delay triplet switch (9x18). 12505 = white-key, Down/Up/Accent/Slide, Note, Rest LEDs; 12506 = black-key and pitch LEDs. 12521 red level knobs, 12522 white. 12554 grid is vertical, 9 cells of 56 (frame 0 none lit); 12555 bank row vertical, 4 cells of 46 (no unlit frame). 12557 master meter vertical, 11 cells of 72. 12570 GR LEDs: two rows (off, lit) of 9 LEDs x 7 px.
- Not placed: 12030 (15x45, best 14.4) and 12567 (6x6, best 10.7) match no place; [G].
- Failed hypothesis: the lit on/off LED buttons (12550 mixer, 12556 pattern, 12066 effects) are frame 1 of their strips. The screenshot shows a pale square in a dotted ring instead (mean difference 21-29 at the right place, frame 0 matches unlit ones). Left [G] in ui_spec section 21; the viewer draws frame 1.
- Third-party collection (ORB_THIRDPARTY_MODS_DIR=~/rb-mods/mods, 320 mods): all compose without a panic; 91 have built-in bitmaps for some ids, every reason 'missing' (sample-only mods or partial skins), no broken JPEG.

Partial side by side (AC 7 needs more): the composed built-in panel at orb's new-song state against panel.png (the original, standard kit, the machine's template song): mean difference 3.8 of 255, 4865 of 318240 pixels differ by more than 40. Every difference is the song state (knob and fader positions, the template's lit 808 step buttons, the PCF pattern 25, meters) or the lit on/off buttons above. A check with the same song and a third-party mod needs the original running.

Acceptance criteria (2026-11-19):
- AC 1: docs/ui_spec.md section 21, all 58 ids with size, role, frames and layout, place, each [C: match d] (template match against panel.png) or [G] (12030, 12567 unplaced; the lit on/off look of 12550/12556/12066). No new code addresses: IDA was not available (above).
- AC 2: Mods > Theme Viewer... (menu.rs, Cmd::ThemeViewer) -> App::theme_viewer_ui: egui immediate viewport (native window) in the standalone app, an egui::Window where viewports are embedded. Resizable (min 156x127.5), the panel letterboxed at 624:510 (View::fit). Test the_native_viewer_opens_closes_and_ignores_all_input: a viewport pass of its own runs, a Close viewport event closes it (no pass after), the menu command reopens it.
- AC 3: Bitmaps::of_kit per kit, file by file fallback to the built-in (missing, broken, or not a recrypted mod); cached per kit, rebuilt when doc.kit changes; the frame list follows the panel each frame. Tests a_mods_bitmaps_fall_back_one_by_one and the_viewer_redraws_with_the_mod_and_the_song (a repacked standard mod with a red 12507 and no 12057: the mod's knob strip, one fallback; back to the built-in kit; a cutoff change from the engine changes the drawn frame).
- AC 4: theme_view::ops draws every control from the Panel model: knobs, faders, toggles, switches (switch_position), 303 LEDs (Panel::led_lit, the main panel's rule incl. the note flash), focus bars, pattern grid and bank, 808/909 step LEDs with the running light, 808 selector, all displays, mixer/effect/master meters, GR LEDs, Play/Record lit, Sync/MIDI from plugin_leds.
- AC 5: tests knobs_show_the_ui_spec_frame (7 knobs of 4 strip kinds, frame ((n-1)p+500)/1000 and not its neighbor), leds_toggles_and_lights_follow_the_state (pitch LED, Play, focus, grid, bank, 808 and 909 step LEDs), displays_show_their_values (tempo 141, disabled bar, bar 3), meters_and_faders_follow_the_state; embedded Standard.rbm only, no MIDI, audio device or original's file.
- AC 6: every_thirdparty_mod_opens_in_the_viewer: ORB_THIRDPARTY_MODS_DIR=~/rb-mods/mods, 320 mods composed without a panic (91 with some built-in bitmaps, all 'missing'); skips without the variable.
- AC 8: the viewer senses hover only; NotAllowed cursor over the panel. Native test: 7 points over main-panel controls x 14 input kinds (move, click, shift click, double click, drag, wheel, Space/C/Enter keys): panel model (with its events), status, errors, song path/kit/modified unchanged, the drawn frame list unchanged (no pressed look), cursor NotAllowed every frame. Embedded test: pointer input over the viewer window lying on the main panel changes nothing and the cursor is NotAllowed; control: the same input with the viewer closed does change the main panel. The viewer writes no status line.
- AC 7 left unchecked: needs the original running (see the final summary for what to capture).

Checks: cargo test (workspace) 948 passed, 0 failed; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. Dev app: target/OpenRB-338.app (cargo build --release -p orb-ui; scripts/macos-dev-app.sh).

Review round 1 (the user, 2026-11-19, via the supervisor):
1. Mods > Theme Viewer... while the viewer is open must bring it to the front with the focus. Done: App::show_theme_viewer sets a pending focus when already open; the next viewer pass sends ViewportCommand::Focus once from inside the viewport. Test the_menu_item_focuses_the_open_viewer: no Focus for a new window; after the second command exactly one Focus in the root pass's viewport_output for the viewer, one viewport pass (no second window), none on later frames. (egui hands an immediate viewport's commands out in the root pass's FullOutput, not the nested one: first test version counted 0.)
2. The plugin has no Theme Viewer. Done: the item is hidden when is_plugin(); show_theme_viewer and theme_viewer_ui do nothing in a plugin. The custom egui::Window branch for embedded viewports is removed (dead: only the plugin had no native windows); show_viewport_immediate alone remains (egui's own fallback covers a backend without windows). The embedded test went with it. New test orb-plugin/tests/editor.rs::the_mods_menu_has_no_theme_viewer (the plugin's Mods menu shows Refresh List and Info..., no Theme Viewer; the command leaves it closed). docs/ui_spec.md sections 18, 20, 21 and docs/plugin.md updated.

Review round 1, continued (the user, 2026-11-19, via the supervisor):
3. Space in the viewer window switches its scaling between bilinear (default) and nearest pixel. Done: the viewer's viewport callback looks for a Space press (pressed, not a repeat) in its own input; App toggles Settings::viewer_nearest through the new setter set_viewer_nearest and saves at once (task-229's merge: a new key theme_viewer_filter = "linear" | "nearest", read, merged, written and refreshed like the others; a save error goes to the error box as for the startup-sound preference). The texture is rebuilt with TextureOptions::NEAREST / LINEAR when the setting differs from what it was made with. Test space_in_the_viewer_switches_the_scaling_and_is_kept: bilinear by default; Space -> nearest, nothing else changes, no playback; the file holds theme_viewer_filter = "nearest" and keeps its other key (startup_sound = false); a held key (presses while down, which egui marks as repeats) switches once; a new App on the same file starts nearest, Space goes back; control: Space in the main window starts playback and leaves the filter. A first test version sent a lone press with repeat: true after a release; egui turns that into a fresh press (repeat is egui's own, from keys_down), so the test holds the key instead. Task description and AC 8 reworded with the CLI to allow this one key; AC 8 re-checked below.
4. The 808's mod window (App::mod_view) is a link to the viewer in the standalone: ui.interact with Sense::click, on_hover_cursor(PointingHand), clicked (at release) -> App::show_theme_viewer (opens, or Focus when open). Not in a plugin. What a click there did before: only the press's row focus (App::focus_click gives row 3 the focus), harmless and kept; the mod window overlaps no other control (808 legend x 10..273 and labels 78..183 of the section; the window starts at x 277). Tests: the_mod_window_opens_and_focuses_the_viewer (hand cursor; a click opens the viewer and changes nothing else (focus was already row 3); a second click sends one Focus, still one window) and orb-plugin editor.rs::the_mod_window_is_no_link_in_the_plugin (no hand cursor, a click opens nothing).
Docs: ui_spec.md sections 6, 17 (the settings key), 18, 20, 21; plugin.md.

Round 1 checks: cargo test (workspace) 952 passed, 0 failed; clippy -D warnings clean; nightly fmt check clean. AC 8 re-checked for its new wording: the native-input test (all input kinds, Space included, change nothing of the song, transport, main panel or status), the Space test (scaling switched and kept, no playback) and the cursor checks. Dev app rebuilt at target/OpenRB-338.app.

Review round 2 (the user, 2026-11-19: "PCF sliders are off"):
- What was wrong: the viewer drew the four PCF faders on the long track 12032 (15x54, travel 43), the mixer's. The original draws them on the short track 12030 (15x45, travel 34) at (530+19i, 203): matched outside the handle's rows, the screenshot equals 12030 there at a mean difference of 1.5 for all four (12032 would run 9 px over the FREQ/Q/AMT/DEC labels, and its tick lines missed the background's). So the handles sat too low and the tracks covered the labels. The round-1 table had 12030 as 'not on the panel': its full-picture search compared it with the handle over it and missed.
- The value -> position rule was otherwise right: handle top = round((1 - p/999) * (track_h - 11)), top at 999, bottom at 0 (section 2's ends); the handle anchor is its top edge. Each fader shows its own parameter (pcf.freq/q/amt/dec at 4, 23, 42, 61; matched handles at those x in the screenshot, 1.8).
- Other faders checked: the mixer level (466, 109+115c) and the master (559, 75) are 12032 and lie on their backgrounds at 4.5; correct, unchanged.
- Fix: theme_view::fader takes its track (TRACK_LONG 12032/54 or TRACK_PCF 12030/45) and theme_view::handle_top(p, track_h). Check: the viewer with the PCF faders at p 529 and mixer 1 at 836 against panel.png: the PCF fader area differs by 1.43 mean, mixer 1's fader by 1.54 (JPEG noise).
- Test fader_handles_sit_on_their_tracks: for p 0, 999, 500, 250 the handle top is 34/0/17/25 on a 45 track and 43/0/21/32 on a 54 track, for the 4 PCF faders, 2 mixer channels and the master: the handle at that place, the track above it, and the PCF labels under the tracks still the background's.
- Docs: ui_spec.md section 21 (12030, 12031, 12032 rows; 12030 removed from the unplaced list) and section 9 (the PCF track is 45 px, not the table's 54). Possible follow-up for the supervisor: if the original's CBitmapFaderView takes its travel from the track, the PCF's drag scale is 34 px, not the 43 of section 2 that the main panel uses [G]; not changed here.

Round 2 checks: cargo test (workspace) 953 passed, 0 failed; clippy -D warnings and nightly fmt clean; dev app rebuilt at target/OpenRB-338.app.

Review round 3 (the user, 2026-11-19, testing on Linux):
- WSLg (Wayland): the app is dropped by the compositor (connection reset, no wl_display.error) on a focus change or when the viewer closes; not on GNOME/Mutter Wayland nor under X11 in WSLg. Taken as a WSLg compositor bug, no code change; documented in docs/building.md's Linux notes with the workaround WAYLAND_DISPLAY= (X11; the .tar.gz then needs libxkbcommon-x11-0).
- Focus on Linux: ViewportCommand::Focus does not raise the open viewer on GNOME (Wayland and X11): Mutter needs an xdg-activation token from the click and egui passes none. The user's decision: on Linux only, the menu item or the mod window's click on an open viewer closes it and opens a new window at the same inner size. Done: theme_view::REOPEN_TO_FOCUS = cfg!(target_os = "linux"); the viewer's viewport id carries a window serial, so a new serial makes egui drop the old viewport (not shown in the pass) and open a new one; each pass records the window's inner size (ViewportInfo::inner_rect) and the builder asks for it. App::set_theme_viewer_reopens drives either way in tests. Tests: on_linux_an_open_viewer_comes_back_as_a_new_window_of_its_size (menu and mod-window click: new viewport id, the builder asks 700x572 after a resize, the old viewport is gone from the root pass's output, no Focus) and the two Focus tests pinned to the macOS/Windows way.
- Wayland: the viewer opened from the menu appeared without frame and title bar until the mouse entered it (winit draws the frame on a repaint). Done: a newly opened window requests repaints of its viewport and the root for its first 3 passes, every platform. Test a_new_viewer_window_repaints_at_once: root repaint delay 0 for the 3 passes after the open, idle (MAX) before and after; with the pass count set to 0 the test fails (checked).
- Docs: ui_spec.md section 21, building.md.

Round 3 checks: cargo test (workspace) 955 passed, 0 failed; clippy -D warnings and nightly fmt clean; dev app rebuilt at target/OpenRB-338.app (macOS: the Focus way).

Review round 4 (the user, 2026-11-19: ReBirth 2.0 with ReBirthday against orb's viewer: Sync/MIDI LEDs lit with a green bar, Loop, the Shuffle pressed look on the 808/909 rows, the 909 Flam button, 'maybe others').

Side by side (AC 7), 2026-11-19 13:19-13:21:
- Test machine: took all 6 redc-win-run permits (owner = this branch), copied the installation without the ISO to C:\re\rb\t187\app with only the recrypted ~/rb-mods/mods/ReBirthday.rbm in its Mods, backed up the user's preferences to C:\re\rb\t187\prf_found.prf (sha256 afcf6797...). Starting the copy handed over to the user's own running ReBirth (PID 5324, single instance) and exited; the user's preferences were unchanged (same hash). The user's ReBirth showed ReBirthday on the new-song template in song mode, stopped; I sent it no input (not my process), took full-screen screenshots only, released the permits. Screenshots (outside git): scratchpad wt187/shots/03_user_rebirthday.png (panel at screen (8, 74); crop shots/03_user_rebirthday_panel.png), orb's viewer of the same mod wt187/orb_rbd2.png, side by side wt187/side_rbd.png, diff masks mask24.png (before) and mask24b.png (after the fixes). Scripts: wt187/tm.py, setup.py, s1.py, check2.py.
- The screenshot is lossless: every strip matched at a mean difference of 0.0 (scripts check2.py and the inline checks), so frames and places are exact.
- Root cause of the user's findings: the viewer cut every strip's frame to the control's size of ui_spec sections 3-9; the original sizes a view by its bitmap (frame (w+1)/n-1 wide, the bitmap's height, at the control's top-left). ReBirthday's strips are other sizes in 28 cases (12041 Sync/MIDI 23x7 -> 7x7 frames, so the 11x11 cut showed part of the green frame; 12551 Loop 81x15 -> 40x15; 12568 Flam 35x21 -> 17x21; 12571 Shuffle 53x21 -> 26x21, so the 18x20 cut looked pressed; 12507 knobs 30x34; ...). Fix: Op::size draws pictures and horizontal strips at their own frame size; vertical strips (grid, bank, meters, GR) keep the control size (same size in every mod seen). After it, all 36 horizontal strips checked match exactly at the documented places with one frame each.
- Other differences found and fixed: the pattern on/off button is at x 4 of its section (orb drew 5; exact match at 4, the built-in screenshot also better at 4); the master's right meter at x 50 (orb drew 51; exact at 50); the 909's selected-instrument LED 12567 was not drawn: it is lit over the selected instrument's dot (BD's at (29, 76) exact; places of the 12 dots from the background art; AC's at (7, 52)), now drawn (theme_view::TR909_SELECTED_LED).
- Confirmed, no change: the dist meter at x 64; Sync/MIDI frame 0 is off, Loop frame 0 off; the mode switch frame 1 = song (the user's song was in song mode); the lit on/off buttons (12550, 12556, 12066) are frame 1 (exact match lit), so the pale look in ui_spec/panel.png is not the strip's frame 1 and stays unexplained [G] (that capture is not lossless).
- Remaining full-panel differences after the fixes (threshold 24 of 255; JPEG decoder noise below it): only state: knob and fader positions, the row focus, the 303 note LEDs, the effects' on/off, PCF pattern and faders, the 808 switches, the 909 steps, row 4's Shuffle on, row 4's bank. orb's viewer was composed from Panel::new() in song mode, not from the template song (the template is a raw block in the preferences at 0x4000, not an RBS), so these were checked control by control by which frame matched.
- Not done: a like-for-like with the built-in kit (the user's ReBirth was running; a second instance hands over; switching its mod would need input to a process I did not start). The meter scale and the fader handle's rounding stay [G] (meters dark; template values unknown).

Also in round 4: the default scaling is Nearest (Settings::viewer_nearest true with nothing saved; a saved theme_viewer_filter wins), the letterbox is black (theme_view::DESK). Tests: a_strip_of_another_size_draws_its_own_frames, the_909_lights_its_selected_instrument, the_letterbox_is_black, the Space test reworked for the Nearest default (a saved linear wins on the next run), the fallback test's odd strip now drawn at its own frame size (a too-narrow strip draws nothing, no panic). Docs: ui_spec.md sections 4, 7, 9, 15, 17, 21; task description updated with the CLI.

Like-for-like with the built-in kit (2026-11-19, after the user closed their ReBirth):
- Permits: all 6 of redc-win-run held by worktree-agent-a6ffc02acfb4902f5 for the whole session (13:49 to 14:00), released after. No ReBirth ran before (tasklist empty).
- My copy C:\re\rb\t187\app with its Mods folder emptied (my ReBirthday.rbm deleted), so the built-in kit. Started it (PID 12148), dismissed the MIDI box, the new song "Untitled song" in pattern mode, stopped. Panel at screen (94, 160). Lossless full-screen BMP screenshot.
- Saved that song from the original (File > Save As, C:\re\rb\t187\untitled.rbs), downloaded it, opened it in orb (App::with_link + open_song, 8 engine calls with Link::sync) and composed the viewer frame (a scratch test, not committed).
- First diff: 5,722 pixels differ, all but one region by 1 to 3 (JPEG decoders: the original's vs the image crate). The one region: the gain-reduction LEDs (y 442..451, x 534..568). The original lit LEDs 1 to 5 (green, green, green, yellow, yellow); orb's viewer lit the link's center LED (and its one amber LED left of it: the link's gr_leds at idle are [0,0,0,2,1,0,0,0,0], byte 18 = unity gain).
- Experiments on the original (playing the untitled song): master COMP switch off: all nine LEDs dark (also while playing); on again: LEDs 1 to 5; threshold dragged to its lowest while playing: all nine lit (lit row: green x3, yellow x3, red x3). So the meter is a bar from the left: none at byte 0, 5 at 18 (unity), 9 at 127. The scale between is G (sub_424A7E not read; no idalib MCP tools this session).
- Fix (viewer only): theme_view::gr_lit maps the link's count of LEDs left of center (n 0..4) to a bar of 0, 5, 6, 8, 9 LEDs from the left; ops draws LEDs 0..gr_lit of the lit row. Test meters_and_faders_follow_the_state updated (unity: LEDs 1-5 lit, 6-9 off; n 3: 8 lit; n 0 and before the report: none).
- After the fix: 312,861 of 318,240 pixels equal, 5,379 differ by 1 to 3 in a channel (max 3), spread over the bitmaps; no place, frame or size differs. The pale lit on/off look of ui_spec/panel.png is that capture's (the original draws frame 1 as the viewer does).
- The main panel's GR LEDs (task-179's center-out amber/red to the left) are not the original's: a design question, filed as the decision task-251 (recommendation: the original's bar).
- Docs: ui_spec section 21 (12570 row, the built-in like-for-like paragraph, the pale button item, the viewer description).
Cleanup of the test machine:
- Closed my ReBirth (taskkill /PID 12148, "Save changes" answered Don't Save); no Rebirth.exe runs after.
- Preferences: my run rewrote C:/Users/pingw33n/AppData/Local/VirtualStore/Windows/ReBirth RB-338 2.prf (fe9a29c2...). Restored from my backup taken just before my run (prf_before_mine.prf, 6309526e...), which is the user's state after their own session; prf_found.prf (afcf6797..., taken in round 4) is older than the user's own session since then, so restoring it would have undone the user's changes. Both copies kept locally in my scratchpad.
- Deleted C:\re\rb\t187 entirely (rmdir /S /Q): app\ (my installation copy: Default Songs, Demo Songs, Mods (empty), ReBirth manual.pdf, ReBirth.GID, ReBirth.cnt, ReBirth.dll, ReBirth.hlp, ReWire.dll_, ReWire.reg, ReadMe.rtf, Rebirth.exe), prf_found.prf, prf_before_mine.prf, untitled.rbs. Nothing else on the machine was touched.
- Permits released (6 available, no holders).

Final round (the user, 2026-11-19): the user approved the viewer on 2026-11-19; no more testing. Last change: Help > Keyboard Shortcuts lists Space in the Theme Viewer window (switches the scaling between nearest and bilinear) in the group "Windows and the file browser", with no bind (crates/orb-ui/src/key_help.rs); standalone only, as the help window itself is (a plugin has no Keyboard Shortcuts, docs/ui_spec.md section 20). Unit test key_help::tests::the_theme_viewer_space_is_listed. docs/ui_spec.md: the Help menu row and the Help > Keyboard Shortcuts paragraph. cargo test 959 passed, clippy -D warnings and nightly fmt clean.

Review round (reviewer, 2026-11-19, send back, text only): (1) theme_view.rs module comment said Space switches "between bilinear (the default) and nearest"; fixed to nearest pixel (the default) and bilinear, as since round 4. (2) docs/ui_spec.md rows 12066 and 12550 still said [G: the lit frame does not match the screenshot (23.6 / 28.7)]; made [C] with the round-4 exact match of frame 1 lit (and the built-in like-for-like), the lossy panel.png being the cause of those numbers, as row 12556 and the side-by-side item say. nightly fmt check clean.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Mods > Theme Viewer... (standalone app only) opens a view-only native window (egui immediate viewport) that draws the panel as the original does: the current kit's 58 bitmaps at the original's geometry, each falling back to the embedded standard mod's when the mod lacks it or it does not decode, each strip at its own frame size as the original sizes its views, and every control from the live panel model (knobs by ((n-1)p+500)/1000, faders on their own tracks, switches, LEDs, the 909's selected-instrument LED, step and running lights, grid and bank, displays, meters, the gain-reduction LEDs as the original's bar from the left). It changes nothing: the not-allowed cursor over the panel, no status line, a black letterbox; its one key, Space, switches its scaling between nearest pixel (default) and bilinear, saved as theme_viewer_filter in ui.toml. The menu item, or a click on the 808's mod window (hand cursor; standalone only), opens it or brings the open one to the front (Focus on macOS/Windows, a new window of the same size on Linux). A plugin has neither. Code: crates/orb-ui/src/theme_view.rs, menu item, App field, the mod window's link, Settings::viewer_nearest, Panel::led_lit; tests crates/orb-ui/tests/theme_view.rs (18) and two in crates/orb-plugin/tests/editor.rs. Docs: docs/ui_spec.md section 21 (all 58 bitmaps; both side by sides) and sections 2, 4, 6, 7, 9, 15, 17, 18, 20; docs/plugin.md; docs/building.md (WSLg).

Side by side (AC 7), test machine, 2026-11-19, lossless screenshots:
- ReBirthday mod: every strip matches pixel for pixel at its place once strips are drawn at their own frame size; fixed the frame size, the pattern on/off x (4), the master right meter x (50) and the missing 909 selected-instrument LED; the rest was song state.
- Built-in kit, like-for-like (the original's new song, saved from it and opened in orb): 312,861 of 318,240 pixels equal, the other 5,379 off by 1 to 3 (JPEG decoder rounding), after fixing the gain-reduction LEDs (a bar from the left: none at gain byte 0, LEDs 1-5 at unity, all nine at the top; theme_view::gr_lit). The pale on/off look of ui_spec/panel.png is that capture's. The main panel's own GR LEDs differ from the original: decision task-251.
- Test machine cleaned: my ReBirth closed, the preferences restored to their state before my run, C:\re\rb\t187 deleted, permits released.

Review rounds 1-4 (the user, 2026-11-19): focus on reopen, no viewer in the plugin, Space for the scaling (default Nearest), the mod window as a link, PCF fader tracks, Linux reopen and repaint, WSLg note, the round-4 fixes; all in the notes.

Results: cargo test 959 passed; clippy and nightly fmt clean; 320 third-party mods compose without a panic. Branch worktree-agent-a6ffc02acfb4902f5, one commit.

Open [G]: the gain-reduction scale between 0, 18 and 127 (sub_424A7E not read), the meter scale, the fader handle's rounding, a mod that resizes the vertical strips. The user approved the viewer on 2026-11-19. Help > Keyboard Shortcuts lists the viewer's Space (key_help.rs, test the_theme_viewer_space_is_listed).
<!-- SECTION:FINAL_SUMMARY:END -->
