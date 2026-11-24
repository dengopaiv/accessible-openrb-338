---
id: TASK-291
title: The plugin reports its release version to the host, not 0.1.0
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T12:48:00Z'
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

**What:** the plugin tells the host the release version, the same number the About box shows (`2026.1118.0919` for a release). A dev build gets the version_string form of a dev build, so it is never mistaken for a release.
- The VST3 version string the host reads (nice-plug's `VERSION`, and the class info's version) must be a `&'static str`: take it from the build script's environment, e.g. orb-ui's `ORB_UI_VERSION` re-exported, or the plugin's own build.rs.
- Check what string shape hosts accept: some parse a dotted version for update checks. `2026.1118.0919` is dotted numbers. A dev build's suffix must not break a host's scan. Record what REAPER shows.
- **State compatibility (the plugin is released):** the saved state carries the plugin's version (nice-plug's PluginState.version). Check that nothing compares it, e.g. migrate_state in crates/orb-plugin/src/instance.rs, whose own STATE_VERSION is separate. Check that a state saved by a 0.1.0 build loads unchanged, with a test.
- docs/plugin.md says where the version comes from.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The VST3 the host loads reports the same version the About box shows (release: YYYY.MMDD.HHMM), checked in a built bundle (e.g. with the validator or REAPER's plugin info) and recorded in the notes
- [ ] #2 A dev build reports a version that is clearly not a release, and hosts still scan it
- [ ] #3 A state saved by a build that reported 0.1.0 loads unchanged; a test covers it
- [ ] #4 docs/plugin.md describes the version's source
- [ ] #5 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #6 The user has seen the version in a host
<!-- AC:END -->
