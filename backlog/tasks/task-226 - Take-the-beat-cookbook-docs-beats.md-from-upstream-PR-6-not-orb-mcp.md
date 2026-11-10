---
id: TASK-226
title: 'Take the beat cookbook (docs/beats.md) from upstream PR #6, not orb-mcp'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-10T19:48:00Z'
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
- [ ] #1 docs/beats.md and examples/ (README.md, empuje.py, empuje.rbs) are in the fork from PR #6's head e50fc7e, the author credited; no orb-mcp crate, docs/mcp.md, .mcp.json or the PR's scripts
- [ ] #2 No part of docs/beats.md or examples/README.md refers to an MCP server or the orb_* tools
- [ ] #3 Each command in the cookbook was run on Windows (or given its Windows form) and its result recorded in the notes; examples/empuje.py rewrites examples/empuje.rbs byte for byte
- [ ] #4 AGENTS.md's fork section notes the decision about PR #6 (cookbook yes, orb-mcp no) and the task-176 / task-177 id clash for a later upstream merge
<!-- AC:END -->
