---
id: TASK-246
title: 'Accessibility docs: what is done, what is to do, what the program does'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-28T05:53:00Z'
updated_date: '2026-11-28T06:13:00Z'
labels:
  - docs
  - ui
milestone: m-7
dependencies: []
references:
  - docs/accessibility_plan.md
  - docs/accessibility.md
  - README.md
priority: medium
type: docs
ordinal: 263000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user asked on 2026-11-28 (a /goal): rewrite the accessibility plan to distinguish clearly between what has been done and what is yet to do, and the accessibility documentation itself marking what the program does or will do. Both documents stopped at 2026-11-09: they miss the upstream merges (task-230, task-233), the screen reader tasks for upstream's new windows and the classic look (task-234 to task-239, task-245), the plugin removal (task-231, task-241), and the open NVDA pass TASK-244.

The user's answers (2026-11-28):
1. NVDA checklist items 23 to 28 (TASK-212, 211, 201, 204, 213, 225) are not checked by ear yet: they stay under to do.
2. docs/accessibility.md is rewritten as a description of what the program does, each part marked (does, heard with NVDA / does, tests only / will do / not built), with the measurements from before the work and the options weighed in a short history section at the end; the developer reference stays.
3. The design choices that task-235, task-236 and task-238 made without asking are listed as open: built this way, waiting for the user's confirmation (the TASK-244 pass).
4. The README's screen reader section is brought up to date too (status, known limits, links, the new windows).

Found while checking the proposal against the code: Ctrl+Up / Ctrl+Down move the row focus silently and entering a machine's section does not say keyboard focus (the proposal's 3.2 and 3.5 said it would); braille output was never checked; docs/ui_spec.md 20.3 still gives F4's gain reduction as N of 4 (it is N of 9 since task-233).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/accessibility_plan.md has a Done part and a To do part that do not mix: every m-7 task, decision and by-ear check is in exactly one of them, as the backlog has it on 2026-11-28
- [x] #2 The To do part lists the by-ear checks waiting for the user (checklist items 23 to 28 and TASK-244's seven), the design choices waiting for the user's confirmation, the open tasks (TASK-227, TASK-199 and TASK-206 on hold) and the known gaps with no task
- [x] #3 docs/accessibility.md describes what the program does, each part marked by its state, checked against the code; the history (before the work, the options) is a section of its own at the end
- [x] #4 README.md's screen reader section matches the two documents (status, the new windows, known limits, links)
- [x] #5 Reviewed by agent-reviewer with a merge verdict
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: docs-a11y-246 (in the main checkout; docs only, no build).

Facts checked against the code before writing:
- The row focus is not announced: a11y/announce.rs key_effect has no row focus case; app.rs follow_section only sets panel.focus; Ctrl+Up / Ctrl+Down are Action::Focus (keys.rs:278). The proposal's 3.2 ("303-2, keyboard focus") and 3.5 ("the row focus moved") were never built: marked Not built, listed in the plan's 2.4.
- F4 says gain reduction N of 9 (announce.rs levels_readout, model::gr_lit, since task-233); docs/ui_spec.md 20.3 and announce.rs's doc comment still said N of 4: both corrected.
- View > Classic / Modern are egui radio items (menu.rs ui.radio), so they carry their state.
- The Replace question focuses No on its first frame (system_dialog.rs replace_ui).
- The headless counts of a11y_baseline.rs: mode off 205 nodes, 198 named, 196 focusable; mode on 600, 593, 200.
- No braille handling anywhere in orb-ui (grep): braille marked Not checked.

Kept stable: the section numbers of docs/accessibility.md that the code and tests cite (1.1, 3.2, 3.3, 3.4, 3.5, 6.2, 6.3; git grep of accessibility.md in crates, tools, docs, AGENTS.md), so no comment had to change for the renumbering.

Also changed: AGENTS.md's description of the two documents (and part 3 -> part 4 for the ground rules); the plan's ground rules now ask each screen reader task to update the plan and the state marks of docs/accessibility.md.

Checks: cargo +nightly fmt --all --check clean (announce.rs comment); no new line over 100 columns outside tables in the three docs.

Review round 1 (agent-reviewer): send back. Fixed in the amend:
1-2. Plan 2.2 now has task-236's six choices as its notes list them (the Tab order added; the cell and Entry words as one), so 12 choices in all; choice 7 says that upstream's mode-off window has AC at the bottom and the mode's picture follows the grid (task-236 review round 1).
3. The claim about the reviewers' agreement is gone (not in the backlog); choice 5 says task-235's notes leave the pinned pattern to the user.
4. accessibility.md 3.10: VoiceOver and Orca are TASK-199 only; TASK-206 named for what it is.
5. TASK-246 is set Done in this commit, so its Part 1 paragraph and 'four are open' hold on main after the merge.
6. The decisions table marks TASK-198, TASK-229 and TASK-232 as outside m-7, and the TASK-195 row as a work task's own choices.
7. 'no open task' for items 23 to 28.
8. accessibility.md's legend adds Not tried / Not checked; NVDA on Windows is 'Does, heard (1 to 22)'.
9. The cited section list includes 3.6 (system_dialog.rs).
10. mod_chooser.rs named by its path, not as part of a11y/.
11. TASK-184's row says it was heard through items 13 to 17, which reach their windows by the menus.
12. Not changed: README.md's 'Not implemented' list is upstream's section (outside the fork's), kept as upstream has it for simpler merges.

Review round 2 (agent-reviewer): merge.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
docs/accessibility_plan.md is rewritten in four parts: Done (the decisions, and the work in six rounds, each task with its commit and whether its by-ear check is heard), To do (13 checks by ear: checklist items 23 to 28 and TASK-244's seven criteria as items 29 to 35; 12 design choices of task-235, task-236 and task-238 waiting for the user; the open tasks TASK-244, TASK-227, and TASK-199 and TASK-206 on hold; the known gaps with no task), the NVDA checklist, and the reference. docs/accessibility.md now describes what screen reader mode does, each part marked Does, heard (N) / Does, not yet heard (N) / Will do (TASK) / Not built / Not tried, with a summary table, the current measurements, the developer reference and a history section; the section numbers the code cites stay. README.md's screen reader section, AGENTS.md's description of the documents, docs/ui_spec.md 20.3 and announce.rs's comment (F4's gain reduction of 9) are brought up to date. Found and listed as gaps: the row focus is never announced; braille never checked. Branch docs-a11y-246, one commit. Checks: nightly fmt --check clean; docs only otherwise. Review: round 1 send back (12 points, 11 fixed, one left as upstream's text).
<!-- SECTION:FINAL_SUMMARY:END -->
