---
id: TASK-252
title: 'Main panel: the gain-reduction LEDs as the original''s bar from the left'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-19T12:38:00Z'
updated_date: '2026-11-19T13:24:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies:
  - TASK-251
references:
  - crates/orb-ui/src/link.rs
  - crates/orb-ui/src/model.rs
  - crates/orb-ui/src/theme_view.rs
  - docs/ui_spec.md
  - task-251
  - task-187
  - task-179
priority: medium
type: enhancement
ordinal: 203600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Decision task-251 (the user, 2026-11-19, option A). task-187's like-for-like with the original showed that the compressor's gain-reduction meter (the nine LEDs under COMP; bitmap 12570 in the original) is a bar from the left: idle with the compressor on the master it lights LEDs 1 to 5; with the threshold at its lowest while playing, all nine; with the master's COMP switch off, none. The engine byte behind it is the compressor's live byte 3, trunc(min(G*18, 127)) (0x1000EB48): 18 at unity gain, 0 at gain 0, 127 at the top. OpenRB-338's main panel (task-179's redesign; docs/ui_spec.md sections 15 and 16; crates/orb-ui/src/link.rs, the gr_leds block after the meters; crates/orb-ui/src/model.rs, the "meter" part with GR_OFF) lights the center LED green and the reduction amber then red to its left, which the original never shows. The theme viewer already draws the bar (`theme_view::gr_lit`), with the 6 and 8 LED steps guessed.

What to do:
- Read the original's meter drawing in IDA (read only): sub_424A7E and what it calls, to get the exact byte-to-LED-count mapping (thresholds for 1..9 LEDs), with addresses; document it in docs/ui_spec.md (sections 15/16 and 21) as [C].
- The link keeps the raw byte (a Panel field) instead of the 0..4 count; one shared mapping function serves the main panel and the theme viewer (replace `gr_lit`'s guess).
- The main panel lights a bar from the left by place, in the redesign's colors by place: green, green, green, amber, amber, amber, red, red, red; none when the master's COMP is off or the byte is 0.
- Update the main panel's art/model code and its screenshot test if any, and the viewer's tests to the read thresholds.

The user checks the dev app (scripts/macos-dev-app.sh) before the agent review. IDA databases: /Users/pingw33n/dev/openrb-338/ida/ReBirth.dll.i64 and Rebirth.exe.i64, through the idalib MCP tools, read only, never saved; if the tools are unavailable, say so and stop before guessing the scale.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/ui_spec.md gives the original's byte-to-LED-count mapping of the gain-reduction meter with addresses ([C]), read from sub_424A7E
- [x] #2 One mapping function from the compressor byte to the lit LED count is used by both the main panel and the theme viewer, with tests at the thresholds and at 0, 18 and 127
- [x] #3 The main panel lights a bar from the left in the colors green x3, amber x3, red x3 by place, none with the master's COMP off; covered by a test
- [x] #4 cargo test, clippy -D warnings and nightly fmt pass; the user checked the dev app
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. IDA (read only, a scratchpad copy of Rebirth.exe.i64, closed save=false): read the comp section's poll sub_404396, the GR view (CHorisontalMeterView) ctor/draw/update and the table it feeds, to get the byte-to-LED mapping with addresses.
2. orb-ui model: Panel keeps the raw byte (gr_byte, the compressor's live byte 3) instead of gr_leds; one function model::gr_lit(byte) -> 0..=9, the original's mapping, used by the main panel's part names and by theme_view (its guessed gr_lit goes); GR colors by place (green x3, amber x3, red x3) shared with orb-ui-art.
3. link: store the byte; Link::moving compares the lit count.
4. orb-ui-art: GR_OFF -> by-place colors, the demo lights LEDs 1 to 5; regenerate assets/ui.
5. Tests: the mapping at every threshold and at 0, 18, 127; the main panel's part names (bar by place, none at 0); a link test that the master's COMP off gives no LED; the viewer's tests on the byte.
6. docs/ui_spec.md sections 2, 15, 16, 19, 21 [C] with addresses; follow-up task for the header/master meters' scale (sub_424220, read on the way, out of scope).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a8bebb32e0d580a51 (worktree .claude/worktrees/agent-a8bebb32e0d580a51), rebased onto main cf8ac3f.

IDA read (Rebirth.exe.i64, scratchpad copy, headless, closed save=false):
- The comp section: ctor sub_403BE0 builds the GR view at section+0xD4 with CHorisontalMeterView's ctor sub_424ACC(bitmap [section+0x30]+0x1CC, 9 LEDs, 10 high, 7 wide) (0x403C90..0x403CB0; fields +0x24 = 9, +0x26 = 10, +0x28 = 7); the input meter is a CPeakMeterView (sub_4248B6) at +0x100. Layout sub_4040CE places the GR view at (8, 25, 63, 10).
- The poll sub_404396: byte +0x127 (live byte 3, the gain byte; +0x124 on, +0x125/+0x126 ratio/threshold, +0x128 the input meter = live byte 4) -> sub_4044FD (table) -> sub_424D45 (sets view+0x20) at 0x4044AD..0x4044CE; +0x128 -> sub_424220 -> sub_424A7E (the peak meter's setter, +0x22) at 0x4044D3..0x4044F4. So the task's sub_424A7E is the input meter's setter, not the GR's; the GR path is sub_4044FD -> sub_424D45.
- sub_4044FD (0x4044FD) fills byte_4D8910 once: t[0] = 0; for n = 1..N: t[n] = __ftol(((log(n / 16.0f) - log(0.0625)) / (log(7.9375) - log(0.0625))) * 127.0) (0x4046AC..0x404711; flt_4BA2F0 = 0x41800000 = 16.0, 0x3FB00000:0 = 0.0625, 0x401FC000:0 = 7.9375, dbl_4BA2F8 = 0x405FC000:0 = 127.0; fsubr/fdivr operand order: (var_44 - log(1/16)) / (var_54 - log(1/16)), intermediates stored as doubles). N is a copy-protection expression: 127 if \\.\SICE opens (the string is delta-decoded at 0x404519..0x40458A), else (dword_4DA0AC >> 16) - ([dword_4DFB24 + 0x13C](.., -6) & 0x7F), the expression whose intended value is 127 (docs/standard_midi_mapping.md: D = expr + 1 = 128). Returns t[byte] (0x404723).
- CHorisontalMeterView vtable 0x4BBB44: slot 15 (+0x3C) = draw sub_424C20, slot 18 = update sub_424D5D. sub_424D5D: count = sub_467F31(word +0x20, +0x24 (9), 127) = __ftol(fild v; fimul 9; fidiv 127) = trunc(t*9/127) (exact integer division for these integers); stored at +0x22 and redrawn (vtable +0x3C) when it changed.
- Draw sub_424C20: two blits sub_46F7BC: columns 0..count*7 from the strip's second row (src y = height 10, the lit row) and the rest count*7..63 from row 0 (unlit). A bar from the left of `count` LEDs.
- So lit LEDs = trunc(t[b]*9/127): 0 for b 0..1, 1 for 2..3, 2 for 4..5, 3 for 6..8, 4 for 9..15, 5 for 16..25, 6 for 26..43, 7 for 44..74, 8 for 75..126, 9 at 127 (python check; closest non-exact table value is far from an integer: only b 1 (0.0) and 127 (127.0) are exact, so a 1-ulp log difference changes nothing). 18 (unity) -> t 75 -> 5 LEDs, matching task-187's screenshots (idle 5, lowest threshold 9, COMP off none).
- COMP off -> none: orb's engine (comp.rs update, 0x1000EFB0) clears byte 3 when the compressor got no call since the last mixer update, so the byte is 0 when it is not routed or is off; the original's meter only shows the byte, so no extra rule is needed.
- On the way, the peak meters (CPeakMeterView): sub_424220 = byte_4DABA8: L[0] = 0, L[i] = __ftol(log(i) / log(127.0) * 127.0) (0x424258..0x424294); the poll stores L(byte) via sub_424A7E (+0x22), the update sub_424A96 copies it to +0x20, the draw sub_4249A8 shows frame sub_467F31(L, frames - 1, 127) = trunc(L*(frames-1)/127), frames = word +4 of the bitmap object (GetGroup 0x46DBC3) [G: that word is the frame count]. orb's meter_state is linear rounded up; a follow-up task.

Code (orb-ui, orb-ui-art):
- model.rs: Panel::gr_byte (the compressor's live byte 3) replaces gr_leds; model::gr_lit(byte) is the original's mapping (sub_4044FD table in f64 with its operand order, then trunc(t*9/127)), bytes above 127 count as 127 (the engine never writes one; the original's table has 128 entries); GR_COLORS = g,g,g,y,y,y,r,r,r by place (replaces GR_OFF), the same const in orb-ui-art (app test checks they agree). part_name lights comp.gr1..gr(n) in their place's lit color, the rest unlit in their place's color.
- link.rs: panel.gr_byte = live byte 3; Link::moving compares gr_lit (the lit count), so a byte change that lights no other LED does not repaint.
- theme_view.rs: its guessed gr_lit is gone; ops draws model::gr_lit(p.gr_byte) LEDs.
- orb-ui-art: demo lights LEDs 1..5 by place; assets/ui regenerated (cargo run -p orb-ui-art -- gen: layout.tsv comp.gr1..5 lines, panel.svg).
- No master-COMP special case: the original shows only the byte, and orb's engine clears byte 3 (comp.rs update, 0x1000EFB0) when the compressor got no call, so COMP off gives 0 -> none (test crates/orb-ui/tests/gain_reduction.rs).
- Observed: the new song's idle byte after 64 calls is 16 (gain still rising toward 1.0), still 5 LEDs.
- Process slip: one import rename in crates/orb-ui-art/src/layout.rs (GR_OFF -> GR_COLORS in the use list) was done with sed instead of Edit; content checked.
- Follow-up: task-253 (the peak meters' log scale, sub_424220, read on the way).

Acceptance evidence:
- AC 1: docs/ui_spec.md section 2 'Gain-reduction meter' (the table of bytes to LEDs, sub_403BE0/sub_424ACC/sub_4040CE/sub_404396/sub_4044FD/sub_424D5D/sub_424C20 with addresses, [C]); sections 9, 15, 16, 19, 21 updated. Note: sub_424A7E, named in the task, is the peak meters' setter; the GR path is sub_4044FD -> sub_424D45 -> sub_424D5D.
- AC 2: model::gr_lit used by model::Panel::part_name (main panel), theme_view::ops (viewer) and Link::moving; tests model::tests::gain_reduction_byte_to_lit_leds (every threshold both sides, 0, 1, 18, 126, 127, 255, monotone) and tests/theme_view.rs meters_and_faders_follow_the_state (bytes 18, 74, 75, 127, 0).
- AC 3: model::tests::gain_reduction_and_rest_leds (bytes 18, 127, 0, 75: green x3, amber x3, red x3 by place, unlit in the same colors) and tests/gain_reduction.rs the_master_comp_switch_turns_the_bar_off (idle on the master: 5 LEDs; COMP off: byte 0, none).
- AC 4 (partly): cargo test 961 passed, 0 failed, 5 ignored; cargo clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. The user's check of the dev app is pending: target/OpenRB-338.app in the worktree (version 2026.1119.1301).

User check: the supervisor reported on 2026-11-19 that the user checked the dev app (target/OpenRB-338.app, version 2026.1119.1301) and said it looks right.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The main panel's gain-reduction LEDs now follow the original (decision task-251, option A): a bar from the left, colored by place (green x3, amber x3, red x3), driven by the compressor's gain byte (Panel::gr_byte, live byte 3) through model::gr_lit. That one mapping serves the main panel, the theme viewer and Link::moving.

The mapping was read in IDA (Rebirth.exe, read only). The comp poll sub_404396 passes the byte through the table sub_4044FD, t = trunc((ln(b/16) - ln(0.0625)) / (ln(7.9375) - ln(0.0625)) * 127). The view's update sub_424D5D lights trunc(t*9/127) LEDs, and its draw sub_424C20 blits a bar from the left. Lit LEDs: 0 for bytes 0..1, 1 from 2, 2 from 4, 3 from 6, 4 from 9, 5 from 16 (18 is unity), 6 from 26, 7 from 44, 8 from 75, 9 at 127. sub_424A7E, named in the task, is the peak meters' setter, not the gain-reduction meter's. COMP off gives byte 0 (the engine clears it, 0x1000EFB0), so no LED is lit.

docs/ui_spec.md sections 2, 9, 15, 16, 19 and 21 record the read [C] with addresses. The art's demo and assets/ui were regenerated. Follow-up task-253: the peak meters' log scale (sub_424220), read on the way.

Branch worktree-agent-a8bebb32e0d580a51, one commit on main cf8ac3f. Tests: cargo test 961 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt clean; the user checked the dev app on 2026-11-19.
<!-- SECTION:FINAL_SUMMARY:END -->
