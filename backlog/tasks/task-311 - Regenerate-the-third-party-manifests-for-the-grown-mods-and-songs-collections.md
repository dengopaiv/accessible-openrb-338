---
id: TASK-311
title: Regenerate the third-party manifests for the grown mods and songs collections
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T16:09:00Z'
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
- [ ] #1 tests/refs/thirdparty_mods.json lists every mod in ~/rb-mods/mods (renames and the four made mods included, with how mods without an original are handled documented)
- [ ] #2 tests/refs/thirdparty_songs.json lists every song of ~/rb-songs/songs with its rbhost render hash (or a documented skip reason)
- [ ] #3 cargo test --release -p orb-cli --test thirdparty passes with both variables set; any orb/rbhost difference is recorded as a finding and reported
- [ ] #4 README.md and docs/rbm.md 11/11.5 give the new counts; cargo test --workspace, clippy and nightly fmt --check pass
<!-- AC:END -->
