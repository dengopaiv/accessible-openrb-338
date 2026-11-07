---
id: TASK-213
title: 'Knobs and faders: an arrow key moves to the next value that sounds different'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-07T09:39:00Z'
labels:
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/a11y/drive.rs
  - crates/orb-ui/src/a11y/names.rs
  - crates/orb-ui/src/a11y/units.rs
  - docs/ui_spec.md
priority: medium
type: enhancement
ordinal: 231000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user (NVDA, 2026-11-07): "The sliders, that are for example in the tr808 section, don't announce every move by arrow." In screen reader mode Up / Down move a knob or fader one position of 0..127 (crates/orb-ui/src/a11y/drive.rs `step`, `Target::Slider`), and the node's value is the physical unit (task-192, `names::control_value_text`, `units::text`). On many controls neighboring positions have the same text, so the value does not change and NVDA says nothing. Measured by the supervisor with Panel::demo() and Speech::Units, counting v in 0..127 where text(v) == text(v+1): tr808.bd_tone and cy_tone 120 of 127 ("42.9 hertz"), bd_decay and cy_decay 112, sd_tone 123 ("sample 1 of 5"), oh_level 64, ch_level 43, ac_level 20, the tunings 12; tr909 bd_tune 124, bd_attack 120, the decays 112, flam 102, sd_tune 123; tb303 tune 103, reso 27, accent 29; transport.shuffle 106; comp.ratio 70; pcf.dec 112; pcf.q, pcf.amt, dist.amount, dist.shape 27. Mostly the engine has only a few distinct values there (a decay changes every 16 positions, so those positions sound the same); some are rounding of the spoken text ("2 percent" twice).

The user's decision (2026-11-07): an arrow press jumps to the next new value.

What, in screen reader mode only:
- Up / Down on a knob or fader, and NVDA's Increment / Decrement requests (drive.rs `request`), move to the nearest position in that direction whose spoken unit text (the Speech::Units text in the panel's current context) differs from the current one. At an end with no different text further on, nothing moves.
- Page Up / Page Down: move at least 8 positions as now, then on to the next different text if the text has not changed; clamp at the ends.
- Home / End, and SetValue requests (a typed value), unchanged.
- With Options > Speak Raw Values on, every press moves one position as now (the number changes on every press, so each is spoken, and it keeps the fine positions reachable by keys). Say it in the key help and docs.
- With screen reader mode off nothing changes (these keys are screen reader mode's).
- Every move still goes through `set_slider` (the mouse's path), so the engine events and song-mode recording are the mouse's.

Code: crates/orb-ui/src/a11y/drive.rs (`step`, `request`; they need the panel's speech style or at least whether raw values are on, and the text function; keep `Panel`-only callers working), the caller in app.rs, crates/orb-ui/src/a11y/names.rs `control_value_text`. Docs: docs/ui_spec.md section 20 where the slider keys are described, key_help.rs rows, the README's screen reader section, docs/accessibility.md if it states one position per key. By-ear check: a new unchecked item of the NVDA checklist in docs/accessibility_plan.md 2.2 (the next free number, 27).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 In screen reader mode, Up on tr808.bd_tone from 0 changes the spoken text in one press and crosses the range in as many presses as the knob has distinct texts; Down likewise; tests cover a quantized knob (bd_tone or a decay), a rounding case (pcf.q), and a control whose every position differs (tb303 cutoff moves one position)
- [ ] #2 Page Up / Page Down move at least 8 positions and land on a different text; at the ends nothing moves; Home / End and SetValue are unchanged (tests)
- [ ] #3 With Speak Raw Values on, and NVDA's Increment / Decrement with it on, a press moves one position as before; with screen reader mode off the keys and the panel behave as upstream (tests)
- [ ] #4 docs/ui_spec.md, the key help and the README describe it; the by-ear check is a new item of the NVDA checklist
<!-- AC:END -->
