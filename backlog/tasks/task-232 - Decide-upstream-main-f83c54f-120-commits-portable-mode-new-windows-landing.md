---
id: TASK-232
title: >-
  Decide: upstream main f83c54f (120 commits): portable mode, new windows,
  landing
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-26T12:07:00Z'
updated_date: '2026-11-26T12:28:00Z'
labels:
  - decision
  - ui
  - tools
dependencies: []
references:
  - https://github.com/pingw33n/openrb-338/commit/f83c54f
priority: medium
type: spike
ordinal: 250000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on 2026-11-26 upstream's main (github.com/pingw33n/openrb-338) was at f83c54f, 120 commits after 6858253, the base of the fork's last upstream merge (task-230). The user asked to bring the fork up to date with it ("Synchronize the repository with the upstream one. Ask questions where applicable. Do not run any tests yet."). Two parts of it clash with the fork's work, and the tests could not run in that session, so the user decided three questions before the merge.

What upstream added since 6858253, in short: the skin layer (task-266: one controller, controller.rs, takes all the panel's input; the modern panel is the first skin, skin/svg.rs) and the classic skin from the kit's bitmaps with View > Classic / Modern and a crossfade (task-267, task-268); the 303, 808 and 909 pattern editors as floating windows (task-274, task-280, task-281, task-290; Ctrl/Cmd+E opens the focused row's); Copy Pattern as JSON on the system clipboard (task-287); Edit > Clear Pattern and Edit > Initialize Song / Loop from Pattern Mode (task-290, task-306); Options > Audio Output with driver, device, rate and buffer, and ASIO on Windows x86_64 with the ASIO SDK in third_party/asiosdk (task-191, task-193); the settings file renamed ui.toml -> OpenRB-338.toml and a Windows portable mode through an OpenRB-338.portable marker (task-289, task-295); Ctrl/Cmd+I for File > Info (task-310); a new song with four sample patterns (task-288); the original's log-scale meters and gain-reduction bar (task-252, task-253); the About window's copy-version button (task-279); the Keyboard Shortcuts help cut by upstream's user (task-284) and no TR-/TB- names in visible text (task-286); the minimum window 536 x 485 (task-299); regenerated third-party manifests, 324 mods and 4834 songs (task-311); plugin work (CLAP dropped, VST3 only, per-device and per-drum outputs, Output Routing, the egui-baseview fork, REAPER tests); AGENTS.md and the agent brief no longer require Edit/Write for edits (ace232b); 69 new backlog files task-250..task-316 and milestones m-8 and m-9; landing page (site/) edits.

Already settled by task-229 and applied without a question: upstream's backlog files from task-176 on and its milestones from m-7 on stay out of the fork (here task-250..task-316, the archived task-282 and task-190's move, m-8, m-9 and upstream's edits of the task-186, 187, 191, 193, 215 and 244 files that the fork had dropped); the landing page stays out (site/, tools/build_site*).

The three questions and the options put to the user:
1. Portable mode, two designs: upstream's (marker OpenRB-338.portable, OpenRB-338.toml beside the exe, logs\, full paths) and the fork's task-223 (marker portable.txt, data\ui.toml and data\logs, folders inside the program's folder stored relative to it). A. combine: upstream's marker, file and layout, plus the fork's relative folders and a one-time move of an existing portable.txt / data\ui.toml copy; B. keep the fork's, upstream's switched off; C. upstream's as it is.
2. The new windows built for the mouse (classic skin, pattern editors, Audio Output) and the skin refactor that moved the panel's input out of app.rs, where the fork's screen reader code hooked in: A. take all, port the fork's hooks into the new controller so the panel works as before in both skins, and give each new window its own m-7 task; B. take all but hide the classic skin and the editors in screen reader mode until their tasks are done; C. leave the editors out.
3. Where the merge lands while no tests run: A. a branch only, as a merge commit, checked with cargo check; B. the local main, not pushed; C. main, pushed.

Recommendations given: 1A, 2A, 3A.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen for each of the three questions; the outcome and date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, 2026-11-26 (asked in the session that does the merge, task-233):
1. Portable mode: A, combine. The program uses upstream's marker OpenRB-338.portable, OpenRB-338.toml beside the exe and logs\ there, so later merges stop conflicting in portable.rs and settings.rs; the fork's relative folders stay (a USB stick under any drive letter); a folder with the fork's older portable.txt stays portable, and its data\ui.toml is copied into OpenRB-338.toml on the first start (the old files are left as they are).
2. The new windows: A, take everything. The fork's screen reader hooks move into upstream's controller.rs so the main panel works as before in both skins; the pattern editors, the Audio Output window and the Initialize Song question come in as upstream built them, each with its own m-7 task to make it accessible.
3. Landing: A, a branch only (sync-upstream-f83c54f, a merge commit with f83c54f as its second parent), checked with cargo check; main and the fork on GitHub stay as they are until the user says to run the tests and the review.
4. Asked later in the same session (upstream's task-286 removed the TR-/TB- names from visible text; the fork's spoken names still had them): follow upstream, the fork's screen reader names say 808, 909 and 303 too (options: follow upstream; keep the long names). Done by task-239.
<!-- SECTION:FINAL_SUMMARY:END -->
