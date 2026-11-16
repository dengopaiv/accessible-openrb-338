---
id: TASK-220
title: 'Plugin: ORB-303/ORB-808/ORB-909 in parameter names, description "OpenRB-338"'
status: Done
assignee:
  - '@agent-chore'
created_date: '2026-11-16T15:44:00Z'
updated_date: '2026-11-16T15:54:00Z'
labels:
  - player
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/params.rs
  - crates/orb-plugin/src/lib.rs
priority: medium
type: chore
ordinal: 227000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why (the user, 2026-11-16):** the plugin must not show the names TB-303, TR-808 and TR-909 to users. The host-visible device names become ORB-303, ORB-808 and ORB-909, and the CLAP description becomes just "OpenRB-338". The parameters' string ids (`g20`, `d<device>c<n>`; crates/orb-plugin/tests/param_ids.txt) don't change, so hosts keep their automation; only the labels change.

**Exact changes:**
- crates/orb-plugin/src/params.rs around lines 258-261, the device name strings:
  - "TB-303 1" becomes "ORB-303 1";
  - "TB-303 2" becomes "ORB-303 2";
  - "TR-808" becomes "ORB-808";
  - "TR-909" becomes "ORB-909".
- params.rs around line 346: the doc comment example "TB-303 1 Cutoff" becomes "ORB-303 1 Cutoff".
- crates/orb-plugin/src/lib.rs around line 228: `CLAP_DESCRIPTION` becomes `Some("OpenRB-338")`.
- Update any test that asserts the old names; search crates/orb-plugin/tests for them.

**Out of scope:** internal identifiers (`Device::Tb303_1`, `TB303`/`TR808`/`TR909` consts), other crates, docs, sample file names.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The host-visible parameter names use ORB-303 1, ORB-303 2, ORB-808 and ORB-909 (a test or the existing name test shows one of each)
- [x] #2 CLAP_DESCRIPTION is "OpenRB-338"
- [x] #3 No string id changes: crates/orb-plugin/tests/param_ids.txt is unchanged
- [x] #4 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. params.rs:258-261: device names TB-303 1/2 -> ORB-303 1/2, TR-808 -> ORB-808, TR-909 -> ORB-909.
2. params.rs:346 doc comment example TB-303 1 Cutoff -> ORB-303 1 Cutoff.
3. lib.rs:227-228 CLAP_DESCRIPTION -> Some("OpenRB-338").
4. Grep crates/orb-plugin/tests for old names (none found in the pre-edit grep; the sample names TR808BD.aif etc. are file names and stay).
5. cargo test -p orb-plugin, cargo clippy --all-targets -- -D warnings, cargo +nightly fmt --all --check; record counts.
6. Check ACs, final summary, Done, one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a0ffb9b4f37a67bc9 (rebased onto main 043341f).

Edits: params.rs device_name ORB-303 1/2, ORB-808, ORB-909; params.rs doc comment ORB-303 1 Cutoff; lib.rs CLAP_DESCRIPTION Some("OpenRB-338") (fmt put it on one line). No existing test asserted the old names (grep of crates/orb-plugin before and after: only the device_name strings and the doc comment); the sample file names TR808BD.aif and TR909bd1.aif in tests/common/mod.rs stay. Added test the_device_names_are_orb_names to tests/params.rs (AC #1 needed a test; none existed). Runs: cargo test -p orb-plugin: 56 passed, 0 failed (lib 5, editor 15, exports 2, fonts 2, no_alloc 4, note_mode 4, params 6, state 4, sync 9, timing 5; clap_gui ok, harness=false). cargo clippy --all-targets -- -D warnings: exit 0, no warnings. cargo +nightly fmt --all --check: exit 0 (first run failed on CLAP_DESCRIPTION line wrap, fixed). git diff of crates/orb-plugin/tests/param_ids.txt: empty; the_ids_do_not_change passes.

Review round 1 (send back, test count only): the branch was rebased onto main 2eb5f36 (task-217 merged), which added tests. Rerun of cargo test -p orb-plugin in the worktree at 68f3949: 66 passed, 0 failed (lib 5, editor 18, exports 2, fonts 2, no_alloc 4, note_mode 5, params 7, preview 4, state 4, sync 10, timing 5; clap_gui ok). The earlier 56 is stale. No code change; the commit is amended.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The host-visible plugin names are now ORB-303 1, ORB-303 2, ORB-808 and ORB-909 (crates/orb-plugin/src/params.rs device_name, and the doc comment example), and CLAP_DESCRIPTION is Some("OpenRB-338") (crates/orb-plugin/src/lib.rs). Parameter string ids are unchanged (tests/param_ids.txt untouched, the_ids_do_not_change passes). A new test the_device_names_are_orb_names in crates/orb-plugin/tests/params.rs shows one name per device. Branch worktree-agent-a0ffb9b4f37a67bc9, rebased onto main 2eb5f36. Tests: cargo test -p orb-plugin 66 passed, 0 failed; clippy -D warnings clean; nightly fmt --check clean.
<!-- SECTION:FINAL_SUMMARY:END -->
