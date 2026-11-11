---
id: TASK-228
title: A changelog for the fork's releases, in the repository and the zip
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-11T13:22:00Z'
updated_date: '2026-11-11T13:28:00Z'
labels:
  - docs
  - tools
milestone: m-7
dependencies: []
references:
  - scripts/package.sh
  - docs/building.md
  - scripts/fork-readme.txt
priority: medium
type: docs
ordinal: 246000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user, 2026-11-11: "Make sure there is a changelog to go with the work we have recently done, then release an update." The fork had no changelog: the first release's notes (v2026.1109.1831) were scripts/changelog.sh's list of commit subjects, which says what changed in developer terms, not what a person playing music notices.

What: a CHANGELOG.md at the repository root, the fork's own (upstream has none, so a rebase on upstream does not conflict), newest release first, each release a heading with its version and date and short plain-language lists (new, changed, left out, known limits), written to read well with a screen reader (headings and lists, no tables). It covers the first release, v2026.1109.1831, and the next one, whose tag is picked when its section is written. scripts/package.sh copies it into the fork's Windows zip as CHANGELOG.txt (Notepad opens it), and scripts/fork-readme.txt names it. docs/building.md, 'Making a release of the fork', gains the step: before the build, the release's section of CHANGELOG.md is committed on main, and the release notes are a short introduction plus that section.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 CHANGELOG.md exists at the repository root with a section for v2026.1109.1831 and one for the next release, in plain language, with no tables
- [x] #2 Every change on main since v2026.1109.1831 that a user can notice is in the next release's section: upstream PR #4 (Save writes the engine's tracks), the numpad keys (task-225), the beat cookbook (task-226) and the decision to leave out orb-mcp
- [x] #3 docs/building.md's release steps say when the changelog section is written and that the notes are built from it
- [x] #4 scripts/package.sh puts CHANGELOG.txt into the Windows zip only, and the zip's README.txt names it (not scripts/fork-readme.txt, which the installer's README shares, and the installer carries no changelog)
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-228 (in the main checkout; the supervisor's own docs work, as the release needs it).

Changed from the description: the README line naming CHANGELOG.txt is in package.sh's Windows-only paragraph, not scripts/fork-readme.txt, because package-installer.sh puts fork-readme.txt into the installer's README too and the installer carries no changelog (AC #4 says so).

The next release's version is v2026.1111.1330, picked before writing its section (building.md step 1).

Evidence: the changes since v2026.1109.1831 are git log cc62823..197a6cb: 32a642d (PR #4, via merge 780543b, task-224), e909060 (task-225), 197a6cb (task-226), and backlog commits 90e7b03 and 08992d1. The numpad list follows crates/orb-ui/src/key_help.rs group "The numpad"; the first release's units were checked in crates/orb-ui/src/a11y/units.rs (hertz, decibels, milliseconds, percent, semitones); its read-outs F2 to F4 come from docs/accessibility_plan.md A11Y-5.

Check: SKIP_BUILD=1 bash scripts/package.sh x86_64-pc-windows-msvc (the old binaries) made a zip with CHANGELOG.txt byte-identical to CHANGELOG.md (cmp), and its README.txt ends the portable paragraph with 'CHANGELOG.txt lists what changed in each release of the fork.' That test zip was deleted. bash -n scripts/package.sh passes.

Review round 1 (send back), fixed:
1. CHANGELOG.md, first release: knobs step to the next value 'spoken differently' (by 1 with Speak Raw Values), as crates/orb-ui/src/a11y/drive.rs next_different and README.md say, not 'sounds different'.
2. The zip's CHANGELOG.txt is now plain text with CRLF: package.sh strips heading marks, bold, backticks and link targets (sed -E) and writes CRLF (awk). Checked with SKIP_BUILD=1 bash scripts/package.sh: no '**', backtick, '](' or leading '#' left, 86 of 86 lines end in CR LF; test zip deleted.
3. The numpad sentence names the exception: in screen reader mode numpad * does nothing while a window that has the keys is open (crates/orb-ui/src/app.rs, numpad::take_multiply loop).
4. docs/building.md step 2: the heading's version is the tag without its v.
5. (process) The task file was created on the branch, already Done in its one commit: the supervisor made the task and did the work at once, for the release the user asked for.
6. (attribution) The supervisor is Claude Opus 5.5, so the commit's Co-Authored-By line is right.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
CHANGELOG.md at the repository root: the fork's changes per release in plain language with no tables, the first release v2026.1109.1831 and the next one, v2026.1111.1330 (numpad keys, task-225; the beat cookbook, task-226; Save writes the engine's tracks, upstream PR #4 via task-224; orb-mcp left out; the numpad not yet tried by hand). scripts/package.sh writes it into the Windows zip as CHANGELOG.txt, plain text with CRLF (Markdown marks and link targets stripped), and the zip's README.txt names it. docs/building.md's release steps pick the tag first and write the changelog section before the tests and build, and the notes are built from that section. README.md and AGENTS.md point at it. Branch task-228, one review round. Testing: packaging runs with SKIP_BUILD=1 (zip contents, no Markdown marks, CRLF on every line), bash -n; no cargo test (no code changed).
<!-- SECTION:FINAL_SUMMARY:END -->
