---
id: TASK-275
title: >-
  Plugin: knob automation is stepped in Studio One (issue 21); make the 0..127
  parameters continuous
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-21T18:03:00Z'
labels:
  - player
milestone: m-7
dependencies: []
references:
  - https://github.com/pingw33n/OpenRB-338/issues/21
  - docs/plugin.md
  - crates/orb-plugin/src/instance.rs
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/tests/param_ids.txt
  - crates/orb-plugin/tests/params.rs
priority: high
type: bug
ordinal: 259700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Report:** GitHub issue 21, 2026-11-21, by a user on Windows 10 with Studio One 7 and Studio Pro, VST3. Automation of a knob ("ORB-303 2 Cutoff") can't be continuous: the lane is a staircase and can't be drawn as a ramp. The same host draws a smooth lane for another synth's filter.

**Cause [C from the code]:** every parameter is a nice-plug `IntParam` (crates/orb-plugin/src/instance.rs, `OrbParams::new`; docs/plugin.md section 3). For VST3 that sets the parameter's `stepCount` to `max - min`, 127 for a knob, and CLAP marks an integer parameter as stepped. Studio One treats a parameter with a step count as discrete and draws and plays its automation as steps. REAPER and Ableton don't, which is why our tests didn't catch it.

**Fix:**
- **The `Kind::Value` parameters with a 0..127 range** (the knobs, faders, levels, pans, sends and the shuffle; crates/orb-plugin/src/params.rs `SPECS`) become continuous for the host: a nice-plug `FloatParam`, linear over min..max, with no step count.
  - The plugin rounds the value to the nearest integer (round half away from zero, or document the rule) before it goes into the engine and the panel. The engine's values are still the 128 integers.
  - The display and parse functions (`params::display`, `params::parse`) still show and accept the integer.
- **The discrete parameters stay stepped `IntParam`s:**
  - the switches (`Kind::Switch`, `Kind::PcfMode`);
  - the patterns 0..31;
  - the delay steps 1..32;
  - the PCF wave 0..54;
  - the compressor insert 0..5 and the PCF insert 0..4;
  - any other Value whose range isn't 0..127. List the split in docs/plugin.md section 3.
- **Compatibility:** the plugin is released (v2026.1118.0919), so projects saved with it must keep their automation and state.
  - The parameter ids don't change (`tests/param_ids.txt`).
  - The normalized mapping stays `(v - min) / (max - min)`, for both the old IntParam and the new FloatParam, so a host's saved automation and parameter values mean the same values. Check it with a test.
  - Check that the plugin state (section 5) still loads values saved by the released build. Whether it stores the parameters as integers or normalized, it must round-trip.
- **Everything else is unchanged:**
  - a change made on the panel still goes to the host through `ParamSetter`, now as the float of the integer;
  - the "send only when the engine doesn't already play it" rule (section 3, out-of-range bytes);
  - the preview's keep-and-send rule.
  A host ramp makes many small value changes; only those that change the rounded integer may send an engine message, so a ramp doesn't flood the engine with repeats.

**Verification:**
- **Unit and integration tests:**
  - every 0..127 Value parameter reports no step count (VST3 `ParameterInfo.stepCount == 0`; CLAP: not `CLAP_PARAM_IS_STEPPED`), and every discrete one keeps its steps, through the exported bundles the way `tests/exports.rs` does;
  - the rounding, and the "no message unless the integer changes" rule;
  - normalized values are mapped the same as the released build's.
- **The REAPER harness** (`ORB_REAPER`, docs/plugin.md 7.1): a ramp on a knob envelope renders as the expected sequence of integer values, or at least stays unchanged against its reference.
- **Studio One:** we don't have it, so the supervisor asks the user and the issue's reporter to check a build.

After the merge the supervisor proposes a reply on issue 21 to the user. Nothing is posted without the user's word.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every 0..127 Value parameter is continuous to the host (VST3 stepCount 0, CLAP not stepped), and every discrete parameter keeps its steps; tested through the bundles, and the split listed in docs/plugin.md section 3
- [ ] #2 The engine still gets integers: the value is rounded by a documented rule, and an engine message is sent only when the rounded integer changes; display and parse still use integers; tested
- [ ] #3 Compatibility with the released build: same ids (param_ids.txt unchanged), same normalized mapping, and a state saved by the released build restores the same values; tested
- [ ] #4 The panel-to-host path (ParamSetter), the out-of-range rule and the preview rule behave as before; the existing plugin tests and the REAPER test (when ORB_REAPER is set) pass
- [ ] #5 The user checked the build, and the reporter was asked to check it in Studio One, before the agent review
<!-- AC:END -->
