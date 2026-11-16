---
id: TASK-220
title: 'Plugin: ORB-303/ORB-808/ORB-909 in parameter names, description "OpenRB-338"'
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-16T15:44:00Z'
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
- [ ] #1 The host-visible parameter names use ORB-303 1, ORB-303 2, ORB-808 and ORB-909 (a test or the existing name test shows one of each)
- [ ] #2 CLAP_DESCRIPTION is "OpenRB-338"
- [ ] #3 No string id changes: crates/orb-plugin/tests/param_ids.txt is unchanged
- [ ] #4 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
