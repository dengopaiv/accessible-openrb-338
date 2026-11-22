---
id: TASK-275
title: >-
  Plugin: knob automation is stepped in Studio One (issue 21); make the 0..127
  parameters continuous
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T18:03:00Z'
updated_date: '2026-11-22T08:10:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-276
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
- [x] #1 Every 0..127 Value parameter is continuous to the host (VST3 stepCount 0, CLAP not stepped), and every discrete parameter keeps its steps; tested through the bundles, and the split listed in docs/plugin.md section 3
- [x] #2 The engine still gets integers: the value is rounded by a documented rule, and an engine message is sent only when the rounded integer changes; display and parse still use integers; tested
- [x] #3 Compatibility with the released build: same ids (param_ids.txt unchanged), same normalized mapping, and a state saved by the released build restores the same values; tested
- [x] #4 The panel-to-host path (ParamSetter), the out-of-range rule and the preview rule behave as before; the existing plugin tests and the REAPER test (when ORB_REAPER is set) pass
- [x] #5 The user checked the build in Studio One (where the released plugin's automation was stepped) before the agent review
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. params.rs: Spec::continuous() (Kind::Value with 0..=127) and params::round (round half away from zero, f32::round, clamped to the range): the one rule from the host's float to the engine's integer.
2. instance.rs: a HostParam enum (Stepped(IntParam) | Continuous(FloatParam, FloatRange::Linear min..max, no step size)); value() returns the rounded integer, so OrbParams::values() and the process side (Core::apply compares integers) send an engine message only when the rounded integer changes. display/parse wrap params::display/parse. Test helpers set_param (integer) and set_normalized (any host value).
3. editor.rs set_param: ParamSetter with the integer as f32 for a continuous parameter; the rest unchanged.
4. lib.rs Plugin::filter_state: a state saved by the released build stores every parameter as {"i32": v}; nice-plug only restores F32 into a FloatParam, so the continuous ids' I32 values become F32 before the restore (otherwise they would silently reset to the defaults).
5. Tests: tests/params.rs (split, rounding, normalized mapping identical to the released IntParam model for every integer and a dense grid of host values), tests/state.rs (a released-build state JSON with i32 values restores; round trip of a fractional value), exports.rs (VST3 stepCount and CLAP IS_STEPPED per parameter through the bundles' exports), core test for no message unless the integer changes.
6. Compatibility check in REAPER with the released v2026.1118.0919 bundle: a project with a knob envelope saved/rendered by the released build renders the same with the new build (VST3 and CLAP; CLAP's plain domain changes from 0..127 to 0..1 for nice-plug's continuous params, so check what REAPER stores).
7. docs/plugin.md sections 3 and 5: the split, the rounding rule, the filter_state migration.
8. cargo test --workspace, clippy, nightly fmt; bundles built; stop before review for the user's check (criterion 5).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a78c4a9fc52097444

Research (nice-plug 0.4.2 source, ~/.cargo/registry):
- State: nice-plug saves each parameter as an externally tagged ParamValue ({"i32": v} for an IntParam, {"f32": x} for a FloatParam; nice-plug-core src/plugin/state.rs). deserialize_object (nice-plug src/wrapper/state.rs) restores only matching pairs (FloatParam+F32, IntParam+I32) and silently skips the rest, after calling Plugin::filter_state. So a released-build state ({"i32": v} for every parameter) would leave the new FloatParams at their defaults: the fix migrates I32 to F32 for the continuous ids in filter_state (instance::migrate_state).
- Normalization: IntRange::Linear normalize (v-min) as f32/(max-min) as f32, unnormalize round(n*(max-min))+min; FloatRange::Linear normalize (v-min)/(max-min), unnormalize n*(max-min)+min. With min 0 the normalized value of every integer is the same f32, and rounding the float's plain value equals the old IntParam's unnormalize.
- VST3 (nice-plug vst3/wrapper.rs 672): stepCount = step_count().unwrap_or(0); FloatParam::step_count() is None, so 0. VST3 hosts store normalized values: unchanged.
- CLAP (nice-plug clap/wrapper.rs 3171-3213 and 10 other places): the CLAP value is normalized * step_count().unwrap_or(1), and the info range 0..step_count; IS_STEPPED when step_count is Some. A stock FloatParam is therefore CLAP range 0..1, where the released IntParam was 0..127.

FAILED (stock nice-plug, CLAP): REAPER 7.86 stores CLAP envelope points in the CLAP value domain. Experiment (scratch script exp.sh, own REAPER instances with -newinst -cfgfile, the released v2026.1118.0919 aarch64-apple-darwin bundles from GitHub Releases, sha256 bb40b6bb... checked against SHA256SUMS.txt): REAPER with the released CLAP saved latch_48k with a ramp on 'ORB-303 1 Cutoff' (param 27) from 0 at 0 s to max at 3.5 s as '<PARMENV 27:3029309 0 127 63.5' with 'PT 0 0 0', 'PT 3.5 127 0'. Loaded with the stock-nice-plug new build REAPER reported the param's range as 0..1 and evaluated the envelope as 18.14 at 0.5 s, 36.29 at 1 s, ... (clamped to 1): the render differed from the released build's in 383586 of 384000 samples, up to 0.203. The VST3 envelope is saved normalized ('<PARMENV 27:3029309 0 1 0.5', PT 0 0, PT 3.5 1) and rendered bit for bit the same.

Fix for CLAP: nice-plug 0.4.2 vendored under vendor/nice-plug ([patch.crates-io], as raw-window-metal), its CLAP wrapper patched (clap_scale in src/wrapper/clap/wrapper.rs, used in all 11 places that used step_count().unwrap_or(1), incl. context.rs's output value): a FloatParam over a linear 0..=max (max whole, >= 1) has the CLAP range 0..=max and is not stepped; stepped params as upstream. With it, the released-build CLAP project renders bit for bit as with the released build (0 of 384000 samples differ), and REAPER reports the param range 0..127 again. The envelope matters: the same project without it differs from the enveloped render in 383810 samples (up to 0.184).

Implementation:
- params.rs: Spec::continuous() (Kind::Value over 0..=127: 86 parameters; 33 stay stepped) and params::round (f32::round: nearest, a half away from zero; clamped; NaN -> min).
- instance.rs: HostParam { Stepped(IntParam), Continuous(FloatParam, Spec) }; FloatParam::new(name, default, FloatRange::Linear 0..127), no step size; value() is the rounded integer, so OrbParams::values() and Core::apply (integers compared with the call before) send an engine message only when the integer changes. set_normalized (tests), migrate_state (I32 -> F32 for the continuous ids), called by Plugin::filter_state in lib.rs.
- editor.rs set_param: ParamSetter with v as f32 for a continuous parameter, unchanged otherwise; still nothing when value() == v.
- core.rs: Core::param_messages(), a counter of the parameter messages apply_one sends (for the test of the integer rule).
- vendor/nice-plug: see above; Cargo.toml [patch.crates-io] nice-plug; Cargo.lock loses the registry source/checksum lines of nice-plug.

Tests (new): tests/continuous.rs (6: the split 86/33, the normalized mapping bit for bit against an IntParam model of the released build for every integer and 20001 grid + step + midpoint host values per parameter, the rounding rule, the text, one message per integer crossed and none while the integer holds (100 sub-integer changes: 0 messages; 60 -> 70 by tenths: 10), migrate_state). tests/exports.rs (2, through the built cdylib the bundles hold): CLAP - 119 params, IS_STEPPED exactly for the 33, range 0..(max-min) for all (0..127 for the continuous), default and values as the released IntParam gave them; a released-form state ({"i32": v} everywhere, d1c3 101, d0c7 3, d5c1 7) restores every value; a CLAP event of 63.6 reads back 63.6 and shows '64'; the saved state stores f32 for the 86 and i32 for the 33 and restores into a fresh instance with equal values. VST3 - stepCount 0 for the 86, max-min for the rest, plainParamToNormalized of every integer = (v-min)/(max-min) in f32.

Final compatibility check (final bundles, after cargo test rebuilt them; scratch repeat.sh: the REAPER-saved project of the released build rendered 3 times per build, each in a fresh resource folder):
- VST3: all 3 released and 3 new renders equal, all 9 cross pairs 0 differing samples.
- CLAP: REAPER's render of the enveloped project is not repeatable with EITHER build: each run gives one of two renders, 135321 of 384000 samples apart from frame 37 (released: run 1 = B, runs 2 and 3 = A; new: runs 1 and 3 = A, run 2 = B). Cross: released 2 = new 1 (0 differing), released 1 = new 2 (0 differing). So every new-build render is bit for bit one of the released build's. The first experiment's 'identical' result was the two builds landing on the same outcome; the later 135321 differences between rel and new were this REAPER variance, not the change. Not investigated further (the released build shows it alike; the existing REAPER fixtures have no envelopes).
- Full workspace: cargo test --workspace 1029 passed, 0 failed, 5 ignored. REAPER test (ORB_REAPER=/Applications/REAPER.app/Contents/MacOS/REAPER): 2 passed, all 22 renders (11 cases x 2 formats) match as before. clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.
- Studio One: not available here; criterion 5 is the user's check of the build and the reporter's.

Review round (the user's decision task-276, 2026-11-22, relayed by the supervisor): no vendored copy; nice-plug comes from the fork github.com/pingw33n/nice-plug, branch OpenRB-338, rev 250daefd57cdc58d41ddb25f053f1a0589f44cad (upstream 91ba75d + this task's patch).
- vendor/nice-plug removed; Cargo.toml [patch.crates-io] takes nice-plug, nice-plug-core and nice-plug-derive from git at that rev (the fork's nice-plug has path deps on the other two; nice-log comes along through the same path; nice-plug-egui 0.6.0 and nice-plug-xtask 0.1.1 stay from crates.io).
- Checked: the fork checkout (~/.cargo/git/checkouts/nice-plug-7f46df0bad08aea4/250daef) has src equal to crates.io's for nice-plug-core 0.4.2, nice-plug-derive 0.1.2 and nice-log 0.3.1, and for nice-plug 0.4.2 except wrapper/clap/wrapper.rs and context.rs, which are byte for byte this branch's former vendored files (cmp).
- cargo tree -d: every nice-plug crate is one version from one source (the fork for nice-plug, nice-plug-core, nice-plug-derive, nice-log); Cargo.lock holds one entry each.
- docs/plugin.md: 'The vendored nice-plug' became 'The forked nice-plug' (where, the base commit, the patch, how to move to a new upstream); task-276 Done with the outcome.

Checks after the switch to the fork: nightly fmt --check clean; clippy --all-targets -D warnings clean; cargo test --workspace 1029 passed, 0 failed, 5 ignored; bundles rebuilt (cargo xtask bundle orb-plugin --release); exports tests 6 passed (incl. the two of this task); REAPER test (ORB_REAPER) 2 passed. Released-build cross check with the fork bundles (3 new renders per format vs the earlier 3 released ones): VST3 all 9 pairs 0 differing; CLAP the 3 new renders equal each other and released run 1 (0 differing), and differ from released runs 2 and 3 by REAPER's two-outcome variance (135321 samples), as recorded before.

User sign-off (2026-11-22, "looks good", relayed by the supervisor): the user checked the CI release build of 1af2c05 (run 44999673403, OpenRB-338-2026.1122.0745-x64-setup.exe) in Studio One 7.2.3 on Windows. With the released plugin they had seen the stepped automation; this build fixes it (VST3). Studio One 7.2.3 offers no CLAP (no CLAP path in its settings, only the VST3 in the browser), so CLAP was not part of that check. The reporter was not asked separately, since the user ran the Studio One check; the reply on GitHub issue 21 comes after the merge. Criterion 5 reworded to what happened (the user's check in Studio One before the agent review) and checked.

Review round 1 (the reviewer's send-back, relayed by the supervisor, 2026-11-22): backlog text only.
- Criterion 5 was reworded. The original text: "The user checked the build, and the reporter was asked to check it in Studio One, before the agent review". It was changed at the supervisor's instruction on 2026-11-22 because the user ran the Studio One check themselves: the CI build of 1af2c05 (run 44999673403), Studio One 7.2.3 on Windows, "looks good". The reporter is asked through the supervisor's reply on issue 21 after the merge, which needs the user's approval.
- task-276: the reference to the no longer existing vendor/nice-plug/src/wrapper/clap/wrapper.rs is replaced by the fork's branch; "The vendored nice-plug" became "The forked nice-plug"; the description is marked as the options as they stood on 2026-11-22 (option 1 then meant vendoring), the outcome being the GitHub fork (its final summary).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The 86 plugin parameters over 0..127 (knobs, faders, levels, pans, sends, shuffle) are continuous for the host: nice-plug FloatParams, linear over 0..127, no step size (VST3 stepCount 0, CLAP not stepped). The 33 discrete ones (switches, PCF mode, patterns, delay steps, PCF wave, inserts) stay stepped IntParams. The engine, panel and display get the host value rounded to the nearest integer (half away from zero, clamped); the process side compares integers, so a host ramp sends one engine message per integer crossed.

Compatibility with v2026.1118.0919: ids unchanged (param_ids.txt untouched); VST3 normalized mapping bit for bit the released IntParam's; a released state ({"i32": v}) restores through Plugin::filter_state (I32 -> F32 for the continuous ids; nice-plug would otherwise skip them). CLAP needed a patch: stock nice-plug gives a continuous FloatParam the CLAP range 0..1, and REAPER saves CLAP automation as CLAP values (0..127), so released CLAP projects broke. Per the user's decision (task-276, 2026-11-22) nice-plug, nice-plug-core and nice-plug-derive come from the fork github.com/pingw33n/nice-plug, branch OpenRB-338, rev 250daefd (upstream 91ba75d + clap_scale, which keeps the CLAP range 0..127). Checked in REAPER 7.86 with the released bundles: a project with a cutoff envelope saved through the released build renders bit for bit the same with the new build (VST3 every run; CLAP each run equals one of the two renders REAPER's CLAP host gives for that project with either build).

User check (2026-11-22): the CI release build of 1af2c05 (run 44999673403) fixes the stepped automation in Studio One 7.2.3 on Windows ("looks good"; VST3, Studio One 7.2.3 has no CLAP). Criterion 5 was reworded at the supervisor's instruction that day, from "the user checked the build, and the reporter was asked to check it in Studio One" to the user's own Studio One check; the reporter is asked through the supervisor's reply on issue 21 after the merge, with the user's approval.

Branch worktree-agent-a78c4a9fc52097444, one commit. Tests: cargo test --workspace 1029 passed, 0 failed, 5 ignored (new: tests/continuous.rs 6, tests/exports.rs 2); REAPER test with ORB_REAPER 2 passed (22 renders); clippy and nightly fmt clean. A state saved by this build loaded in the released build loses its continuous values (no downgrade).
<!-- SECTION:FINAL_SUMMARY:END -->
