---
id: TASK-198
title: 'Decide: no web version in this fork'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:35:00Z'
updated_date: '2026-11-05T19:35:00Z'
labels:
  - decision
dependencies: []
references:
  - AGENTS.md
  - docs/accessibility.md
  - docs/accessibility_plan.md
priority: high
type: spike
ordinal: 199500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: upstream planned a web version, milestone m-6 (2026-11-03: the engine and the UI on wasm32, hosted on GitHub Pages; TASK-169 to TASK-175). In this fork the next milestone by the queue rule (AGENTS.md: the lowest open milestone first) was m-6, so an agent working the fork's queue would have started on the web version instead of the fork's purpose, screen reader accessibility (m-7). The web version would also not be accessible: eframe 0.37's web runner drops the AccessKit update (docs/accessibility.md section 3.9).

Options: A. keep m-6 open in the fork, after m-7. B. no web version in this fork: archive m-6 and its tasks, m-7 first.

The user's words (2026-11-05): "The program will never have a web version. We only need a local editor. Nothing too complex. If necessary, we'll gut the upstream to keep only what we need and what is local."

References: AGENTS.md (section "This fork"), docs/accessibility.md, docs/accessibility_plan.md, backlog milestone m-6.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen; the final summary records the choice and the date
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, @dengopaiv, 2026-11-05: B. This fork is a local desktop editor and will never have a web version.

- Milestone m-6 and its tasks TASK-169 to TASK-175 are archived in the fork (backlog milestone archive / task archive: kept with their ids, off the board; upstream keeps them).
- m-7 (screen reader accessibility) is the fork's first open milestone.
- The fork's guiding rule, in AGENTS.md "This fork": a local editor, nothing too complex; upstream parts the fork does not need, and anything not local, may be removed when necessary, each removal decided as its own task.
<!-- SECTION:FINAL_SUMMARY:END -->
