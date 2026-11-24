---
id: TASK-296
title: Split orb play into its own orb-play binary, so orb needs no audio output
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-24T18:02:00Z'
labels:
  - cli
  - player
  - tools
  - docs
dependencies: []
references:
  - crates/orb-cli/src/main.rs
  - crates/orb-cli/Cargo.toml
  - scripts/build.sh
  - scripts/package.sh
  - docs/building.md
  - README.md
priority: low
type: enhancement
ordinal: 269700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked for this on 2026-11-24; it is unscheduled. Today the `orb` CLI (crate orb-cli) depends on orb-player for `orb play` (crates/orb-cli/src/main.rs around line 571, `orb_player::output::start`), so it links cpal and the platform's audio libraries. On Linux that means libasound. `orb render`, `compare`, `info` and `mod` need none of that: they should build and run on a machine with no audio output device or audio libraries, such as a server, a container or CI.

**What changes:**
- **A new binary `orb-play`**, in a new crate crates/orb-play, plays a song in the real-time player as `orb play` does today. It keeps the same arguments (the song, `--mod-dir` / `ORB_MOD_DIR`), the same start and exit lines, the same exit codes, and the default output with 2048 frames (task-191 left `orb play` unchanged). The shared argument and mod handling it needs comes from orb-cli as a library, or moves to wherever avoids duplication; orb-cli's lib must not pull in orb-player.
- **orb-cli no longer depends on orb-player or cpal**, directly or through another crate. Check with `cargo tree -p orb-cli -i cpal`, which must find nothing.
- **`orb play` stays as a subcommand** but only prints a message that playback moved to `orb-play` (e.g. "orb play moved to its own program: run orb-play SONG.rbs ..."). It exits with an error code (non-zero, documented) and accepts and ignores its old arguments, so old scripts get the message and not a parse error. `orb --help` lists it as moved.
- **Packaging:** scripts/build.sh and scripts/package.sh build and ship `orb-play` next to `orb` on all six targets (orb-play.exe on Windows). Update the macOS app or DMG layout if it carries `orb`, the README.txt run lines and the release workflow if it names the binaries. docs/building.md gets the binary table and the Linux build-dependency note (orb builds without ALSA dev headers; orb-play and the app need them).
- **Docs:** every `orb play` mention in README.md, docs/building.md, docs/ui_spec.md, docs/plugin.md, docs/rbm.md and docs/midi.md (and the site if it mentions it) becomes `orb-play`. AGENTS.md's run line too, if it names `orb play`.
- **Tests:** existing `orb play` tests move to orb-play. A test checks that `orb play` prints the message and exits non-zero.

**Not a compatibility concern:** `orb play` shipped in releases, which is why it is kept as a message rather than removed.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A new orb-play binary plays a song exactly as orb play did (arguments, lines, exit codes, default output, 2048 frames)
- [ ] #2 orb-cli has no orb-player or cpal dependency (cargo tree -p orb-cli -i cpal finds nothing); orb render works on a Linux box without ALSA libraries (checked in a container or CI job without libasound)
- [ ] #3 orb play prints that playback moved to orb-play, ignores its old arguments and exits with a documented non-zero code; a test covers it
- [ ] #4 The packages of all six targets ship orb-play next to orb; README.txt, README.md and the docs name orb-play
- [ ] #5 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
<!-- AC:END -->
