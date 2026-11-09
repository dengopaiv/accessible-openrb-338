---
id: TASK-221
title: 'Release packages: README.txt names the fork; the installer builds on Windows'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-09T17:54:00Z'
updated_date: '2026-11-09T18:02:00Z'
labels:
  - tools
  - docs
milestone: m-7
dependencies: []
references:
  - scripts/package.sh
  - scripts/package-installer.sh
  - README.md
priority: high
type: docs
ordinal: 239000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: the user is getting the fork ready for its first public release (2026-11-09), to be announced where OpenRB-338 is being written about. The README.txt inside the Windows zip (scripts/package.sh) and the installer (scripts/package-installer.sh) is upstream's text: it does not say that this is the screen reader accessible fork, how screen reader mode turns on, or which keys to start with, and the zip's points to docs/building.md, which is not in the package. README.txt is the first file a blind user opens after unzipping or on the installer's finish page.

What: both README texts get a paragraph on the fork (its GitHub page and upstream's) and on screen reader mode (Automatic with NVDA, Options > Screen Reader Mode, F1 key list, F5, F6 / Shift+F6, Tab, F10 or Alt, the full guide in the README on GitHub). The zip's unsigned note says what SmartScreen shows and what to choose, as the installer's does. Keys as in README.md "Using OpenRB-338 with a screen reader". The program and the packages' names are unchanged.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The zip's and the installer's README.txt name the fork, its GitHub page and upstream's, and tell how screen reader mode turns on and the first keys, matching README.md
- [x] #2 The zip's README.txt no longer points to docs/building.md; it says what SmartScreen shows for the unsigned program
- [x] #3 scripts/package.sh x86_64-pc-windows-msvc and scripts/package-installer.sh x86_64-pc-windows-msvc build; the texts are checked in the built zip and installer stage
- [x] #4 scripts/package-installer.sh builds the installer on Windows (Git Bash, NSIS 3.13), where it failed before with 'File: ... path.ps1 -> no files found' (openrb.nsi line 131); macOS and Linux unchanged
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. scripts/fork-readme.txt: the fork paragraph, read by package.sh and package-installer.sh into README.txt. 2. package.sh: the unsigned note per target, SmartScreen on Windows, no docs/building.md pointer. 3. package-installer.sh: Windows paths for makensis via cygpath -aw, NSIS's default folder when makensis is not on PATH. 4. docs/building.md: the Windows build of the installer. 5. Stage both packages from existing binaries (DIST, SKIP_BUILD=1) and read the texts.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: task-221-release-readme

Found while checking: scripts/package-installer.sh had never run with Windows' makensis (upstream builds installers on macOS and Linux). On main it fails here with makensis -V4 showing: File: "C:/Users/.../installer-stage/path.ps1" -> no files found, then "Error in script openrb.nsi on line 131". The same run with paths from cygpath -w builds the installer. Fix in the script: nsis_path() = cygpath -aw where cygpath exists, unchanged elsewhere.

Check (2026-11-09): DIST=<scratch> SKIP_BUILD=1 with the binaries of dist/x86_64-pc-windows-msvc/bin: scripts/package.sh x86_64-pc-windows-msvc made OpenRB-338-2026.1109.1755-x86_64-pc-windows-msvc.zip, its README.txt read with unzip -p (fork paragraph, keys, SmartScreen note, no docs/building.md). scripts/package-installer.sh x86_64-pc-windows-msvc, with makensis not on PATH, made OpenRB-338-2026.1109.1758-x64-setup.exe (8.6 MB); its README.txt read with 7z e -so: the fork paragraph, 33 lines, 33 CRs (CRLF, no doubled CR).

Review round 1 (agent-reviewer, send back): (1) the co-author trailer should be Sonnet because the assignee is @agent-worker. Not taken: the supervisor (Opus) did this task itself, so the trailer stays Opus and the assignee becomes @supervisor. (2) package.sh also changes the tar.gz README of Linux and macOS: now said in the commit message. (3) The task was created on this branch, so its file appears with the commit as Done. (4) README.md Known limits still said NVDA had not been tried by the user: replaced with the 2026-11-07 check (docs/accessibility_plan.md); "every control has a name and a spoken value" kept, as README.md says "every control has a name, a role and a value". (5) The SmartScreen note now covers both programs ("when one of them first starts"). (6) Zip README LF: as before, Notepad reads it.

After review round 2 (merge): README.md's Download link and Quick start step 1 pointed to upstream's Releases, whose packages have no screen reader mode. Now they point to the fork's Releases (Windows x64: installer and zip), and to upstream's for macOS, Linux and Windows on ARM. Sent for a third look.

Review round 3 (README links): the text was found right; send back only until a release exists at the fork's Releases page. Order kept: merge, push, build the packages from the pushed main, create the release as a draft for the user to publish before the README is announced. The minor point (the plain-archives line read as upstream's only) fixed. Merged by the supervisor (fast-forward of the one commit onto main) on 2026-11-09.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The Windows zip's and the installer's README.txt now carry scripts/fork-readme.txt: the fork and upstream's GitHub pages, how screen reader mode turns on, F1, F5, F6, Tab and F10 (as README.md's guide). The zip's unsigned note says what SmartScreen shows (Windows) instead of pointing to docs/building.md. scripts/package-installer.sh now builds on Windows: Windows paths for makensis through cygpath -aw (before: openrb.nsi line 131, path.ps1 'no files found'), and NSIS's default folder when makensis is not on PATH; without cygpath nothing changes. docs/building.md (The Windows installer) says so. Checked by staging both packages from existing binaries (DIST, SKIP_BUILD=1) and reading README.txt out of the zip (unzip -p) and the installer (7z e -so; CRLF, 33 lines). Scripts and text only, no Rust change. Branch task-221-release-readme.
<!-- SECTION:FINAL_SUMMARY:END -->
