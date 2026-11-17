---
id: TASK-239
title: >-
  Docs: VST logo and trademark line only in Plugin Usage, not in the README or
  packages
status: Done
assignee:
  - '@agent-chore'
created_date: '2026-11-17T17:13:00Z'
updated_date: '2026-11-17T17:30:00Z'
labels:
  - docs
  - tools
milestone: m-7
dependencies:
  - TASK-237
references:
  - README.md
  - docs/plugin_usage.md
  - scripts/package.sh
  - scripts/package-dmg.sh
  - scripts/package-pkg.sh
  - scripts/changelog.sh
  - task-233
priority: medium
type: docs
ordinal: 400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The user's decision (2026-11-17), revising task-233: the Steinberg VST Compatible logo and the sentence "VST is a registered trademark of Steinberg Media Technologies GmbH." stay only in docs/plugin_usage.md (its VST section between the vst3-only markers, which Help > Plugin Usage shows in the VST3 build; crates/orb-ui/src/plugin_usage.rs). Remove them everywhere else that users read:
- README.md: the logo <picture> (around lines 66-71), the trademark sentence after it (line 73), the License-section sentences (lines 169-170), and the registered sign after VST (lines 51, 61: "VST 3" without the sign).
- The package texts: scripts/package.sh (the .zip/.tar.gz README.txt), scripts/package-dmg.sh (the DMG README.txt), scripts/package-pkg.sh (the pkg welcome page), scripts/installer/ and scripts/package-installer.sh (the Windows installer readme) if they carry it, and scripts/changelog.sh (the release notes): drop the sentence and the registered sign.
Keep: assets/steinberg/*.svg (plugin_usage.md uses them) and assets/README.md's source/terms note for them; docs/plugin_usage.md unchanged. Update developer docs that state where the attribution must appear (docs/building.md, docs/plugin.md, assets/README.md) to say it is in Plugin Usage only. Don't touch tests or Rust code unless a test checks the README/package text (grep crates/ for the sentence; the hits in crates/orb-plugin/tests and orb-ui-art are about the panel/test text, check whether they assert on the removed places).
Base on main after task-237 merges (it edits scripts/package.sh, the installer and docs/building.md).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 grep -rn -E 'Steinberg|VST®|registered trademark' README.md scripts/ finds nothing; docs/plugin_usage.md still has the logo and the sentence.
- [x] #2 Developer docs (docs/building.md, docs/plugin.md, assets/README.md) say the attribution is in Plugin Usage only.
- [x] #3 cargo test passes; bash -n on changed scripts passes; scripts/changelog.sh runs.
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. README.md: drop the registered sign after VST 3 (lines 51, 61); remove the logo <p> block and the trademark sentence after it (lines 66-73); remove the two License sentences about the trademark and the logo (lines 169-170).
2. scripts/package.sh: heading becomes 'Plugin (VST 3 and CLAP instrument)' with its underline resized; drop the trademark line (67-69).
3. scripts/package-pkg.sh: welcome page drops the sentence and the registered sign (80-81).
4. scripts/package-dmg.sh: the plugin paragraph drops the sentence and the registered sign (65-66), reflowed.
5. scripts/changelog.sh: drop the comment paragraph about the trademark sentence (18-19), and the printed sentence and registered sign (118), keeping 'The plugin is a VST 3 and CLAP instrument.'
6. Developer docs: docs/plugin.md 892-896 and assets/README.md 66-67 and 71-80 say the attribution is in Help > Plugin Usage only (assets/README.md: the README no longer shows the logo). Developer-doc trademark sentences that do not state a location (docs/plugin.md:3, docs/building.md:901, assets/README.md:69) stay.
7. Untouched: docs/plugin_usage.md, assets/steinberg/*.svg, crates/ (no test checks the README or package text). The stale comment at crates/orb-ui/src/plugin_usage.rs:510 is fixed in the same commit (coordinator follow-up).
8. Verify: grep -rn -E 'Steinberg|VST®|registered trademark' README.md scripts/ (empty); bash -n on the four changed scripts; scripts/changelog.sh HEAD once; cargo test in the worktree; cargo test -p orb-ui and cargo +nightly fmt --all --check after the comment edit.
9. Commit subject docs: (the current AGENTS.md rule: a commit that changes only documentation, including text in scripts, is docs:). One commit, Change/Result/Testing/Task body.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-acf4ed6561fc81b68 (worktree /Users/pingw33n/dev/OpenRB-338/.claude/worktrees/agent-acf4ed6561fc81b68), based on main ad4d5f5 (task-237 merged; no rebase needed).

Edits made per the plan: README.md (VST 3 without the sign at lines 51, 61; logo block, trademark line and License sentences removed); scripts/package.sh (heading 'Plugin (VST 3 and CLAP instrument)', underline 34 dashes to match, sentence removed); scripts/package-pkg.sh (welcome page); scripts/package-dmg.sh (paragraph reflowed); scripts/changelog.sh (comment paragraph and the printed sentence now 'The plugin is a VST 3 and CLAP instrument.'). Developer docs: docs/building.md (new sentence after the trademark line: attribution in Help > Plugin Usage only), docs/plugin.md 892-896 (Help > Plugin Usage only), assets/README.md (table row VST3, and the logo paragraph now names Plugin Usage as the only place). Kept: docs/plugin_usage.md, assets/steinberg/*.svg, docs/plugin.md:3, docs/building.md:901 and assets/README.md:69 (developer-doc trademark lines that state no location). Not touched: crates/orb-ui/src/plugin_usage.rs:510 comment says 'as the README does' and is now stale; Rust is out of scope, a follow-up for a later Rust task. Evidence: grep -rn -E 'Steinberg|VST®|registered trademark' README.md scripts/ -> no output (exit 1). grep on docs/plugin_usage.md -> lines 111, 112, 115 still hold the logo and the sentence. bash -n OK on package.sh, package-pkg.sh, package-dmg.sh, changelog.sh. Underline length 34 = heading length 34. scripts/changelog.sh HEAD -> exit 0, first line 'The plugin is a VST 3 and CLAP instrument.'. cargo test started in the worktree (output in the scratchpad, cargo-test-239.log).

cargo test (whole workspace, run in the worktree, log in the scratchpad cargo-test-239.log): exit 0, 93 test binaries, 895 passed, 0 failed, 5 ignored. AC 2 evidence: grep -n 'Plugin Usage' over docs/building.md, docs/plugin.md, assets/README.md shows building.md:902 'in Help > Plugin Usage only', plugin.md:895 'go to Help > Plugin Usage only', assets/README.md:67 and :81 name Plugin Usage as the only place. Follow-up noted, not done: crates/orb-ui/src/plugin_usage.rs:510 comment 'as the README does' is stale (Rust out of scope).

Coordinator follow-up (same commit, amended): crates/orb-ui/src/plugin_usage.rs:510 comment changed from 'as the README does' to 'as the page's picture does' (the README no longer shows the logo). Checks: cargo +nightly fmt --all --check exit 0; cargo test -p orb-ui exit 0, 23 binaries, 305 passed, 0 failed.

Review round 1 (send back, coordinator): (1) docs/plugin.md:892-898 rewrapped to the paragraph's width (the 153-column line 896 is gone; no line over 100 bytes in the paragraph). (2) The plan step 9 and the final summary now state the subject used: docs: (the current AGENTS.md rule), not docs,tools. The plan step 7 and 8 also record the orb-ui comment fix and the fmt and orb-ui test runs.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Outcome: the VST logo and the trademark sentence now appear only in Help > Plugin Usage (docs/plugin_usage.md, unchanged). The task's acceptance criteria are checked (AC 1 grep, AC 2 developer docs, AC 3 cargo test, bash -n, changelog run).

Branch and commit: worktree-agent-acf4ed6561fc81b68, one commit on main ad4d5f5, subject 'docs: VST logo and trademark line only in Plugin Usage' (docs: per the current AGENTS.md rule that doc text in scripts is docs). The hash is recorded in the handback, since the review round amends it.

Changes: README.md (logo block, sentence, License sentences, registered sign on VST 3 removed); scripts/package.sh, package-pkg.sh and package-dmg.sh (package texts); scripts/changelog.sh (release-notes sentence and its comment, now 'The plugin is a VST 3 and CLAP instrument.'); docs/building.md, docs/plugin.md and assets/README.md (attribution is in Help > Plugin Usage only); crates/orb-ui/src/plugin_usage.rs:510 (comment 'as the README does' now 'as the page's picture does'). Kept: docs/plugin_usage.md, assets/steinberg/*.svg and the developer-doc trademark lines docs/plugin.md:3, docs/building.md:901 and assets/README.md:69.

Numbers and tests: grep -rn -E 'Steinberg|VST®|registered trademark' README.md scripts/ gives no output; cargo test 895 passed, 0 failed, 5 ignored over 93 binaries; cargo test -p orb-ui 305 passed; cargo +nightly fmt --all --check clean; bash -n OK on the four scripts; scripts/changelog.sh HEAD exit 0.
<!-- SECTION:FINAL_SUMMARY:END -->
