---
id: TASK-291
title: The plugin reports its release version to the host, not 0.1.0
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-24T12:48:00Z'
updated_date: '2026-11-24T14:48:00Z'
labels:
  - player
milestone: m-5
dependencies: []
references:
  - crates/orb-plugin/src/lib.rs
  - crates/orb-ui/build.rs
  - crates/orb-ui/src/version.rs
  - crates/orb-plugin/src/instance.rs
  - docs/plugin.md
priority: medium
type: enhancement
ordinal: 266700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-24.
- The plugin's `Plugin::VERSION` is `env!("CARGO_PKG_VERSION")` (crates/orb-plugin/src/lib.rs:158), which is always `0.1.0`: the workspace version in Cargo.toml never changes.
- Releases are numbered by UTC build time instead, e.g. `2026.1118.0919` (docs/building.md, scripts/version.sh). The About box shows that number through orb-ui's `ORB_UI_VERSION`, set by crates/orb-ui/build.rs:91 (version.rs).
- Hosts show the plugin's version in their plugin lists, info dialogs and crash reports, and some use it to notice an updated plugin.
- With 0.1.0 nobody can tell which build a user runs. Example: GitHub issue 23, a user whose Ableton Live crashes, sent a file name that matched no build.

**What:** the plugin tells the host exactly the About box's version string (`version::version_string`, crates/orb-ui/src/version.rs:68), e.g. `2026.1118.0919 (abc1234)` for a release or a main build, `2026.1124.1200 (dev-7429b9d)` off main, with `-dirty` when uncommitted (the user's decision, 2026-11-24: release versions are known, so a local main build looking like one is no problem). No new marking of dev builds.
- The VST3 version string the host reads (nice-plug's `VERSION`, and the class info's version) must be a `&'static str`: take it from the build script's environment, e.g. orb-ui's `ORB_UI_VERSION` re-exported, or the plugin's own build.rs.
- Check that hosts accept the string with its parenthesized git part (some parse a leading dotted version). Record what REAPER shows. If a host rejects it, report back instead of changing the format.
- **State compatibility (the plugin is released):** the saved state carries the plugin's version (nice-plug's PluginState.version). Check that nothing compares it, e.g. migrate_state in crates/orb-plugin/src/instance.rs, whose own STATE_VERSION is separate. Check that a state saved by a 0.1.0 build loads unchanged, with a test.
- docs/plugin.md says where the version comes from.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A state saved by a build that reported 0.1.0 loads unchanged; a test covers it
- [x] #2 docs/plugin.md describes the version's source
- [x] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #4 The version read by pluginval's VST3 host (--verbose) matches the About box
- [x] #5 The VST3 the host loads reports exactly the About box's version string, checked in a built bundle (e.g. with the validator or REAPER's plugin info) and recorded in the notes
- [x] #6 A dev build (dev- or -dirty in the string) still scans and loads in hosts
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Plugin::VERSION = orb_ui::about::VERSION (the About box's string). 2. Check nothing compares the state's version. 3. Tests: state of a 0.1.0 build migrates as any other; the VST3 class info carries the string. 4. docs/plugin.md section 2.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-acd93ee90947c41ff

VERSION is now orb_ui::about::VERSION (crates/orb-plugin/src/lib.rs). nice-plug writes PluginState.version (state.rs:85) and never compares it; migrate_state/STATE_VERSION (instance.rs) are separate, so a 0.1.0 state loads unchanged (test the_states_version_string_does_not_matter in tests/continuous.rs, beside the 0.1.0 released-state test). exports.rs checks the class info version through the VST3 C API; first try compared it exactly and failed, because the cdylib and the test binary are built at different minutes (HHMM differs), so it now checks the shape and the git part. Built bundle target/bundled/OpenRB-338.vst3: binary holds '2026.1124.1431 (dev-902f119-dirty)'. pluginval (scripts/validate-plugin.sh) passes with the dev-/-dirty string (AC 6 on macOS). REAPER.app is installed but I did not run it (user's session may be open); AC 4 and a host check are for the user.

Review/finish: the user chose pluginval as the host check (2026-11-24). pluginval 1.0.4 --verbose --skip-gui-tests --validate on target/bundled/OpenRB-338.vst3 (c92ede6), run by the supervisor: JUCE's VST3 host printed 'OpenRB-338: OpenRB-338 v2026.1124.1431 (dev-902f119-dirty)' (JUCE adds the leading v); strictness 5 passed. Criterion 4 reworded accordingly. Final run on main 902f119: cargo test --workspace 1110 pass, 0 fail; clippy macOS and x86_64-pc-windows-msvc clean; nightly fmt --check clean.

Review round 1 (send back, fixed): the state-version test now runs both a 0.1.0-labelled and a new-version state through OrbPlugin::filter_state plus the parameter and field restore (nice-plug's deserialize_object is crate-private, so the restore mirrors it as tests/state.rs does) and asserts the same parameters, fields and song; exports.rs comment says the build minute is not compared and why, and the redundant assert is gone (the expected side is unwrapped instead); commit body test count corrected. AC 6 caveat: it rests on pluginval alone. pluginval's JUCE VST3 host loaded the dev-/-dirty build; REAPER and other hosts were not run with this build. The user is asked whether that is enough.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Plugin::VERSION is orb_ui::about::VERSION, the About box's string, so the VST3 class info and the saved state carry e.g. 2026.1124.1431 (dev-902f119-dirty) instead of 0.1.0. Nothing compares the state's version; a 0.1.0 state loads unchanged (test the_states_version_string_does_not_matter). exports.rs checks the class info version's shape and git part through the VST3 C API. docs/plugin.md section 2 states the source. pluginval's VST3 host read 'v2026.1124.1431 (dev-902f119-dirty)' and passed at strictness 5. cargo test --workspace 1110 pass, clippy (macOS, Windows) and nightly fmt clean. Branch worktree-agent-acd93ee90947c41ff, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
