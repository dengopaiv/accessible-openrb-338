---
id: TASK-240
title: 'Take upstream''s third-party manifests: 326 mods, 5685 songs'
status: Done
assignee:
  - '@supervisor'
created_date: '2026-11-27T15:43:00Z'
updated_date: '2026-11-27T16:06:00Z'
labels:
  - tests
  - tools
dependencies: []
references:
  - https://github.com/pingw33n/openrb-338/commit/cabcca4
priority: medium
type: chore
ordinal: 257000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on 2026-11-27 the local clones of rb-mods and rb-songs (C:\GIT\music\rb-collections) were pulled to upstream's main (rb-mods bfbd8c9, rb-songs 6e0b92e). They now hold 326 recrypted mods and 5685 songs, while the fork's manifests (tests/refs/thirdparty_mods.json and thirdparty_songs.json, from the merge of upstream f83c54f in task-233) list 324 mods and 4834 songs, so the gated thirdparty tests fail on the unlisted files.

Upstream regenerated both manifests for exactly these collections in its commit cabcca4 (upstream's task-317, rendered with rbhost on its test machine). This task takes that commit's manifests and its count updates in README.md and docs/rbm.md. It leaves out upstream's backlog file (the fork does not take upstream tasks from 176 on, task-229) and tools/build_site.py (the site was removed in task-230). AGENTS.md's mod count in the fork section is brought to 326. The rest of upstream's newer main (the web version, its tasks 170 to 174, 318 and 319) is not taken: the fork has no web version (task-198).

Before the pull, rb-songs had one local change, an accidental save of songs/You Know.rbs from 2026-11-07 (the user confirmed it was accidental); it was stashed for the pull and then dropped.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 tests/refs/thirdparty_mods.json and tests/refs/thirdparty_songs.json equal upstream cabcca4's files
- [x] #2 The manifests list exactly the files of the pulled clones: 326 mods and 5685 songs, none missing and none unlisted
- [x] #3 cargo test --release -p orb-cli --test thirdparty passes with ORB_THIRDPARTY_MODS_DIR and ORB_THIRDPARTY_SONGS_DIR set to the clones
- [x] #4 No upstream backlog file, no tools/build_site.py and no web-version change is brought in
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: thirdparty-manifests-326-5685. Applied upstream cabcca4 with cherry-pick -n; its modify/delete conflicts (upstream's backlog task-317 file, tools/build_site.py) kept deleted. Manifests compared to cabcca4: identical sha256. Against the clones (Python, names NFC-normalized): mods 326 listed, 326 on disk, 0 unlisted, 0 missing, 0 hash differences; songs 5685 listed, 5685 on disk, 0 unlisted, 0 missing, 0 hash differences, 0 skipped; 4 manifest paths are NFD, matched by same_path in the test. Run: ORB_THIRDPARTY_MODS_DIR=C:\GIT\music\rb-collections\rb-mods\mods ORB_THIRDPARTY_SONGS_DIR=C:\GIT\music\rb-collections\rb-songs\songs cargo test --release -p orb-cli --test thirdparty -- --nocapture: 3 passed in 1188 s (mods: 326 checked, 326 load, 0 fail; songs: 5685 distinct, RBS1X 754, RBS42 4931, all equal rbhost's renders, 0 not compared). Other stale counts: only dated run records in docs/rbm.md and docs/accessibility.md, left as written; AGENTS.md fork section 320 -> 326 mods. docs/rbm.md 11.5 got this run's record.

Review round 1 (agent-reviewer): send back on the commit message. The before-count was a raw byte comparison; under NFC, main's manifest left 1044 songs unlisted and listed 193 absent ones (plus 2 unlisted mods): the Result line now says so. The attribution line is right (the supervisor is Opus). The task was created Done in its one commit, so no In Progress state was committed, which is what the To Do to Done rule guards; noted in the merge comment. Also added the missing blank line before the run of 2026-11-25 in docs/rbm.md 11.5, which the reviewer noticed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Took upstream cabcca4's third-party manifests (326 mods, 5685 songs) and its count updates in README.md and docs/rbm.md, without upstream's task file and tools/build_site.py; AGENTS.md's fork mod count is 326. The manifests are byte-identical to cabcca4's and list exactly the files of the pulled clones (rb-mods bfbd8c9, rb-songs 6e0b92e). Gated thirdparty test: 3 passed in 1188 s, 326 mods load, all 5685 songs equal rbhost's renders. Branch thirdparty-manifests-326-5685, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
