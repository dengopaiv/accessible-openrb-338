---
id: TASK-316
title: 'Plugin website link: the project site'
status: Done
assignee:
  - '@agent-chore'
created_date: '2026-11-26T08:57:00Z'
updated_date: '2026-11-26T09:04:00Z'
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
- [x] #1 orb_plugin::URL is https://pingw33n.github.io/OpenRB-338/ and the exports test asserts it
- [x] #2 both links point to the project site: orb_plugin::URL and orb_ui::about::URL are https://pingw33n.github.io/OpenRB-338/, and the docs that describe them (docs/plugin.md section 2, docs/ui_spec.md Help > About) name it
- [x] #3 cargo test -p orb-plugin, cargo test -p orb-ui, clippy -D warnings and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Set pub const URL in crates/orb-plugin/src/lib.rs:51 to https://pingw33n.github.io/OpenRB-338/.
2. Update the assertion in crates/orb-plugin/tests/exports.rs:463 to the same URL.
3. Update the URL row of the identity table in docs/plugin.md:89 (the only plugin doc naming the link; docs/ui_spec.md:1399 and crates/orb-ui/src/about.rs are the About box, left alone).
4. Run cargo test -p orb-plugin, cargo clippy --all-targets -- -D warnings and cargo +nightly fmt --all --check in the worktree.
5. Record results, check ACs, write final summary, set Done, one commit with subject 'ui: plugin website link points to the project site'.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a3d40560251990972 (worktree /Users/pingw33n/dev/OpenRB-338/.claude/worktrees/agent-a3d40560251990972), rebased onto a979681.

Grep of the old URL (https://github.com/pingw33n/OpenRB-338) in crates, docs, README, scripts, xtask, tests: hits in crates/orb-plugin/src/lib.rs:51, crates/orb-plugin/tests/exports.rs:463, docs/plugin.md:89 (plugin identity table); the rest is orb-ui (crates/orb-ui/src/about.rs:38, crates/orb-ui/tests/menu.rs:2483, docs/ui_spec.md:1399, left alone) or the releases/rb-songs/rb-mods links in README.md and docs/building.md (not the plugin link).

Scope change (coordinator, 2026-11-26): the standalone About link changes too. AC 2 (About unchanged) is replaced by a both-links AC; cargo test -p orb-ui added to AC 3; commit subject is now 'ui: plugin and About links point to the project site'. Changed: crates/orb-ui/src/about.rs:38 URL, crates/orb-ui/tests/menu.rs:2483 (the only orb-ui test asserting the old URL), docs/ui_spec.md:1399 (Help > About line).

Results (worktree worktree-agent-a3d40560251990972, after the scope change): cargo test -p orb-plugin -p orb-ui: 58 test result lines, all ok, 0 failed (the orb-ui menu test asserts about::URL). cargo clippy --all-targets -- -D warnings: exit 0, no warning or error. cargo +nightly fmt --all --check: exit 0. Before the scope change, cargo test -p orb-plugin alone: 26 + 7 + 43 passed, 0 failed. git diff main --stat (mandatory check): only the task file and the seven files changed here (crates/orb-plugin/src/lib.rs, crates/orb-plugin/tests/exports.rs, crates/orb-ui/src/about.rs, crates/orb-ui/tests/menu.rs, docs/plugin.md, docs/ui_spec.md, the task file). No engine change, so no corpus run. Not run: the Windows or macOS VST3 host; the URL is a constant in the factory info and the exports test checks it through the VST3 C API.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The plugin's vendor URL and the standalone's About link now point to the project site, https://pingw33n.github.io/OpenRB-338/ (trailing slash). Changed: orb_plugin::URL (crates/orb-plugin/src/lib.rs:51) and the exports assertion (crates/orb-plugin/tests/exports.rs:463); orb_ui::about::URL (crates/orb-ui/src/about.rs:38) and its menu test (crates/orb-ui/tests/menu.rs:2483), per the coordinator's scope change of 2026-11-26; docs/plugin.md section 2 identity table and docs/ui_spec.md Help > About line. No saved state or parameter ids change. Checks: cargo test -p orb-plugin -p orb-ui all ok, clippy -D warnings exit 0, nightly fmt --check exit 0; the mandatory git diff main check lists only the task's files. Branch worktree-agent-a3d40560251990972, one commit on top of main (a979681). Not done: the docs/plugin.md identity table sits under the heading 'Permanent after the first release'; the URL is a vendor string the host reads, not saved state, so it was updated in place.
<!-- SECTION:FINAL_SUMMARY:END -->
