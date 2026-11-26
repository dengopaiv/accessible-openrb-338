---
id: TASK-316
title: 'Plugin website link: the project site'
status: To Do
assignee:
  - '@agent-chore'
created_date: '2026-11-26T08:57:00Z'
labels:
  - ui
milestone: m-5
dependencies: []
references:
  - crates/orb-plugin/src/lib.rs
  - crates/orb-plugin/tests/exports.rs
priority: medium
type: enhancement
ordinal: 287700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked (2026-11-26) that the plugin's website link point to the project site, https://pingw33n.github.io/OpenRB-338/ (with the trailing slash), instead of the GitHub repository.

**Where:** crates/orb-plugin/src/lib.rs:51 `pub const URL` ("The project's page."), which the plugin reports to hosts as its vendor URL (lib.rs:161, the VST3 factory info). The test crates/orb-plugin/tests/exports.rs:463 asserts the old URL; update it. Grep docs/plugin.md and the rest of crates/orb-plugin for the old URL where it describes this link and update those places too. The standalone app's About link (crates/orb-ui/src/about.rs:38) is NOT part of this task; leave it.

This does not affect saved plugin state or parameter ids.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 orb_plugin::URL is https://pingw33n.github.io/OpenRB-338/ and the exports test asserts it
- [ ] #2 docs that describe the plugin's website link name the new URL; orb-ui's About link is unchanged
- [ ] #3 cargo test -p orb-plugin, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->
