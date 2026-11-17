---
id: TASK-225
title: >-
  Plugin: host-event commands dropped when the player's garbage queue is full
  are lost
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-16T16:36:00Z'
updated_date: '2026-11-17T09:36:00Z'
labels:
  - player
  - engine
milestone: m-7
dependencies: []
references:
  - crates/orb-plugin/src/core.rs
  - crates/orb-player/src/lib.rs
  - docs/plugin.md
priority: medium
type: bug
ordinal: 230000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The plugin's process side (crates/orb-plugin/src/core.rs, Engine::send) applies host events at once with orb-player's Audio::apply_now (crates/orb-player/src/lib.rs). apply_now refuses a command whenever the garbage queue has fewer than GARBAGE_PER_COMMAND free slots, even for a command that makes no garbage; core.rs then only counts it (Core::dropped_events) and never retries it. The garbage queue fills when the main thread sends many commands that return garbage (pattern edits, a song load, a mod) faster than it collects (OrbParams::collect_garbage runs at each main-thread command and each editor frame); task-218's test fixture reproduced it: 128 pattern edits before the first blocks, then the first four Gate commands were refused.

What is lost today when that happens:
- Latch Pattern Mode: a note's DeviceParam (pattern controller) and Command::Relocate: the pattern does not change, or changes only at the next bar line.
- Note mode: Command::Note: a note-on is not heard, or a note-off is lost and a 303 note or a held drum key hangs.
- The host sync (Engine::sync): Play, Stop, Locate, Position, Tempo: the song may not start or stop with the host (sync re-sends Tempo and the start/stop state at the next block only where it compares engine state; check each).
- Parameter changes (apply_one) and the Standard Mapping's actions: the value stays wrong until it changes again.
Only Gate Pattern Mode repairs itself (task-218: gates() compares each engine gate with the held notes before every block and resends).

**What to do.** Make no host event silently lost. Options to weigh (pick and record the choice in the notes): (a) let apply_now apply commands that return no garbage regardless of the garbage queue's room (most host-event commands carry no box; only DeviceData, Pattern, Load, Mod and Preview do); (b) a fixed-size retry ring in core.rs for refused commands, applied before the next block (no allocation in process); (c) per-feature reconciliation like task-218's gates (compare engine state with the wanted state before each block). (a) looks smallest; check GARBAGE_PER_COMMAND's reasoning in orb-player before relying on it. Real-time rules hold: no allocation and no lock in process (tests/no_alloc.rs).

Engine numerics are not involved; if orb-engine changes, run the full corpus (ORB_CD_ISO=... cargo test --release -p orb-engine --test corpus_golden) with zero differences.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 With the garbage queue filled by main-thread commands (e.g. 128 pattern edits not collected), a Latch note's pattern selection, a note-mode note-on and note-off, and the host's start and stop all take effect within one block (orb-plugin tests through process_with)
- [x] #2 A refused command is never only counted: Core::dropped_events stays 0 in those tests, or is documented as counting only retried commands
- [x] #3 process still allocates nothing and takes no lock (tests/no_alloc.rs passes, extended with a full-queue case)
- [x] #4 docs/plugin.md describes what happens when the queue is full
- [x] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Choice: option (a), made exact. Research: Audio::apply_now/drain (crates/orb-player/src/lib.rs) refuse any command while the garbage queue has < GARBAGE_PER_COMMAND (2) free slots. Of the commands only DeviceData, Pattern, Mod (1 box each), Preview (1 when one played), Load (up to 2: the pending and/or the current song) and Play (1 when a song is pending: switch -> make_current throws the old engine) hand anything back. Every command the plugin's process side sends (params::command, the Standard Mapping's Action::command, Note, Gate, DeviceParam, Relocate, Tempo, Position, Locate, Stop, Play) carries no box; only Play can make garbage, when a song is pending.
1. orb-player: Audio::room_for(c): a command that hands nothing back always fits; Play needs one slot while a song is pending; a box command (DeviceData, Pattern, Mod, Preview, Load) needs GARBAGE_PER_COMMAND slots plus one more while a song is pending (the reserve for that song's Play). Invariant: while a song is pending at least one slot is free, so Play and every box-free command always fit. drain peeks and applies queued commands in order while the next fits (a box command that does not fit stalls it and what follows); apply_now refuses only what does not fit. Add Audio::garbage_room() for tests.
2. orb-plugin core.rs: dropped_events documented as refusals of a box command, which the process side never sends (stays 0); module doc on the full queue.
3. Tests: new crates/orb-plugin/tests/full_garbage.rs: with 128 uncollected pattern edits filling the queue, a Latch note's pattern (shown and playing), a note-mode note-on/off (sounding) and host start/stop take effect within one block, dropped_events 0; orb-player unit tests for room_for (pending reserve, Play with pending at 1 slot). gate_mode.rs: dropped_events now 0. no_alloc.rs: a full-queue phase.
4. docs/plugin.md: what happens when the queue is full.
5. cargo test, clippy, nightly fmt; bundle + scripts/validate-plugin.sh. orb-engine is untouched (no corpus run needed).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad21b533a3b807736 (base main 13b61ee).

Research (crates/orb-player/src/lib.rs at 13b61ee): GARBAGE = 64, GARBAGE_PER_COMMAND = 2 ('a load replaces the pending song and the current one'). drain applied commands while slots >= 2, apply_now refused any command below 2 free slots. What each command hands back (Audio::apply/throw): DeviceData, Pattern, Mod: 1 box; Preview: 1 when one played; Load (activate): the old pending song, plus the current one when it does not play (make_current): up to 2; Play: 1 (the current song, switch -> make_current) only while a song is pending; everything else 0. The plugin's process side (core.rs Engine::send) sends only box-free commands (params::command, StandardMapping Action::command: DeviceParam, MixerGlobal, MixerChannel, Shuffle, ...; Note, Gate, Relocate, Tempo, Position, Locate, Stop, Play). So option (a), but Play needs care: refused with a pending song and 0 slots, a Note sent after it would land on the pending song.

Choice: (a) made exact, no retry ring (b): a retry ring cannot drain while the main thread does not collect (editor closed: OrbParams::collect_garbage runs only at main-thread commands and editor frames), and per-feature reconciliation (c) would be needed for every event kind. New Audio::room_for(c): box-free -> always; Play -> 1 slot while a song is pending; box commands -> GARBAGE_PER_COMMAND + 1 while a song is pending. Invariant: pending => >= 1 free slot (a Load that makes a song pending hands back nothing; a box command applied while pending hands back <= 2 of the >= 3 it required; only Play ends pending). drain now peeks: applies in order while the next command fits (a box-free main-thread command behind a stalled box command still waits: order kept). Audio::garbage_room() added for tests.

Failed hypothesis in my own test: after a Load over the playing song and 128 edits, I expected 1 free slot; it is 2 (the edits need 3 while pending and stop at 2). Asserted GARBAGE_PER_COMMAND.

Tests: crates/orb-player/tests/garbage.rs (2: box-free commands incl. Play/Stop/Locate/Note/DeviceParam/Relocate/Tempo pass a full queue, a box one is refused, the queued Tempo follows once collected; a pending song's Play fits at the kept slot). crates/orb-plugin/tests/full_garbage.rs (2): 128 uncollected 808 pattern-31 edits (same bytes) fill the queue from block 0 (garbage_room < 2 every call); Latch: host start (call 4 -> playing after call 5), note ch1 #5 at call 20 (pattern shown and playing after it, not before), host stop at 40 / start at 50 within one block; note mode: 303 note on at 10 / off at 30 (sounds, then < 0.002 at the end: no hung note), 808 BD at 50 lights sounding at 51; both sessions frame-for-frame equal to an instance that collects after every call; dropped_events 0. With the old rule (room_for returning slots >= 2) both tests fail (Latch: not playing at call 5; note: 808 BD not sounding). gate_mode.rs: its fixture's dropped_events() > 0 is now == 0. no_alloc.rs: a full-queue phase in Latch, Gate and Note (128 uncollected 909 edits each, 5000 frames of host notes/transport), at 44100, 48000 and 44101 Hz: 0 allocations, dropped 0.
Commands: cargo test -p orb-player --test garbage; cargo test -p orb-plugin --test full_garbage --test gate_mode --test no_alloc.
Full: cargo test (ORB_CD_ISO set) 861 passed, 0 failed, 5 ignored; clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean. orb-engine unchanged: no corpus run needed.

Bundles: cargo xtask bundle orb-plugin --release (CLAP + VST3 built, not installed); scripts/validate-plugin.sh: clap-validator and pluginval all passed (exit 0). Rebased onto main f1a9aa0 (backlog-only commits); git diff main --stat lists only this task's files.

Review round 1 (verdict merge, two fixes before the merge): (1) crates/orb-player/src/lib.rs Audio::throw: its comment named the old rule (call keeps GARBAGE_PER_COMMAND slots free); it now refers to room_for and the slot kept for a pending song's Play. (2) docs/plugin.md section 1: host events (apply_now) are not ordered after main-thread commands still queued behind a stalled box command (the editor's pattern select, the panel's Play/Stop), a window of about one editor frame. Rebased onto main eeeebdb; commit message checked for the new no-number-sign rule.

After review round 1 on main eeeebdb: cargo test 862 passed, 0 failed, 5 ignored (main added one); clippy -D warnings and nightly fmt --check clean; bundles rebuilt (target/bundled/OpenRB-338.clap, OpenRB-338.vst3) and scripts/validate-plugin.sh: all passed.

Rebased onto main c100bbe (task-228, task-229, task-214): cargo test 868 passed, 0 failed, 5 ignored; clippy -D warnings and nightly fmt --check clean; bundles rebuilt and scripts/validate-plugin.sh: all passed.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Host events are no longer lost when the player's garbage queue is full. orb-player (crates/orb-player/src/lib.rs): new Audio::room_for decides by what a command hands back: a box-free command always fits, Play while a song is pending needs one slot (kept free: a box command takes GARBAGE_PER_COMMAND + 1 while a song is pending), DeviceData/Pattern/Mod/Preview/Load need GARBAGE_PER_COMMAND. apply_now refuses only what does not fit, and drain peeks and applies the main thread's commands in order while the next fits; Audio::garbage_room for diagnostics. The plugin's process side sends only box-free commands, so nothing it sends is refused: Core::dropped_events is documented as counting only refusals of a box command and stays 0. Option (a) of the task, chosen over a retry ring (it cannot drain while the main thread does not collect) and per-feature reconciliation.

Tests: orb-player tests/garbage.rs (2), orb-plugin tests/full_garbage.rs (2: Latch note, host start/stop, note-mode note-on/off and drum hit within one block with 128 uncollected edits, frame-for-frame equal to an instance that collects; both fail under the old rule), gate_mode.rs fixture now dropped == 0, no_alloc.rs extended with a full-queue phase in each mode at three rates. docs/plugin.md section 1 describes the full queue; section 11 updated. cargo test 861 passed / 0 failed / 5 ignored; clippy -D warnings and nightly fmt clean; bundles built and validators passed. orb-engine untouched (no corpus run). Branch worktree-agent-ad21b533a3b807736, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
