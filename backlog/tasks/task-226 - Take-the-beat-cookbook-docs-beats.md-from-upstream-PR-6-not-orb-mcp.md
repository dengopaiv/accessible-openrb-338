---
id: TASK-226
title: 'Take the beat cookbook (docs/beats.md) from upstream PR #6, not orb-mcp'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-10T19:48:00Z'
updated_date: '2026-11-10T20:45:00Z'
labels:
  - docs
dependencies: []
references:
  - https://github.com/pingw33n/openrb-338/pull/6
priority: low
type: docs
ordinal: 244000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user decided on 2026-11-10 about upstream pull request #6 (github.com/pingw33n/openrb-338/pull/6 by gaolivaresm, open, head e50fc7e): "I think we don't need an mcp, but the cookbook would be nice." The PR adds crates/orb-mcp (an MCP server over orb-cli), docs/mcp.md, .mcp.json, scripts/local-env.sh, scripts/mcp-demo.sh and upstream task files, plus docs/beats.md ("Writing a song: the beat cookbook", 616 lines: drum column order, 303 step flags, shuffle, levels, tricks from the shipped songs and the manual, with sources) and examples/ (empuje.py, a generator, and empuje.rbs, the song it writes byte for byte, with examples/README.md).

What: take docs/beats.md and examples/ (README.md, empuje.py, empuje.rbs) from the PR's head into the fork, credited to their author in the commit (Co-authored-by) and in the document's header; nothing of orb-mcp. The cookbook uses tools/rbs.py build / json / check / dump, which the fork has. Remove or rewrite what needs the server: the section "From an MCP client" (line 250 at e50fc7e) and every mention of docs/mcp.md or the orb_* tools. Check that each command in the document runs on Windows as written or give the Windows form (python is `py -3` or `python`, no /tmp). Run examples/empuje.py and check that it writes examples/empuje.rbs byte for byte. Link the cookbook from docs where songs are discussed (e.g. AGENTS.md's list of knowledge documents or docs/RBS42.txt's neighbors) only if a natural place exists. The project is GPL-3.0-only; the PR is a contribution to it.

Later upstream merges: if upstream merges PR #6, the fork's next upstream merge brings orb-mcp (the user does not want it: remove it in that merge, as a fork removal decided here) and upstream task files task-176 and task-177, whose ids the fork already uses (fork task-176: the NFC test, task-177: Decide how screen reader mode turns on); renumber the incoming ones then. A docs/beats.md that differs from this copy is resolved toward upstream's text minus the MCP parts.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/beats.md and examples/ (README.md, empuje.py, empuje.rbs) are in the fork from PR #6's head e50fc7e, the author credited; no orb-mcp crate, docs/mcp.md, .mcp.json or the PR's scripts
- [x] #2 No part of docs/beats.md or examples/README.md refers to an MCP server or the orb_* tools
- [x] #3 Each command in the cookbook was run on Windows (or given its Windows form) and its result recorded in the notes; examples/empuje.py rewrites examples/empuje.rbs byte for byte
- [x] #4 AGENTS.md's fork section notes the decision about PR #6 (cookbook yes, orb-mcp no) and the task-176 / task-177 id clash for a later upstream merge
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Take the four files from e50fc7e (docs/beats.md, examples/README.md, empuje.py, empuje.rbs); remove the MCP section and mentions; header credit and Windows forms; run each command on Windows; add the AGENTS.md bullet and list entry; fix the TASK-225 commit column in docs/accessibility_plan.md.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
The user's reason for leaving orb-mcp out (2026-11-10): AI is welcome for building the music tools, not for making the music; nothing that lets an AI write songs (an MCP server or similar) belongs in the fork. The AGENTS.md note of this task should give that reason, so a later upstream merge drops such parts too. The cookbook stays because it is for a person writing songs.

Branch: worktree-agent-ac341cae3f32f184e

Author line of the PR commit: gaolivaresm <olivareg@gmail.com> (Co-authored-by in the commit). Removed from the author's text: the section 'From an MCP client' of docs/beats.md (lines 250-262 at e50fc7e) and, in examples/README.md, the paragraph 'Or through the MCP server ... orb_build and orb_song (docs/mcp.md)'. Added: a header paragraph in docs/beats.md (credit, Windows forms of python3, orb, /tmp, xdg-open, pgrep) and a Windows sentence in examples/README.md. Nothing else of the author's text changed; no sentence framed the document as instructions for an AI. Left as is: the worked example mentions ~/Musica/rebirth/hiphop-90bpm.rbs (the author's own file, not in the repo).

Commands run on Windows (py -3, scratchpad folder, orb built with CARGO_TARGET_DIR=C:\GIT\music\accessible-openrb-338\target, cargo build --release -p orb-cli, 40 s). The short-way beat.json: py -3 tools/rbs.py build -> ok; check -> 'ok'; orb.exe render beat.rbs -o beat.wav -> 470399 frames, 10.67 s, level script (py -3, audioop) RMS 311 peak 3428 (the doc's hiphop example has a different song, RMS 2653 there). render --range 0:4 -> 1881640 bytes wav; -o .flac 236984 bytes; -o .ogg 104238 bytes; orb play beat.rbs --seconds 1 -> played 1 s on the sound card, exit 0. tools/rbs.py dump and json of 'Silent Default Song.rbs' work (json piped to head gave a BrokenPipeError only because head closed the pipe). Not run: xdg-open (Windows form Start-Process, not run to keep the machine silent), pgrep (Windows form Get-Process orb), the last section's 'chords01.rbs' / 'Plucked 303-01.rbs' (the author's files outside the repository, not available), 'cd ~/Proyectos/openrb-338' (the author's path).

examples/empuje.py: py -3 examples/empuje.py empuje.json -> '64 bars at 118 BPM'; tools/rbs.py build -> empuje.rbs; sha256 dbb9270e8853a314e702f4eb754493d659abe6db02e65cca88353f2b160932e4 for both the generated file and examples/empuje.rbs: byte for byte equal. orb render of it: 5740474 frames, 130.17 s, RMS 1020 peak 13631. No test suites run (docs only).

Review round 1 (send back): (1) commit Result narrowed to what was run; (2) docs/beats.md header no longer claims the Start-Process and Get-Process forms were run; (3) final summary narrowed; (4) header now gives Windows forms for the heredocs (PowerShell @'...'@ | Set-Content, or an editor; a .py file run with py -3) and for cd. Run on Windows: the PowerShell here-string wrote beat2.json, tools/rbs.py build + check ok, beat2.rbs sha256 5F1DE8E8...788B equal to the beat.rbs built from the bash heredoc, orb.exe render + the .py level script gave 470399 frames, RMS 311, peak 3428 (same as before). Not run: Start-Process, Get-Process, the cd form (a plain cd to the checkout).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Took docs/beats.md and examples/ (README.md, empuje.py, empuje.rbs) from upstream PR #6 head e50fc7e, credited to gaolivaresm in the header and the commit; orb-mcp, docs/mcp.md, .mcp.json, the scripts and task files were not taken. Removed the MCP section and mentions; no MCP or orb_* tool is mentioned. Windows forms are given in a header paragraph of docs/beats.md. Run on Windows: py -3 and orb.exe forms (build, check, render, --range, flac, ogg, play, dump, json), the PowerShell here-string form for the JSON and the .py file form for the level check; empuje.py reproduces empuje.rbs byte for byte (same sha256). Given but not run: Start-Process, Get-Process, cd. Not run: the author's own chords01.rbs and Plucked 303-01.rbs commands (files not in the repository). AGENTS.md: a fork bullet on PR #6 and docs/beats.md in the knowledge list. docs/accessibility_plan.md: TASK-225 commit set to e909060. Docs only, no tests.
<!-- SECTION:FINAL_SUMMARY:END -->
