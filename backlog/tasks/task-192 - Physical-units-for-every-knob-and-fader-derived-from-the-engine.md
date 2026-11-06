---
id: TASK-192
title: Physical units for every knob and fader, derived from the engine
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
updated_date: '2026-11-06T06:40:00Z'
labels:
  - ui
  - engine
  - docs
milestone: m-7
dependencies:
  - TASK-186
  - TASK-181
references:
  - crates/orb-engine/src
  - crates/orb-engine/src/tb303.rs
  - crates/orb-engine/src/tr808.rs
  - crates/orb-engine/src/tr909.rs
  - crates/orb-engine/src/drums.rs
  - crates/orb-engine/src/pcf.rs
  - crates/orb-engine/src/delay.rs
  - crates/orb-engine/src/dist.rs
  - crates/orb-engine/src/comp.rs
  - crates/orb-engine/src/process.rs
  - crates/orb-engine/src/seq.rs
  - crates/orb-engine/src/tables/mod.rs
  - crates/orb-engine/src/tables/data.rs
  - crates/orb-engine/src/song.rs
  - crates/orb-ui/src/behavior.rs
  - docs/initial_investigation.md
  - docs/export_state.md
  - docs/accessibility_plan.md
  - docs/accessibility.md
priority: medium
type: feature
ordinal: 215000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Planned in docs/accessibility_plan.md as A11Y-14; the analysis is docs/accessibility.md. Read both before starting.

- Why: the user decided (D5) that values are spoken in physical units, "as precise as possible
  as we infer them from the code". The engine reproduces the original bit for bit, so the code
  that turns each 0..127 value into a DSP parameter is the best source there is; the original
  panel shows no units.
- What: for each knob and fader (77 knobs, 9 faders; `layout.tsv`), find in the engine how its
  value becomes a parameter, and derive what a musician would want to hear:
  - frequencies in Hz or kHz (the 303's cutoff, from its filter coefficient table
    `FILT[reso >> 4][cutoff]` by the pole frequency of the 2-pole sections; the PCF's frequency;
    tunings as semitones and cents from the pitch tables);
  - times in ms or s (the decays: the time to fall 60 dB from the per-sample or per-pair gain
    multipliers, `TR808_DECAY`, `PCF_DECAY`, the 909's decay table; the 909 attack; the delay
    time from the steps and the tempo);
  - levels in dB (the faders' and level knobs' gains, the compressor's threshold, the accent
    levels), and ratios as "4 to 1";
  - pan as left/right percent ("left 30", "center");
  - shuffle as the delay of the off-beat sixteenths in percent of a sixteenth (or ms at the
    tempo);
  - quantities with no physical unit (resonance, distortion shape and amount, envelope
    modulation depth if it has no clean frequency range) as percent of their range.
  Each mapping is a function in `crates/orb-ui/src/a11y/units.rs` (computed in f64 from the same
  tables the engine uses, imported from orb-engine, not copied), and is marked in a new
  knowledge document `docs/control_units.md` with: the control, the formula, the code it comes
  from (engine function and the original's address as the engine's comments give it), and how
  exact it is: [C] closed form from the engine's own formula, [D] derived numerically from its
  tables or by measuring a render (state the measurement), [G] estimated (say from what). The
  spoken text rounds to a sensible precision (3 significant digits for Hz and ms, 0.1 dB) and
  says the unit in words NVDA reads well ("1.24 kilohertz", "350 milliseconds", "minus 6.5
  decibels"). A setting "Speak raw values" (off by default, `ui.toml`, Options menu in screen
  reader mode) adds the 0..127 number ("1.24 kilohertz, 64"). The engine is not changed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `docs/control_units.md` lists every knob and fader with formula, source and its [C], [D] or [G] mark; a test checks that the list and `layout.tsv` agree.
- [x] #2 Tests: for each [C] and [D] mapping, sample values (0, 64, 127 and two others) give the documented numbers; for at least the 303 cutoff and one decay, the number is checked against a measurement of an engine render (a sine or impulse through the filter, a decaying voice) within the stated tolerance.
- [x] #3 The spoken value of every knob and fader uses the unit; the "Speak raw values" setting adds the number; with screen reader mode off nothing changes.
- [x] #4 The rendered audio is unchanged: `cargo test --release -p orb-engine --test corpus_golden` passes (no engine file is modified).
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. crates/orb-ui/src/a11y/units.rs: one mapping per knob and fader family (86 controls), computed in f64 from orb-engine's public tables and functions (tables::Tables, tables::data, tables::tr909_decay, seq::shuffled, pcf::smooth_cutoff/env_index, comp::params, Tb303::note_on/process). A few scalar formula constants that the engine keeps private (303 cutoff/envmod scale and base, 808/909 accent step, OH/SD noise multiplier 0.998, flam divisor, master x2) are restated with their addresses, and each is checked by a unit test against the engine's public behavior (no engine file changes). Context: the 303 cutoff and accent read the 303's resonance and the sound model, the PCF frequency the Q fader, the compressor ratio the threshold, the flam the tempo; without a panel the engine's ctor defaults.
2. Definitions: frequencies as pole frequency of the 2-pole table entry (theta = acos(c0/(2 sqrt c1))), the 303 cutoff at the envelope's neutral point (e = 0.4, no accent), drum tone/attack as the one-pole -3 dB point, decays as time to fall 60 dB from the per-pair / per-4-sample / per-block multipliers (plus hold blocks), levels in dB relative to the knob's maximum (gain ~ L^2), mixer level/send/master from T[] (master x2), tunings in semitones and cents from T808/T909, sample-switch knobs as "sample N of M", shuffle as percent of a sixteenth, compressor threshold as the knee level in dB full scale and ratio as the static curve's slope at the knee, no-unit quantities in percent.
3. names.rs: control_value_text keeps its signature (units, engine defaults); value_text_as(panel, c, Speech) with Speech = Plain (screen reader mode off: task-186's text) / Units / UnitsRaw ("1.24 kilohertz, 64"). a11y::describe reads the style from the egui context, set by App::show each frame from screen_reader_mode() and the setting.
4. Settings: speak_raw_values (ui.toml, default false); Options > "Speak Raw Values" only in screen reader mode.
5. docs/control_units.md: every knob and fader with formula, source (engine function, original's address), mark [C]/[D]/[G], and the spoken text at 0, 32, 64, 96, 127. Tests: the doc's ids equal layout.tsv's knobs and faders, the marks equal the code's, the sample texts equal the code's output.
6. Measurement tests (crates/orb-ui/tests/units_measure.rs, release): the 303 cutoff against the resonance peak of a Tb303 render (saw, resonance 127, envelope pinned at 0.4), and the 808 bass drum decay against the time to fall 60 dB of a Tr808 render with a constant synthetic sample; tolerances stated.
7. Update docs/ui_spec.md section 20, docs/accessibility_plan.md where needed, key_help if it lists the Options items; tests; corpus golden in release; fmt, clippy.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa5414e6f1b26cda0

Research (no IDA on this machine: the engine's Rust code and its address comments, docs/initial_investigation.md 4, 5, 6, 7, 8):
- 86 controls: 77 knobs, 9 faders (layout.tsv, kind knob/fader). Every mapping in docs/control_units.md with formula, source and mark.
- Public paths used (no engine file changed): Tables::new() (FILT, PCF LP, T[], PAN, T808, T909, DT, smoothers), tables::data (TB303_TUNE, TR808_TONE, TR808_DECAY, TR909_BD_ATTACK, PCF_DECAY), seq::shuffled, pcf::smooth_cutoff and env_index, comp::params, Tb303::new/note_on/process (pub fields env_decay, acc_env, coefs), Tr808/Tr909 step with pub fields (accent, simple/dual voices' m/hold/g, flam_tick).
- Private scalar constants restated in units.rs with their addresses, each checked by a unit test against the engine's behavior: 303 CUT_SCALE/CUT_BASE (dbl_10153B80/88) and EM_SCALE/EM_BASE (dbl_10153B90/98) through Tb303::process at resonance 0 (the coefs it looks up), the drum accent step 0.03125 through Tr808/Tr909::step (accent field), the 0.998 noise multiplier and the holds through the voices' m/hold after a step, the flam divisor 700000 through Tr909::step (flam_tick). The master's x2 (flt_10151F40) has no public path short of a full frame; it is the documented [C] constant of docs/initial_investigation.md 4.
- Choices: the 303 cutoff is the pole frequency at the envelope's neutral point e = 0.4 (where (e - 0.4)*em is 0, so the envmod does not enter) and no accent, at the synth's resonance and sound model; the envmod is the sweep span em*log2(1602) octaves; the 303 decay is the filter envelope's 60 dB time (the 303 envelope drives the cutoff, not the amp). Resonance, PCF Q and amount, distortion amount and shape are percent (no clean unit: Q depends on the cutoff index through q, the PCF amount on the pattern's velocities).
- Fact: a FILTER drum voice ends when its gain falls below 3e-8 (drums.rs SILENT, 0x1001E7E2 / flt_1015364C), which for a full-level 808 bass drum (g = 0.635*0.999/32768 = 1.94e-5) is about 56 dB down, so the render never reaches -60 dB: the decay test measures the 40 dB point (two thirds of T60 for an exponential).
- Speech gating: AC3 "with screen reader mode off nothing changes" is read as: the mode off keeps task-186's text (the number, pan "left 20"); units only in screen reader mode (App::speech: Plain / Units / UnitsAndRaw), put into the egui context each frame (a11y::set_speech) and read by a11y::describe before the node builder (which holds the context's write lock; reading inside it would deadlock).

Work log:
- Failed hypothesis: the PCF base settles at 4*freq. Running pcf::smooth_cutoff with its f32 store to rest gives one f32 step below 4*freq for every freq 1..127, so the resting index is 4*freq - 1 (127 of 128 values; 0 stays 0). Recorded in docs/initial_investigation.md 8.2 [D] and docs/control_units.md; the spoken PCF frequency uses the settled value.
- First draft bug: the family of `tb303_1` was trimmed to `tb` (digits and underscores trimmed from the end); fixed to the names.rs rule.
- Measurements (crates/orb-ui/tests/units_measure.rs, release, 2026-11-06):
  303 cutoff vs the resonance peak of a Tb303 render (saw at 16.35 Hz, envelope pinned at 0.4, Hann window over 65536 samples, saw 1/k, pre-filter c3 and DC blocker divided out, parabola on log magnitude): cutoff 0/40/64/100/127 at reso 127: spoken 326.7/643.6/963.9/1766.8/2783.1 Hz, peak 325.1/642.4/963.3/1766.2/2782.5 Hz (0.49/0.19/0.06/0.03/0.02 %); cutoff 64 reso 112: 918.7 vs 917.0 Hz (0.18 %). Tolerance stated: 2 %.
  808 BD decay vs a Tr808 render (constant sample, level 127, tone open): the 40 dB time x 1.5 equals the spoken 60 dB time to 0.1 ms for decay 0/40/64/100 (0.3131/0.9011/1.4422/4.4768 s). Tolerance stated: 1.5 blocks. The 60 dB point is not reachable in a render: the FILTER voice stops at g < 3e-8, ~56 dB down at full level.
- Reproduce: cargo test --release -p orb-ui --test units_measure -- --nocapture; sample table: cargo test -p orb-ui --lib units::tests::print_sample_table -- --ignored --nocapture.
- Tests (release): cargo test --release -p orb-ui --no-fail-fast: every binary passes (lib 95 + 2 ignored, a11y_names 6, a11y_menu_bar 8, units_measure 2, menu 54, ...) except the known crash.rs a_deliberate_panic_exits_101_and_writes_the_log (TASK-200, not this task). cargo test --release -p orb-engine --test corpus_golden: 4 passed (diag, app at fitted N, shipped, shipped-out-of-corpus), no skip messages; git diff main -- crates/orb-engine is empty. cargo clippy --workspace --all-targets -D warnings clean; nightly-2026-10-18 fmt --check clean.
- For TASK-187 (parallel): control_value_text(id, v) keeps its signature and now gives the unit with the engine's ctor context; a caller with the panel should use names::control_value_text_in(panel, id, v, speech) (speech from a11y::speech(ctx)) so the cutoff/accent/PCF/ratio/flam follow their context and the mode.

Review round 1 (send back, AC1-AC4 met in substance): 1. control_units.md said 157 ms for the open hi-hat's 0.998 fall, the table and code 156 ms; 2. undefined k in the 808 BD gain formula of the facts; 3. names::value_text and control_value_text spoke units whatever the mode and with the default context: AC3 not safe by construction; 4. the PCF "every frequency 1..127" claim was printed, not asserted; 5. (optional) test the 1602 span and the 2.0 interpolation at a fractional resonance, engine file refs on the restated constants.
Fixes: 1. 156 ms everywhere. 2. k = f32(1/127) defined, 0.005 = LEVEL_SCALE (tr808.rs). 3. One entry point each: names::value_text(panel, c, speech) and names::control_value_text(panel, id, v, speech); the context-free and always-units versions are gone (callers: a11y::describe, the tests). 4. The test asserts that exactly v = 0 rests at index 4v and v = 1..127 at 4v - 1. 5. Tests added: the FILT row's pole frequencies span 1602^(di/1023) between two indices (0.1 %), the 2.0 engine's interpolated coefficients at resonance 64 and 100 equal tb303_coefs; the low-end precision of the pole frequency from f32 coefficients checked against the closed form w*44100 for the PCF table; restated constants name their engine file.
- Low-end check: the pole frequency from the f32 PCF coefficients is within 1.9 %% of the closed form w*44100 at index 3 (about 9 Hz), the worst case (documented in control_units.md Definitions).
- Round 1 results (release): cargo test --release -p orb-ui --no-fail-fast: lib 98 passed + 2 ignored, a11y_names 6, a11y_menu_bar 8, units_measure 2, menu 54, every other binary passes; only the known crash.rs a_deliberate_panic_exits_101_and_writes_the_log fails (TASK-200). clippy --workspace --all-targets -D warnings clean; nightly-2026-10-18 fmt --check clean. Commit amended.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Every knob and fader (77 knobs, 9 faders) speaks a physical unit in screen reader mode, derived from the engine's code (decision A11Y-D5).

- crates/orb-ui/src/a11y/units.rs: one mapping per control family, computed in f64 from orb-engine's public tables and functions (FILT, PCF tables, T[], T808/T909, DT, TONE8, DEC16, C8, PCF_DECAY, TB303_TUNE, seq::shuffled, pcf::smooth_cutoff/env_index, comp::params, Tb303::note_on/process). Private scalar constants (303 cutoff/envmod scale and base, drum accent step, 0.998 noise multiplier and holds, flam divisor) are restated with addresses and each checked by a unit test against the engine's behavior; master x2 is the documented [C] constant. All mappings are [C] except the 303 cutoff, the 303 accent and the PCF frequency ([D]); none is [G].
- Units: Hz (pole frequency of the 2-pole table entry; one-pole -3 dB point or "open"), 60 dB decay times, dB (drum levels 40 log10 of the effective level, T[] for mixer/send, 2T[] master, accents, feedback per repeat, compressor knee in dBFS), semitones and cents, "sample N of M", percent of a sixteenth (shuffle), "N to 1" (compressor slope above the knee), pan as percent of travel, percent for resonance/Q/amounts/shape. Context: 303 cutoff and accent follow the synth's resonance and the sound model, PCF frequency the Q fader, ratio the threshold, flam the tempo.
- names.rs: Speech (Plain / Units / UnitsAndRaw); one entry point each, control_value_text(panel, id, v, speech) and value_text(panel, c, speech), so no caller gets units with the mode off or without the panel's context (review round 1); units_context. a11y::set_speech/speech carry the style in the egui context; App::show sets it from screen_reader_mode() and Settings::speak_raw_values; with the mode off the text is task-186's.
- Settings speak_raw_values (ui.toml, default false); Options > "Speak Raw Values" only in screen reader mode.
- docs/control_units.md (new): definitions, every control with formula, source and mark, sample texts at 0/32/64/96/127, the render checks, facts found. docs/ui_spec.md section 20 and the Options table; docs/initial_investigation.md 8.2: the PCF's resting index is 4*freq - 1 [D].
- Measured: 303 cutoff vs render resonance peak within 0.5 % (tolerance 2 %); 808 BD decay vs render within 0.1 ms (tolerance 1.5 blocks).
- Tests (release): orb-ui all pass except the known crash.rs failure (TASK-200); corpus_golden 4 passed, engine untouched; clippy and nightly fmt clean.
Branch worktree-agent-aa5414e6f1b26cda0, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
