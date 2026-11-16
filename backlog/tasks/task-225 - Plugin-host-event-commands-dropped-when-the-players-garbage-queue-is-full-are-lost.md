---
id: TASK-225
title: >-
  Plugin: host-event commands dropped when the player's garbage queue is full
  are lost
status: To Do
assignee: []
created_date: '2026-11-16T16:36:00Z'
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
- [ ] #1 With the garbage queue filled by main-thread commands (e.g. 128 pattern edits not collected), a Latch note's pattern selection, a note-mode note-on and note-off, and the host's start and stop all take effect within one block (orb-plugin tests through process_with)
- [ ] #2 A refused command is never only counted: Core::dropped_events stays 0 in those tests, or is documented as counting only retried commands
- [ ] #3 process still allocates nothing and takes no lock (tests/no_alloc.rs passes, extended with a full-queue case)
- [ ] #4 docs/plugin.md describes what happens when the queue is full
- [ ] #5 cargo test, clippy -D warnings and nightly fmt pass
<!-- AC:END -->
