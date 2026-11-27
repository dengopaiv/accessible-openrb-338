---
id: TASK-239
title: Screen reader names say 808, 909 and 303, not TR-/TB-
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:28:00Z'
updated_date: '2026-11-27T22:20:00Z'
labels:
  - ui
milestone: m-7
dependencies:
  - TASK-233
references:
  - crates/orb-ui/src/a11y/names.rs
  - crates/orb-ui/src/a11y/nav.rs
  - crates/orb-ui/src/key_help.rs
priority: low
type: enhancement
ordinal: 252000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream's task-286 (commit 3fba188, merged into the fork by task-233) removed the names TB-303, TR-808 and TR-909 from all user-visible text (status line, menus, the pattern editors, the Keyboard Shortcuts help, the CLI's drum names): the app says 303, 808 and 909 (and 303-1 / 303-2, upstream's task-293). The fork's screen reader text still has the long forms in places: the section names and control names spoken by NVDA (crates/orb-ui/src/a11y/names.rs, nav.rs SECTIONS, announce.rs), the screen reader group of the Keyboard Shortcuts help (key_help.rs screen_reader_group: 'TR-808, TR-909', 'on the TR-909 with Ctrl'), and their docs (docs/accessibility.md, docs/accessibility_plan.md, docs/ui_spec.md section 20). The user decided on 2026-11-26, asked in the session of task-233: the fork's spoken names follow upstream.

What to do: replace TR-808, TR-909, TB-303 (and 'Bass Line 1/2' only where upstream says 303-1 / 303-2 for the same thing; keep 'Bass Line' where it is the panel's own printed section name) in every string a screen reader speaks or the help shows, the tests that pin them (a11y_baseline's names, the names table tests) and the docs that quote them. Find them with git grep -n 'TR-[0-9]\|TB-[0-9]' -- crates/orb-ui docs. Comments that cite the original's names may stay.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 git grep for TR-808, TR-909 and TB-303 finds no user-visible or spoken string in crates/orb-ui (comments and docs quoting the original excepted)
- [x] #2 The Keyboard Shortcuts help passes upstream's guard (no TR-/TB- text) with screen reader mode on as well as off (the_users_cuts_and_renames_hold covers groups(is_mac, true))
- [x] #3 The a11y tests pass with the new names; docs/accessibility.md and docs/ui_spec.md section 20 say the same names
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Replace TR-808/TR-909 with 808/909 and Bass Line 1/2 with 303-1/303-2 (upstream's menu names, task-293) in a11y names.rs, nav.rs SECTIONS, announce.rs, key_help.rs screen reader group, the a11y tests, and docs/accessibility*.md, docs/ui_spec.md section 20. Keep comments quoting the original and upstream's own text. Run cargo test, clippy, fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a2fc3965f552a24c9

Replaced TR-808/TR-909 with 808/909 and Bass Line 1/2 with 303-1/303-2 (upstream menu names, task-293) in a11y/names.rs, nav.rs, announce.rs, key_help.rs, tests a11y_announce/grid/step303/sections, docs/accessibility.md, accessibility_plan.md, ui_spec.md section 20 (lines quoting the original's code or panel left). Left: comments citing the original, upstream's own text (ui_spec editor sections, pattern_clipboard test's TR-808 device). Evidence: cargo test --release --workspace 1158 passed 0 failed; clippy clean; nightly fmt clean (one rewrap applied). Two concurrent test runs happened by mistake; one stopped. By-ear check added to TASK-244 as an AC.

Review round 1: README.md (lines 88-89, 126, 138, 141) still had the long names; changed to 303-1, 303-2, 808, 909. Grepped all top-level .md files: only README.md had them (CHANGELOG quotes the old names on purpose). Commit body now says the commit adds one criterion to TASK-244. Text only, no cargo run.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Spoken names follow upstream: 808, 909, 303-1, 303-2 in the a11y names, F6 sections, F2 readouts and the Keyboard Shortcuts help (screen reader group), with tests and docs updated. 1158 tests pass, clippy and fmt clean. Branch worktree-agent-a2fc3965f552a24c9. By-ear check is an AC of TASK-244.
<!-- SECTION:FINAL_SUMMARY:END -->
