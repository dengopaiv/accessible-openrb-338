---
id: TASK-239
title: Screen reader names say 808, 909 and 303, not TR-/TB-
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-26T12:28:00Z'
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
- [ ] #1 git grep for TR-808, TR-909 and TB-303 finds no user-visible or spoken string in crates/orb-ui (comments and docs quoting the original excepted)
- [ ] #2 The Keyboard Shortcuts help passes upstream's guard (no TR-/TB- text) with screen reader mode on as well as off (the_users_cuts_and_renames_hold covers groups(is_mac, true))
- [ ] #3 The a11y tests pass with the new names; docs/accessibility.md and docs/ui_spec.md section 20 say the same names
<!-- AC:END -->
