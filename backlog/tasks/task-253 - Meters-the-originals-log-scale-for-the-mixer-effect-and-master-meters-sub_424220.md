---
id: TASK-253
title: >-
  Meters: the original's log scale for the mixer, effect and master meters
  (sub_424220)
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-19T12:49:00Z'
updated_date: '2026-11-19T14:32:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies:
  - TASK-254
references:
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/theme_view.rs
  - docs/ui_spec.md
  - task-252
priority: low
type: enhancement
ordinal: 244000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: task-252 read, on the way to the gain-reduction meter, how the original scales its other meters (CPeakMeterView: the four mixer channels, the master's two, the delay, dist, PCF and comp input meters), and OpenRB-338 does not follow it. orb's link::meter_state maps the engine's meter byte linearly, ceil(byte*10/127) states (docs/ui_spec.md section 16, [G]); the main panel and the theme viewer (theme_view::header_meter_frame) both use it.

What the original does (Rebirth.exe, read in task-252, docs/ui_spec.md section 2 'Meters'): each section's poll (sub_412DC7 master, sub_413F47 mixer channel, sub_407F7A delay, sub_408AF5 dist, sub_41B914 PCF, sub_404396 comp input) passes the byte through sub_424220, a table L[0] = 0, L[i] = __ftol(log(i) / log(127.0) * 127.0) (0x424258..0x424294), and stores it with sub_424A7E (view +0x22); the update sub_424A96 copies it to +0x20 and redraws; the draw sub_4249A8 shows frame sub_467F31(L, F - 1, 127) = trunc(L * (F - 1) / 127), where F is the word at +4 of the view's bitmap object (read by 0x46DBC3) [G: that word is the strip's frame count, 11 for 12557 and 4 for 12558/12067].

What to do: confirm F (read the bitmap object's ctor/loader that sets +4, read only in IDA), then make one function byte -> frame for the original's strips (theme viewer) and decide how the main panel's 11-state and 5-segment meters follow it (the redesign's meters need not match exactly; if the change is visible ask the user via a Decide task first). Unit tests at the frame thresholds; docs/ui_spec.md sections 2, 15, 16, 21 updated [C].
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/ui_spec.md gives the meter byte-to-frame mapping with addresses, F confirmed [C]
- [x] #2 The theme viewer draws the meters by that mapping; tests at the thresholds
- [x] #3 The main panel's meters follow the mapping or a Decide task records why not
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. IDA (read only, scratchpad copy of Rebirth.exe.i64, closed save=false): confirm F, the word at +4 of the meters' bitmap objects (who sets it, the values for 12557, 12558, 12067); re-check sub_424220, sub_467F31, sub_4249A8 against the disassembly.
2. model.rs: Panel::meters holds the raw meter byte (0..=127) by id instead of the linear state; model::peak_meter_level(byte) (sub_424220's table, f64, operand order, trunc) and model::peak_meter_frame(byte, frames) = trunc(L*(frames-1)/127); the linear model::meter_state (moved from link.rs) stays the main panel's mapping until the decision.
3. theme_view: the master meters (12557, 11 frames) and the header meters (12558, 12067, 4 frames) drawn by peak_meter_frame; header_meter_frame (guessed) goes.
4. link.rs stores the bytes; the main panel's part_name converts with model::meter_state, so the main panel does not change; demo bytes chosen to keep the demo's states.
5. Tests: the table and the frames at every threshold (both sides) for 11 and 4 frames, 0, 127, above 127; the viewer's test on bytes; the main panel's mapping unchanged.
6. The main panel's meters change visibly under the original's scale: a Decide task (m-5, @pingw33n) with the options and a recommendation; AC 3 waits on it.
7. docs/ui_spec.md sections 2, 15, 16, 21 [C] with addresses. Build the dev app for the user's check (the viewer's meters change), task stays To Do.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a471880677e001704 (worktree .claude/worktrees/agent-a471880677e001704).

IDA read (Rebirth.exe.i64, scratchpad copy, headless, closed save=false):
- F confirmed [C]: the view's bitmap object (+0x1C, set by the CPeakMeterView ctor sub_4248B6 at 0x4248ED) is built by sub_46DB82, which stores its second argument at +4 (0x46DB90) and the resource id at +8 (0x46DB9A); CBitmapSequence sub_46DBFD and CFilmStrip sub_46DCB6 call it. GetGroup 0x46DBC3 returns that word. The resource list sub_4287A0 builds: 12067 (0x2F23) at +0x124 with 4 (push 4 at 0x428A09); 12558 (0x310E) at +0x13C with 4 (0x428A3B); 12557 (0x310D) at +0x148, a CFilmStrip of 11 frames, cells 0,0,15,56 (0x428A54..0x428A5E).
- Which strip each view takes: master ctor sub_412AF0 [+0x30]+0x148 for both meters (0x412B84..0x412BC1); mixer sub_413B55 [+0x2C]+0x13C (0x413CAE); delay sub_4076C0, dist sub_408590, PCF sub_41B390, comp sub_403BE0 [+0x30]+0x124 (0x4077DD, 0x408642, 0x41B4D9, 0x403CBB).
- Re-checked: sub_424220 (0x424258..0x424294: fild i; log; fstp double; log(127.0: push 405FC000:0); fdivr [ln i] = ln(i)/ln(127); fmul dbl_4BB9A8 = 0x405FC000_00000000 = 127.0; __ftol; byte store), 128 entries (cmp 0x80 at 0x424250), lookup masked & 0xFF (0x4242A6). sub_467F31: fild a; fimul b; fidiv c; __ftol. sub_4249A8: frame = sub_467F31(word +0x20, F - 1, 127) at 0x424A18..0x424A38. sub_424A7E stores +0x22; sub_424A96 copies to +0x20 and redraws (vtable +0x3C) only when it changed. The master poll sub_412DC7 passes bytes +0xF4/+0xF5 straight to sub_424220 (0x412E73..0x412EB8); other polls at 0x4044DD, 0x4081FB, 0x408C16, 0x414407, 0x41BC60.
- Mapping (python over the formula): L = [0, 0, 18, 28, 36, 42, ...]; only L(1) = 0 and L(127) = 127 are exact, the others at least 0.0031 from an integer. Frames: 11 frames step at bytes 2, 3, 5, 7, 12, 19, 30, 49, 81, 127; 4 frames: green from 6, amber from 26, red only at 127.
- Before: orb's linear meter_state steps at 1, 13, 26, 39, 51, 64, 77, 89, 102, 115; the viewer's guessed header_meter_frame was green from state 1 (byte 1), amber from state 7 (byte 77), red from state 9 (byte 102).

Visible on the main panel: following the original would light much more at low levels (byte 30: master state 7 against 3). So the main panel is not changed; decision task-254 (options A one log scale with header_meter_segments, B per-strip formula with 5 states, C keep linear; recommendation A). AC 3 waits on it.

Code:
- model.rs: Panel::meters holds the meter byte; model::meter_state (linear, moved from link.rs, unchanged) for the main panel's part_name; model::peak_meter_level (sub_424220) and model::peak_meter_frame(byte, frames) (sub_4249A8) for the viewer. Demo bytes 76, 63, 101, 38, 101, 88, 50, 38, 25 keep the demo's states 6, 5, 8, 3, 8, 7, 4, 3, 2. Removed a stray doc line ("`c` for a section name like `mixer3` -> 2.") glued onto header_meter_segments' doc.
- link.rs: stores the bytes; Link::moving's "any meter > 0" is unchanged in meaning (state > 0 iff byte > 0).
- theme_view.rs: header_meter_frame (guessed) gone; 12557 by peak_meter_frame(b, 11), 12558 and 12067 by peak_meter_frame(b, 4).
- Tests: model::tests::peak_meter_frames_follow_the_originals_log_scale (every frame's first byte and the byte before, 11 and 4 frames, 0, 127, 128, 255, monotone), main_panel_meter_state_is_linear, header_meters_map_the_11_states_to_4_segments (on bytes); tests/theme_view.rs meters_and_faders_follow_the_state (bytes 30/29 -> 7/6, 0 -> 0, 127 -> 10 on 12557; 127/126/unset -> 3/2/0 on 12558; 6/5/26 -> 1/0/2 on 12067).

Acceptance evidence:
- AC 1: docs/ui_spec.md section 2 'Meters': the byte-to-frame mapping (sub_424220, sub_4249A8, sub_467F31, the polls), F confirmed (sub_46DB82 +4, the resource list sub_4287A0: 12557 11, 12558 4, 12067 4) and the table of thresholds, [C]; sections 9, 15, 16, 21 updated.
- AC 2: theme_view draws 12557 by model::peak_meter_frame(b, 11) and 12558/12067 by peak_meter_frame(b, 4); tests model::tests::peak_meter_frames_follow_the_originals_log_scale (every threshold, both sides) and tests/theme_view.rs meters_and_faders_follow_the_state.
- AC 3: open, waits on decision task-254 (the main panel would change visibly; the user decides). The main panel is unchanged (model::meter_state, the same linear mapping, now applied to the byte in part_name).
Checks: cargo test 963 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. main had not moved (1d7033c), no rebase needed; git diff main --stat lists only this task's files.
The theme viewer's meters change visibly (the log scale): dev app target/OpenRB-338.app in the worktree for the user's check; the task stays To Do.

Decision task-254 (the user, 2026-11-19): option A. Applied: model::meter_state = peak_meter_frame(byte, 11) (the master's states step at 2, 3, 5, 7, 12, 19, 30, 49, 81, 127; byte 1 now shows nothing); the header meters keep header_meter_segments over it (one segment from byte 2, two from 7, three from 30, four from 81). Demo bytes 19, 12, 49, 5, 49, 30, 7, 5, 3 keep the demo's states 6, 5, 8, 3, 8, 7, 4, 3, 2, so orb-ui-art's assets (which do not use the model's demo) need no regeneration. Tests: model::tests::main_panel_meter_state_follows_the_originals_scale (every state's first byte and the byte before, 255), header_meters_map_the_11_states_to_4_segments (bytes 0, 1, 2, 6, 7, 29, 30, 80, 81, 127, 200; master 29, 30, 126, 127). docs/ui_spec.md sections 2, 15, 16, 19 updated.
- AC 3: the main panel follows the mapping (decision task-254), the tests above.

User check: the supervisor reported on 2026-11-19 that the user checked the dev app (target/OpenRB-338.app, version 2026.1119.1423) and said the meters look fine.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The original's meter scale is read and applied. The theme viewer follows it, and the main panel follows it by decision task-254 (option A, the user, 2026-11-19). The user checked the dev app on 2026-11-19 and said the meters look fine.

What the original does (Rebirth.exe, read in IDA, read only): a peak meter maps the engine's byte through the table sub_424220, L = trunc(ln(b) / ln(127) * 127). The draw sub_4249A8 then shows frame trunc(L * (F - 1) / 127). F is the frame count, confirmed: sub_46DB82 stores it at +4 of the bitmap object (0x46DB90), and the resource list sub_4287A0 gives 11 for 12557 and 4 for 12558 and 12067.

Resulting steps: the master's meter steps at the bytes 2, 3, 5, 7, 12, 19, 30, 49, 81 and 127. The original's four-frame meters are green from 6, amber from 26 and red only at 127.

What changed in orb: Panel::meters keeps the byte. The theme viewer draws each strip by model::peak_meter_frame. The main panel's model::meter_state is peak_meter_frame(byte, 11), and its header meters keep header_meter_segments over that state (segments from the bytes 2, 7, 30 and 81). docs/ui_spec.md sections 2, 9, 15, 16, 19 and 21 are updated [C].

Branch worktree-agent-a471880677e001704, one commit on main 1d7033c. Tests: cargo test 963 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
