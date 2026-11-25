---
id: TASK-311
title: Regenerate the third-party manifests for the grown mods and songs collections
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T16:09:00Z'
updated_date: '2026-11-25T18:50:00Z'
labels:
  - tests
  - tools
  - rbm
  - testmachine
milestone: m-5
dependencies: []
references:
  - tools/thirdparty_mods_manifest.py
  - tools/rbhost/thirdparty_songs.py
  - crates/orb-cli/tests/thirdparty.rs
  - docs/rbm.md
priority: high
type: task
ordinal: 282700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user changed both third-party collections on 2026-11-25. The opt-in third-party tests (crates/orb-cli/tests/thirdparty.rs: `ORB_THIRDPARTY_MODS_DIR`, `ORB_THIRDPARTY_SONGS_DIR`; AGENTS.md, docs/rbm.md 11 and 11.5) fail on any file their manifests don't list. Afterwards the supervisor runs every test, the third-party ones included.

**The collections** (read only: never modify, delete or commit their contents; they are git repos github.com/pingw33n/rb-mods and rb-songs):
- **~/rb-mods/mods:** 324 recrypted mods. tests/refs/thirdparty_mods.json lists 320.
  - The latest rb-mods commits rename mods to the names songs reference (4cb83ec).
  - They add four mods made from the sample-swap instructions in songs' ReadMes (0e465e8: CaveyRageRemix, Maracatu, RB339Zam, SaturdayMorning).
  - Those four have no original encrypted file.
- **~/rb-songs/songs:** about 4,860 files, many new sites (propellerheads.se, caveman.demon.nl, dance.nl, rotorkopf.dk, mrt.u-net.com, tb303.com, ...). tests/refs/thirdparty_songs.json lists 1268 songs (1270 files). Not every file is a song (ReadMes and the like); see how the test and thirdparty_songs.py pick song files.

**What to do:**
1. **Mods manifest.** Update it for the renamed and new mods.
   - tools/thirdparty_mods_manifest.py recrypts ORIGINAL mods and requires equality with the recrypted folder.
   - The originals folder (~/rb_stuff/Mods.orig) no longer exists. The last update, task-168 (88a1770), "matched originals by content"; read its notes for where they came from.
   - Renames may be handled by matching content (sha256) against the current manifest.
   - For the four made mods, and any mod with no original, decide how the manifest lists them. For example, list them with the recrypted file's hash and the slot hashes from orb's own reader, marked as having no original, and make `--check` skip them.
   - If that needs a choice the user should make, stop and report the options before changing the tool.
2. **Songs manifest.**
   - Regenerate it on the Windows test machine with tools/rbhost/thirdparty_songs.py (batched and resumable; docs/rbm.md 11.5).
   - Render only the new or changed songs if the tool allows it (`--only`, as task-168 did), keeping the existing entries for unchanged files.
   - Follow doc-001 for the test machine and its permits. It is shared with the user: touch only processes you start, and never disturb the user's programs.
   - This may take hours. Keep the notes current so the run can resume.
3. Run `cargo test --release -p orb-cli --test thirdparty -- --nocapture` with both variables set. Every song's orb render must equal rbhost's. A song where they differ is a finding: record it, with the first difference, and stop to report. Do not paper over it.
4. Update the counts in README.md and docs/rbm.md 11 and 11.5.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 tests/refs/thirdparty_mods.json lists every mod in ~/rb-mods/mods (renames and the four made mods included, with how mods without an original are handled documented)
- [x] #2 tests/refs/thirdparty_songs.json lists every song of ~/rb-songs/songs with its rbhost render hash (or a documented skip reason)
- [x] #3 cargo test --release -p orb-cli --test thirdparty passes with both variables set; any orb/rbhost difference is recorded as a finding and reported
- [x] #4 README.md and docs/rbm.md 11/11.5 give the new counts; cargo test --workspace, clippy and nightly fmt --check pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Mods: originals are gone (no rb_stuff); the 317 unchanged mods keep their manifest entries (--keep); the 7 others (3 renamed/edited: BadRat, noize, H2-69; 4 made) get entries read from the recrypted file with original:false; --check skips them. 2. Songs: stale check (new sha / resolution / mod hash), render all with thirdparty_songs.py on the test machine (resumable). 3. thirdparty cargo test. 4. README/docs counts, unit test, workspace test, clippy, fmt.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ac109b29212ec097d

Mods: collection 324 mods (+.DS_Store). 317 equal the old manifest by name+sha; 3 (BadRat, noize, H2-69 (was H2-69_MIXED)) are same-size files whose INFO name changed (noize: noïze->noize; BadRat: 'BadRat BR-338 2.0'->'BadRat BRM 2.0'), slots identical; 4 are new made mods. Originals folders do not exist. Tool change: --keep OLD_MANIFEST, --orig optional, original:false entries. Command: thirdparty_mods_manifest.py --mods ~/rb-mods/mods --dat DAT --keep OLD --orb /usr/bin/true -> '324 mods'; 317 entries unchanged, 7 new/changed lines.
Songs: stale check (script in scratchpad): 4834 distinct files (no duplicates now), 3612 not in the manifest, 0 skipped before, 0 resolution changes, 0 pinned-mod changes, 46 manifest songs gone from the folder (dropped). Render: nohup thirdparty_songs.py --songs ~/rb-songs/songs --mods ~/rb-mods/mods --rb20ful DAT --remote-dir t311 (log: scratchpad b/run.log; manifest rewritten after each song; to resume rerun the same command).

Render: thirdparty_songs.py --remote-dir t311, 3612 songs in ~2h20m (one permit, released, remote folder deleted by the tool): '4834 songs rendered, 0 skipped'. Test: cargo test --release -p orb-cli --test thirdparty -- --nocapture (both variables): 'mods: 324 checked, 324 load, 0 fail; 124 keep built-in slots; 29395 embedded files and 12952 slots equal the manifest's'; 'songs: 4834 files ..., 0 duplicates, 4834 distinct: {RBS1X: 584, RBS42: 4250}; 4834 renders equal rbhost's, 0 not compared'; 2 passed (510 s). No orb/rbhost difference. cargo test --workspace: 1181 passed, 0 failed; clippy --all-targets -D warnings clean; nightly fmt --check clean; tools unit test 2 pass. --check was not run: the originals no longer exist.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Both manifests regenerated for the 2026-11-25 collections. thirdparty_mods.json lists all 324 mods: 317 entries kept (unchanged by name and sha), 7 without an original (BadRat, noize, H2-69 renamed/INFO-edited; CaveyRageRemix, Maracatu, RB339Zam, SaturdayMorning made) read from the recrypted file with original:false; tools/thirdparty_mods_manifest.py got --keep, optional --orig, and --check skips original:false entries (unit test added). thirdparty_songs.json lists 4834 songs (3612 rendered new by rbhost, 46 gone dropped), 0 skipped. orb equals rbhost on all 4834 (thirdparty test 2 passed). README and docs/rbm.md 11/11.5 updated. cargo test --workspace 1181 pass; clippy and nightly fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
