---
id: TASK-306
title: 'Edit: Initialize Song / Loop from Pattern Mode, as in the original'
status: To Do
assignee:
  - '@agent-complex'
created_date: '2026-11-25T13:06:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies: []
references:
  - docs/ui_spec.md
  - docs/song_load.md
  - docs/RBS42.txt
  - crates/orb-ui/src/menu.rs
priority: medium
type: feature
ordinal: 277700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-25 for the original's Edit menu commands that initialize the song, or the loop, from Pattern Mode, behaving exactly as in ReBirth 2.01. docs/ui_spec.md section 12 lists only the menu items in scope so far and says what the original's other Edit items do "was not read".

**Part 1, reverse engineering** (Rebirth.exe and ReBirth.dll; the user's IDA databases, opened read only through idalib):
- Find the menu items and their exact names, command ids, enable conditions and handlers. Start from the menu resource and strings, and the WM_COMMAND dispatch that handles the ids in section 12.
- Read what each one writes:
  - which song tracks or events (pattern selection, mutes, knobs and other automated controls, tempo, shuffle);
  - at which position (the song start, the loop start, the current position);
  - what is overwritten or cleared;
  - how the loop start and length change;
  - whether the song gets modified and whether there is a confirmation dialog;
  - how the engine messages go (msg 35 and the like) and any undo.
- Check the facts against the disassembly, not only Hex-Rays, and give addresses.
- Write them up in the document they belong to (docs/ui_spec.md section 12 or 18, and docs/song_load.md or docs/RBS42.txt for the song events), marking each [C] or [G].
- Where the code leaves it unclear, confirm on the Windows test machine with ReBirth (doc-001: rcs_client.py and the permits). Save a song before and after the command and compare the files.
- A defect of the original goes into docs/original_bugs.md too.

**Part 2, implementation in orb:**
- The same items in the standalone app's Edit menu, with the original's names unless the user says otherwise. Their effect on the song is the original's, byte for byte in a saved .rbs, and so is their effect on playback.
- Check whether the plugin's editor should show them. The plugin's song mode and Edit menu are in docs/plugin.md and ui_spec section 20. If it is unclear, ask through the supervisor; it is a decision for the user.
- Tests: compare orb's result on a song with the original's saved file for the same song and command. Store the files as committed test inputs if they are small and not copyrighted (songs made on the test machine from the template), otherwise say how they are produced.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The original's commands are documented with names, ids, enable conditions, handlers and exactly what they write to the song (tracks, position, loop, modified flag, dialog), with addresses, [C]/[G], in docs/
- [ ] #2 orb's Edit menu has the commands; on test songs orb's saved .rbs after each command equals the original's saved file for the same song and command (or the differences are documented as the original's bugs or orb-only fields)
- [ ] #3 Playback after the command matches the original (song mode plays what pattern mode had)
- [ ] #4 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [ ] #5 The user has checked the commands in the app
<!-- AC:END -->
