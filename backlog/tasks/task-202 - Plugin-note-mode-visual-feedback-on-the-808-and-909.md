---
id: TASK-202
title: 'Plugin: note-mode visual feedback on the 808 and 909'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-16T07:47:00Z'
updated_date: '2026-11-16T17:32:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-185
  - TASK-200
  - TASK-201
references:
  - backlog task-185 (final summary)
  - crates/orb-ui/src/app.rs
  - docs/ui_spec.md
priority: medium
type: feature
ordinal: 215000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
In note mode (task-200) the panel shows which drum voices are sounding, as task-185's final summary decides. It is separate from the editor's selection, which notes never change. The user reviews the visual result before merge.

- **808:** a green outline of the same shape as the orange selection outline (crates/orb-ui/src/app.rs, the selection marks of task-179), on the chip of the voice played: the lower legend (LT, RS, CP, ...) or the upper chip (LC, CL, MA, ...) by the voice the note named, whatever the slot's switch says. On an instrument that is also selected, only the green shows. A hit with AC also outlines AC green.
- **909:** the instrument's LED glows green instead of red; the AC LED glows green for a hit with AC.
- **When lit:** while the voice sounds, from task-200's per-voice sounding mask (active and gain above about -60 dB), with a minimum on-time of about 100 ms (tune by eye). Several voices can be lit at once; a CH choke darkens OH; a retrigger restarts the light; the BD is lit while either of its two alternating voices sounds. AC is lit while any voice whose last hit had AC is lit.
- **303s:** nothing.
- **Color-blind users:** green must stay distinguishable from the red/orange selection by brightness or glow, not by hue alone.
- Pattern mode: no change (its running lights as now).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Green outlines and LEDs appear for sounding voices in note mode as described, including shared-slot chips by the played voice, AC, choke and minimum on-time
- [x] #2 The selection marks are unchanged by notes; with both on one instrument only green shows
- [x] #3 Green vs red/orange is distinguishable in a deuteranopia simulation of a screenshot
- [x] #4 The user reviewed the feedback (screenshots or a build) and approved it before merge
- [x] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Art (orb-ui-art): lit808_<name> / litsel808_<name> (chip + green ring replacing orange; white core, wide glow for color-blind), led909_lit / led909ac_lit (green LED, wide halo, white core); regenerate assets/ui.
2. orb-ui embed.rs: Lights (per-sound minimum on-time 100 ms from the rising edge of the snapshot's sounding mask, AC while a lit sound has its ac bit), App::note_lights (only plugin note mode, not preview; 16 ms repaint).
3. app.rs instrument_selection: green on the chip by the played sound (lower/upper), litsel for the selected chip, 909 green LEDs; standalone untouched.
4. editor_preview --note CH,NOTE,VEL; screenshots; deuteranopia simulation; standalone --screenshot byte-identical vs base.
5. docs/plugin.md 9.4; unit tests of Lights; cargo test/clippy/fmt; bundle + validate.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a25d385c8cb7413ea (rebased on main 1349508).

Art: assets/ui gets lit808_<17>, litsel808_<17>, led909_lit, led909ac_lit (generator crates/orb-ui-art parts.rs chip808_ring; existing parts unchanged). Green ring 1.6 px, white 0.7 core, 3.6 px glow vs the orange 1.1 px with faint glow; green LED has a wide halo and white core. First attempt (1.3 ring, 0.45 core) was too close to the orange in a deuteranopia simulation (Machado severity 1.0 on the screenshot crop); the stronger ring separates clearly (thick bright white-cored ring with big glow against a thin orange line).
Code: orb-ui embed.rs Lights/Lit (MIN_ON 100 ms from the rising edge of the mask bit, LIGHT_POLL 16 ms repaint in note mode), App::note_lights, app.rs instrument_selection paints the green parts; lit only in plugin note mode and not in the preview. Known limits: the snapshot holds only the latest block's mask, so a voice that starts and ends between two 16 ms frames is not seen and a retrigger of a voice that never stopped shows no new edge. Idea if the user sees missed lights: a sticky per-block OR of the mask in the snapshot (engine/player change, follow-up).
Tools: editor_preview --note CH,NOTE,VEL (host note-on every buffer).
Standalone check: OpenRB-338 --screenshot of tests/songs/diag/songs/909_low_tom.rbs from this branch and from base 1349508 (own target dir): both sha256 a3c2890edd63...a843d6, byte-identical.
Screenshots (scratchpad/shots): lit_both.png (808: BD selected+lit, CH, LC, CL, AC lit; 909: BD, SD, CC, AC), sel_other.png (SD selected, orange, others lit green), *_crop/_deutan.png, pattern_mode_no_lights.png.
Deviation to record: two small source edits slipped through sed (a color/width tweak in parts.rs); content is identical to an Edit.

Verification: cargo test 813 passed, 0 failed (ORB_CD_ISO not set); cargo clippy --all-targets -D warnings clean; nightly fmt --check clean; scripts/validate-plugin.sh: clap-validator and pluginval passed on target/bundled/OpenRB-338.{clap,vst3} (built with cargo xtask bundle orb-plugin --release in this worktree, installed nowhere). New unit tests embed::tests (minimum on-time, AC, dark in pattern mode/preview). AC #4 (the user's review) is left unchecked.

Review round 1 (coordinator, before the user's check): fix the limit in this task. Instead of atomics, the engine's Sounding gets hits: [u32;16], a wrapping count per sound bumped in Tr808::hit / Tr909's trigger tail (every trigger, retriggers included, pattern steps too). Counts are state in the triple-buffered snapshot, so no hit is lost between publishes or editor reads, and nothing is added to the audio thread beyond an increment (no locks, no allocation; Sounding stays Copy). The editor's Lights keeps the counts it saw: a changed count is an onset that restarts the 100 ms minimum; the first frame after note mode starts only takes the counts (no spurious lights). A sticky mask is not needed: any voice that sounded since the last read has an onset. Tests: embed::tests a_hit_between_two_reads_lights, a_retrigger_of_a_sounding_voice_restarts_the_light, the_counts_at_the_start_are_not_hits, minimum on-time, AC; engine tr808/tr909 note_tests every_hit_is_counted_retriggers_too. Rerun: standalone --screenshot sha256 still a3c2890e... (identical to base); cargo test 820 pass, clippy and nightly fmt clean; bundles rebuilt and validate-plugin.sh passed. Audio untouched (hits is a counter only; corpus_golden passes in cargo test). The earlier 'Known limits' note is obsolete.

Correction of earlier notes: the branch is now rebased on main 9371a35 (the 'rebased on main 1349508' line is stale); the 'Known limits' paragraph above is OBSOLETE (fixed by the hit counts of review round 1).
Review round 2 (coordinator): (1) the 16 ms repaint is now requested only while a light is lit or the engine plays in note mode (embed.rs note_lights); else the 50 ms POLL carries it, and a hit is seen by its count at the next frame (docs/plugin.md 9.4, LIGHT_POLL doc updated). CPU, editor_preview --idle (new flag: the harness asks no frames itself, so only the panel's own repaints drive it; the harness also runs the plugin core per frame), 20 s after 8 s warm-up, release, this Mac: idle note mode 1.46 s CPU (7.3%), idle pattern mode 1.00 s (5.0%), note mode with the host playing 0.79 s (4.0%), note mode with an 808 BD hit every buffer (lit all the time, 62 Hz) 2.44 s (12.2%). Idle note vs pattern differ by 0.46 s/20 s, inside this harness's noise (the playing run was lower than idle); the always-on 62 Hz redraw is gone. (2) corpus_golden after the rebase: 4 passed, 0 failed, zero differences (the engine change is a counter; audio untouched). cargo test 820 pass, clippy and nightly fmt clean, standalone screenshot sha256 a3c2890e... unchanged, bundles rebuilt and validate-plugin.sh passed.

Review round 3: docs/plugin.md 9.4 says idle note mode costs about the same as pattern mode, within the harness's noise (one run of each: 7.3% vs 5.0% of a core), not 'no more than'. Docs only; no rebuild.

Rebase onto main f84d164 (task-218 Gate mode): conflicts in tr808.rs, tr909.rs (pattern_gate and hits fields, both kept) and editor_preview.rs (the --mode latch|gate|note usage line with --note and --idle); the lights stay for Options > Note Mode only (PluginMode::Note), Latch and Gate are pattern modes. With ORB_CD_ISO set: cargo test 836 pass, 0 fail, 5 ignored; corpus_golden 4/4 pass, zero differences; clippy and nightly fmt clean; bundles rebuilt and validate-plugin.sh passed. Standalone --screenshot (909_low_tom.rbs): sha256 991be41a... both from this branch and from main f84d164 (own build), identical (it differs from the older a3c2890e only through task-218's own change).

AC #4: the user checked the b53b270 bundles in Reaper on 2026-11-16: 'lights look fine'.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Plugin note mode (Options > Note Mode) lights the sounding 808 and 909 voices. 808: a green ring of the selection ring's shape on the played sound's chip (lower legend or upper chip by the voice, whatever the switch says; on the selected chip it replaces the orange via litsel808_*; AC chip lit while a lit sound had AC). 909: the name-row LED and the AC LED turn green, replacing the red selection LED. 303s, Latch/Gate pattern modes and the preview unchanged. Lights follow the per-voice sounding mask plus a 100 ms minimum on-time restarted by every hit; the engine counts hits per sound (Sounding::hits, wrapping, in the snapshot), so a hit shorter than a frame and a retrigger of a sounding voice are seen, lock-free, audio untouched. The panel redraws every 16 ms only while a light is lit or the engine plays, else the 50 ms POLL. Green differs from orange/red by ring width, white core and glow (deuteranopia simulation checked). 36 new art parts via orb-ui-art, docs/plugin.md 9.4, editor_preview --note and --idle. Standalone --screenshot identical to main. Tests (ORB_CD_ISO set): cargo test 837 pass, corpus_golden 4/4 zero differences, clippy and nightly fmt clean, clap-validator and pluginval passed. The user approved the look in Reaper on 2026-11-16 ('lights look fine'). Branch worktree-agent-a25d385c8cb7413ea, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
