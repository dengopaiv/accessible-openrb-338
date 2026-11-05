---
id: TASK-203
title: 'Decide: a name the system dialog did not confirm'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-06T05:43:00Z'
updated_date: '2026-11-06T05:43:00Z'
labels:
  - decision
  - ui
milestone: m-7
dependencies: []
references:
  - crates/orb-ui/src/system_dialog.rs
  - crates/orb-ui/src/app.rs
priority: medium
type: spike
ordinal: 223000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Context (TASK-194): in screen reader mode Save As and Export use the system's file dialog. The dialog asks about replacing a file for the name the user typed (`a.txt`). The app then adds the extension the format needs (`a.txt.rbs`, or `.wav` for an export without one), and the file `a.txt.rbs` may exist already: the dialog never asked about that one. On Windows rfd sets the first filter's default extension, so a bare name is usually covered, but a name with a dot (`v1.2 mix`, `a.txt`) is not, and on Linux every extension-less name hits this. Without a rule the song or the export would replace the file without a question.

Options:
- A. Ask in OpenRB-338: when the path written differs from the path the dialog returned and that file exists, a Yes/No box "Replace <name>?" (No is the default and has the focus) asks before writing. Yes writes, No writes nothing.
- B. Refuse the name: an error box says the name needs the extension, and nothing is written.
- C. Reopen the dialog with the corrected name (`a.txt.rbs`) as its suggestion, so that the dialog asks itself.

Recommendation: A (no loop, the user keeps the name they chose, the question is the same one the browser asks).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen one of the options
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The user chose A, ask in OpenRB-338 (2026-11-05): a name the system dialog did not confirm, that is a file already, is asked about in a Yes/No box "Replace <name>?" before it is written; No is the default. Applied in TASK-194 (review round 1).
<!-- SECTION:FINAL_SUMMARY:END -->
