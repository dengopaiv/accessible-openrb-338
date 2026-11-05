---
id: TASK-176
title: 'Third-party songs test: compare song paths in Unicode NFC'
status: Done
assignee:
  - '@dengopaiv'
created_date: '2026-11-05T19:30:00Z'
updated_date: '2026-11-05T19:31:00Z'
labels:
  - tests
milestone: m-5
dependencies: []
priority: medium
type: bug
ordinal: 199000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Why: on Windows (and Linux) the gated `songs` test (crates/orb-cli/tests/thirdparty.rs) reported 4 song files of the public rb-songs collection as not in tests/refs/thirdparty_songs.json, although all 1268 renders equalled rbhost's. Each of the 4 has a letter with a diacritic in its path. The manifest holds those paths decomposed (NFD: `u` + U+0308), rb-songs' git stores them composed (NFC: U+00FC) and Windows and Linux check them out that way; the test compared the bytes.

What: the test compares a song file's path with the manifest's in NFC (`same_path`, dev-dependency unicode-normalization 0.1, with a unit test). tools/rbhost/thirdparty_songs.py writes the manifest's paths in NFC (`nfc`) and still opens files by the file system's path. docs/rbm.md section 11.5 (the run of 2026-11-05) records it. Done in commit 4d76af3, before this task was created.

References (the CLI sets references only at creation): crates/orb-cli/tests/thirdparty.rs, crates/orb-cli/Cargo.toml, tools/rbhost/thirdparty_songs.py, tests/refs/thirdparty_songs.json, docs/rbm.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The songs test accepts a manifest path and a file path that are equal in NFC, with a unit test
- [x] #2 The gated thirdparty test passes on Windows with the public collections
- [x] #3 The manifest tool writes NFC paths
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Fixed in 4d76af3 (main of the fork, pushed 2026-11-05).

Before: on Windows 11 with rb-mods fe08cb8 (320 recrypted mods) and rb-songs 4af1075, `mods` passed and `songs` failed on 4 NFD paths; every render equal.
After: ORB_THIRDPARTY_MODS_DIR / ORB_THIRDPARTY_SONGS_DIR set, `cargo test --release -p orb-cli --test thirdparty`: 3 pass (mods; songs: 1268 distinct songs equal to rbhost's renders, 531 on 43 third-party mods; same_path's unit test), 473 s. clippy clean. The tool, imported with its test-machine client stubbed, would write the same 1270 names as the manifest's in NFC and differ from the committed raw strings in exactly the 4 paths.
<!-- SECTION:FINAL_SUMMARY:END -->
