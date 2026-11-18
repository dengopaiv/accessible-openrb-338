---
id: TASK-229
title: 'Decide: upstream main 6858253 (107 commits): plugin, landing page, task ids'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-18T19:10:00Z'
updated_date: '2026-11-18T19:10:00Z'
labels:
  - decision
  - ui
  - tools
dependencies: []
references:
  - https://github.com/pingw33n/openrb-338/commit/6858253
priority: medium
type: spike
ordinal: 247000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on 2026-11-18 upstream's main (github.com/pingw33n/openrb-338) was at 6858253, 107 commits after 32a642d, the base of the fork's last upstream merge (task-224). The user asked to bring the fork up to date with it ("Freshen up our fork with the material from the main. Pose questions where necessary."). Three parts of it clash with the fork's rules, so the user decided them before the merge.

What upstream added since 32a642d, in short: a VST3/CLAP instrument plugin (crates/orb-plugin on nice-plug, xtask as its bundler, vendor/raw-window-metal for macOS, plugin note, pattern and gate modes in orb-player and orb-engine, orb-ui made embeddable with Help > Plugin Usage, packaging of the bundles in every package, validators in CI); standalone MIDI work (MIDI Standard Mapping layer, task-182..195, mostly still To Do upstream); the panel redesign (task-179) and the double click that sets a knob to its default (#11); pattern copy, paste and duplicate (PR 16); a ui.toml save that merges instead of overwriting; DX12 first on Windows (upstream task-177, the fork's task-217); an optimized test profile; Linux, macOS and installer packaging changes; the ASIO decision (task-192, ASIO itself not built yet); a GitHub Pages landing page (site/, .github/workflows/pages.yml, tools/build_site.py, tools/site_textures) with GoatCounter visitor counting; 74 new backlog tasks task-176..task-249 and a milestone m-7 "MIDI input and the VST3/CLAP plugin".

The three questions and the options put to the user:
1. The plugin: A. take it and ship OpenRB-338.vst3/.clap in the portable zip; B. take the code but leave it out of the zip until an NVDA check; C. leave the plugin out of the fork, removed by its own task.
2. The landing page: A. remove site/, the pages workflow and the site builder tools in the merge; B. keep the files, drop only the workflow; C. keep everything (the workflow would fail on each fork release, Pages being off).
3. Upstream's backlog ids task-176..249 and m-7 collide with the fork's own task-176..228 and m-7 (different file names, so git does not see it): A. leave upstream's new task and milestone files out of the fork, readable on upstream's GitHub, and say in AGENTS.md that upstream's task numbers from 176 up mean upstream's tasks; B. renumber them into the fork (task-229 on, m-8) with a mapping document.

Recommendations given: 1A, 2A, 3A.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The user has chosen for each of the three questions; the outcome and date are in the final summary
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Decided by the user, 2026-11-18 (asked in the session that does the merge):
1. The plugin: C, leave it out. The merge takes upstream's code as it is (so the merge base stays right); a separate removal task (task-231) then removes the orb-plugin crate and its packaging from the fork.
2. The landing page: A, removed in the merge (task-230): site/, .github/workflows/pages.yml and the tools that only build the site. Reason: the fork's rule that nothing that goes online is built (AGENTS.md "This fork"; GoatCounter counts visitors through an online service).
3. Upstream's backlog: A, left out. The merge does not bring upstream's new task files task-176..task-249 or its milestone m-7 into the fork's backlog; AGENTS.md says that task numbers from 176 up in upstream's docs, code comments and commit messages refer to upstream's own backlog. Upstream's edits to tasks both sides share (below 176) come in as usual.
<!-- SECTION:FINAL_SUMMARY:END -->
