---
id: TASK-192
title: Physical units for every knob and fader, derived from the engine
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-05T19:32:00Z'
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
- [ ] #1 `docs/control_units.md` lists every knob and fader with formula, source and its [C], [D] or [G] mark; a test checks that the list and `layout.tsv` agree.
- [ ] #2 Tests: for each [C] and [D] mapping, sample values (0, 64, 127 and two others) give the documented numbers; for at least the 303 cutoff and one decay, the number is checked against a measurement of an engine render (a sine or impulse through the filter, a decaying voice) within the stated tolerance.
- [ ] #3 The spoken value of every knob and fader uses the unit; the "Speak raw values" setting adds the number; with screen reader mode off nothing changes.
- [ ] #4 The rendered audio is unchanged: `cargo test --release -p orb-engine --test corpus_golden` passes (no engine file is modified).
<!-- AC:END -->
