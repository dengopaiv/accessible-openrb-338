---
id: TASK-306
title: 'Edit: Initialize Song / Loop from Pattern Mode, as in the original'
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-25T13:06:00Z'
updated_date: '2026-11-25T14:48:00Z'
labels:
  - ui
  - re
milestone: m-5
dependencies: []
references:
  - docs/ui_spec.md
  - docs/song_load.md
  - docs/RBS42.txt
  - crates/orb-ui/src/menu.rs
priority: medium
type: feature
ordinal: 277700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user asked on 2026-11-25 for the original's Edit menu commands that initialize the song, or the loop, from Pattern Mode, behaving exactly as in ReBirth 2.01. docs/ui_spec.md section 12 lists only the menu items in scope so far and says what the original's other Edit items do "was not read".

**Part 1, reverse engineering** (Rebirth.exe and ReBirth.dll; the user's IDA databases, opened read only through idalib):
- Find the menu items and their exact names, command ids, enable conditions and handlers. Start from the menu resource and strings, and the WM_COMMAND dispatch that handles the ids in section 12.
- Read what each one writes:
  - which song tracks or events (pattern selection, mutes, knobs and other automated controls, tempo, shuffle);
  - at which position (the song start, the loop start, the current position);
  - what is overwritten or cleared;
  - how the loop start and length change;
  - whether the song gets modified and whether there is a confirmation dialog;
  - how the engine messages go (msg 35 and the like) and any undo.
- Check the facts against the disassembly, not only Hex-Rays, and give addresses.
- Write them up in the document they belong to (docs/ui_spec.md section 12 or 18, and docs/song_load.md or docs/RBS42.txt for the song events), marking each [C] or [G].
- Where the code leaves it unclear, confirm on the Windows test machine with ReBirth (doc-001: rcs_client.py and the permits). Save a song before and after the command and compare the files.
- A defect of the original goes into docs/original_bugs.md too.

**Part 2, implementation in orb:**
- The same items in the standalone app's Edit menu, with the original's names unless the user says otherwise. Their effect on the song is the original's, byte for byte in a saved .rbs, and so is their effect on playback.
- Check whether the plugin's editor should show them. The plugin's song mode and Edit menu are in docs/plugin.md and ui_spec section 20. If it is unclear, ask through the supervisor; it is a decision for the user.
- Tests: compare orb's result on a song with the original's saved file for the same song and command. Store the files as committed test inputs if they are small and not copyrighted (songs made on the test machine from the template), otherwise say how they are produced.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The original's commands are documented with names, ids, enable conditions, handlers and exactly what they write to the song (tracks, position, loop, modified flag, dialog), with addresses, [C]/[G], in docs/
- [x] #2 orb's Edit menu has the commands; on test songs orb's saved .rbs after each command equals the original's saved file for the same song and command (or the differences are documented as the original's bugs or orb-only fields)
- [x] #3 cargo test --workspace, clippy -D warnings (macOS and Windows) and nightly fmt --check pass
- [x] #4 The user has checked the commands in the app
- [x] #5 orb's own one-level Undo (not the original's) undoes Initialize Song and Initialize Loop: 'Undo Initialize Song/Loop', the tracks and the saved file byte-identical to before, also while playing, sharing the one Undo with the pattern commands; its callback cost measured; documented as orb's in docs/ui_spec.md
- [x] #6 Playback after the command: orb's tracks equal the original's node for node in 4 of 4 cases, and one export of the original (Initialize Loop on init_loop.rbs) equals orb's render of the saved song; the six exports that match at no idle count are recorded as open in docs/song_load.md section 14 (cause [G]) with follow-up task-308
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. RE (done, see notes): Edit 306 Initialize Loop from Pattern Mode and 307 Initialize Song from Pattern Mode, exe sub_42C204 / sub_42BC83 -> song msgs 29 and 28, DLL 0x1001243A / 0x100123C7.
2. Test songs tests/songs/edit_init (build.py over the Silent Default Song template): init_loop (loop bars 3..5) and init_zero (loop from 0).
3. Test machine: open each, run each command, Save As; export the loop after it; keep the saved files as test inputs.
4. Engine: split SongDevice::init_track into slot 7 (quantisation) and slot 8 (insert the pattern copy at a tick); seq: delete range except tick 0 (sub_10014910), insert a chased state (sub_10015240, also into the recording list with record on), the refresh chase (sub_1001FDC0); Engine::init_song / init_loop; player Command::InitSong / InitLoop.
5. UI: Edit menu items with the original's names, enabled in song mode only; Initialize Song asks with the original's text and buttons Cancel / Initialize; the song is marked modified; status line.
6. Tests: engine unit tests of both commands; a test that applies each command to the input song and compares orb's saved file with the original's byte for byte; renders vs the original's exports.
7. Docs: ui_spec 12 and 18, initial_investigation 5.2 (msgs 28, 29), song_load, original_bugs if a defect; checks; dev app; commit (still To Do).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-aa9a478c0a04f2813

RE (2026-11-25, copies of the user's IDA databases, read only):
- Menu resource SMNU 128 (strings in Rebirth.exe): 'Initialize Loop from Pattern Mode=306?101050', 'Initialize Song from Pattern Mode=307?101060' (next: 'Copy Touched Controls to Loop#L=308', '...to Song#T=309').
- Enable: sub_42BC83 (song window menu update, vtable 0x4BBFD4): only when the window's globals copy byte +0x1060 (song mode, global byte 0) is set: sub_470AB7(306,1) 0x42BFB4, sub_470AB7(307,1) 0x42BFC3; 308/309 with +0x1062 (record).
- Handler: sub_42C204 (WM_COMMAND, vtable slot before it), song-mode switch at 0x42C3C6: 306 (0x132) -> sub_40B194(song, 29) 0x42C59A, then sub_42590B (modified, +552 counter) 0x42C5A8; no dialog. 307 (0x133) -> sub_475D5C(500128003) 0x42C5B7 (SALR 128 item 3: '[WARN]This will overwrite your entire Song - all knob and button automation will be cleared.[Cancel][Initialize]'), result 33 -> sub_40B194(song, 28) 0x42C5E3, sub_42590B. Pattern-mode branch has no 306/307.
- DLL song msg table 0x10151D6C + 4*msg: 28 -> 0x100123C7, 29 -> 0x1001243A (30 0x100124EA, 31 0x10012508 are the Copy Touched Controls).
- msg 28: sub_1001FD70 (every track: sub_10014910(0, 0x7FFFFFFF) on +0x108), every device slot 8 (vtable +0x20) with tick 0 (edi=0, 0x100123EB), then sub_1001FDC0(trunc(pos)) (sub_1001F870).
- msg 29: sub_10021520(loopEnd) (every track: sub_10020350: chase sub_100151F0 at loopEnd, sub_10015240 inserts each chased controller at loopEnd into +0x108, and into +0x198 when record is on), sub_100208A0(loopStart, loopEnd) (sub_10014910 on every track), every device slot 8 with loopStart (sub_1001F8C0 = globals dword+3 = byte 12), sub_1001FDC0(trunc(pos)).
- sub_10014910(start, end): unlinks every node with start <= tick < end except tick-0 nodes (0x10014945 test of the tick), sets the list's +12 flag.
- sub_10015240(tick, state): ctrl 0..127 not 0xFF: tick rounded up by sub_100146E4, dropped above 0xBB4FF, same-tick same-ctrl node replaced, else a pool node linked before the tick's group.
- sub_1001FDC0(t): in song mode every track chased to t (state +0x84, applied pointer +0x228), list flag cleared (sub_10014660), every non-0xFF controller applied (sub_10012B40). No relocate, no position change.
- Slot 8 (303 0x10025650 read in disassembly): reads the pattern copy +0x30 (sub_10018C60) and inserts (tick, c, v) c=0..8 via sub_1001F900 -> sub_10014750.

Test machine (2026-11-25, all 6 redc-win-run permits, my ReBirth PID 22368 only, folder C:\re\rb\t306, scripts in the scratchpad: rcs input/screenshot):
- Songs: tests/songs/edit_init/build.py -> init_loop.rbs (loop 1536..3840) and init_zero.rbs (loop 0..1536), song mode, chunks differ from the tracks.
- In song mode the Edit menu reads Cut Loop, Copy Loop, Paste at Song Position, Paste Replace at Song Position, Clear, then 'Initialize Loop from Pattern Mode' and 'Initialize Song from Pattern Mode' enabled, Copy Touched Controls greyed (record off): docs/test_machine/menu_edit_song.png.
- Initialize Loop: no dialog. Initialize Song: box 'ReBirth RB-338' with the warning icon, text 'This will overwrite your entire Song - all knob and button automation will be cleared.', buttons Initialize (left, default: Enter chooses it) and Cancel (docs/test_machine/init_song_dialog.png).
- After each command: Save As and Export Loop. Saved: init_loop.after_init_loop.rbs (17550 B, sha1 db7c109f), init_loop.after_init_song.rbs (16840, 9867146a), init_zero.after_init_loop.rbs (17236, a7633b43), init_zero.after_init_song.rbs (16840, 6f083c49). WAVs: init_loop.after_init_loop.wav c63ada2f, init_loop.after_init_song.wav 4356c05e, init_zero.after_init_loop.wav 26b3151c, init_zero.after_init_song.wav 56cb60ae (sha1; kept in the scratchpad, not committed).
- The dumps match the model read from the disassembly: Init Song leaves only tick-0 events, holding the device chunks' (pattern copy's) values in the slot-8 order; Init Loop keeps the events before the loop and at/after its end, inserts at the loop end the state chased there (new nodes before the tick's existing ones, in descending controller order; an existing node of the same controller keeps its place and value), deletes [start, end) except tick 0, and inserts the pattern copy at the loop start (init_zero: at tick 0, replacing the tick-0 values).

Implementation and checks (2026-11-25):
- Engine: SongDevice::init_track split into slot 7 quantise and slot 8 insert_pattern(track, tick) (every device; the 303/808/909 read the pattern copy +0x30, the mixer +0x58); TrackMut::insert is now the pool-checked insert (sub_10014434), as sub_1001F900 -> sub_10014750 is. seq: Pool::retain, Track::remove_range (sub_10014910), Track::insert_state (sub_10020350 / sub_10015240, recording list too with record on), Sequencer::clear_tracks (sub_1001FD70), clear_range (sub_10021520 + sub_100208A0), refresh (sub_1001FDC0). Engine::init_song (msg 28) and init_loop (msg 29); orb_player::Command::InitSong / InitLoop.
- crates/orb-engine/tests/edit_init.rs: load each input, run the command, every track's list equals node for node (list order) the TRAK of the original's save: 4 of 4 equal.
- First UI comparison of the whole saved file: differs only in bytes the original's packers never write or zero: GLOB 483..511 and USRI 544..711 are never written by sub_433EC2 / sub_43489D (stale bytes of the shared buffer unk_4DB948; the four saves have GLOB[498] = 1 and USRI tail bytes set) -> OB-26; and COMP 3..7, which sub_4347D1 memsets to 0 (DELY sub_434605, PCF sub_4346E1, DIST sub_434567 likewise) while orb keeps the template's COMP[3] = 127 -> Decide task-307, decided by the user on 2026-11-25: option 1, keep the bytes as read. With those three ranges masked the four saves are equal byte for byte (crates/orb-ui/tests/edit_init.rs).
- Playback: the original's export after Initialize Loop on init_loop.rbs (first session) equals orb's render of the saved file at N = 32752 (orb compare --idle-blocks 32752: EQUAL; seed (export - open) = 48 s). Three more exports in fresh sessions (tools in the scratchpad: fresh.sh, fitn2.sh) match at no N in 0..30000: there the command changed the live 303 tune at the position (Initialize Song, and Initialize Loop with the loop at 0) a few seconds after the open, so the free-running 303 phase (docs/initial_investigation.md 'idle blocks') depends on the time before and after the command, which a one-N render cannot model; the first difference is at frame 66, the first 303 sample. Failed hypothesis: the session's other open songs (first run) -> refuted by the fresh sessions. The first session's other three exports, made after other songs were opened, also match at no N in 0..60000.

Acceptance evidence:
- AC1: docs/ui_spec.md section 12 (names, ids 306/307, enable sub_42BC83 0x42BFB4/0x42BFC3, handler sub_42C204, alert 500128003 and its buttons, modified mark sub_42590B, no undo), docs/song_load.md section 14 (msgs 28/29 at 0x100123C7/0x1001243A and every step with addresses), docs/initial_investigation.md 5.2 (msgs 28, 29; 30/31 now named: Copy Touched Controls), docs/original_bugs.md OB-26, docs/test_machine.md (screenshots). All [C] but the stated [G] cause of the phase mismatch.
- AC2: crates/orb-ui/tests/edit_init.rs: the 4 saves equal the original's byte for byte except OB-26 (GLOB 483..511, USRI 544..711, stale in the original) and COMP 3..7 (original writes zeros, orb keeps as read: task-307, decided 2026-11-25, keep as read). crates/orb-engine/tests/edit_init.rs: tracks equal node for node.
- AC3: Engine::init_* run the original's refresh chase at the position (unit test initialize_song_applies_the_pattern_values_at_once, seq refresh_chases_in_song_mode_only); the original's export after Initialize Loop equals orb's render of the saved file at N 32752 (EQUAL); the other exports differ only through the 303 phase history (song_load 14).
- AC4: cargo test --workspace 1174 passed, 0 failed, 6 ignored; cargo clippy --all-targets -D warnings (macOS) clean; cargo xwin clippy x86_64-pc-windows-msvc --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.
- AC5 open: the user's check. Dev app: target/OpenRB-338.app (version 2026.1125.1344).
- Plugin: first shown disabled by section 20's rule; the user decided on 2026-11-25 to hide them in the plugin (see below).

Change from the user (2026-11-25, via the supervisor): both commands undoable, as orb's addition.
- Design: the engine keeps the snapshot. Command::InitSong/InitLoop carry a Box<Pool> made on the UI thread (Pool::new reserves the whole capacity); before the command the callback copies every list into it (Engine::keep_undo, Pool::copy_from: no allocation) and keeps it, the snapshot it replaces goes back as Garbage::Pool. Edit > Undo sends Command::UndoInit: Engine::undo_init swaps the pool in one move (Sequencer::swap_pool), points each track at the current tick and runs the song-mode refresh chase as the command does; the post-command lists go back through the garbage queue. room_for counts the three commands as box commands.
- UI: Undo gets a tracks flag (row usize::MAX, so no pattern editor keeps it); 'Undo Initialize Loop' / 'Undo Initialize Song'; the status 'Initialize Loop undone'; the song stays modified, as after a pattern Undo (doc.modified set). A pattern command replaces it and the reverse; open / New forget it. The menu bar now clears the Edit menu's item places each frame (they were stale while the menu was closed, which made a test click a stale place).
- Cost on 99000 events (release, this Mac, cargo test --release -p orb-engine --lib -- --ignored init_cost --nocapture): snapshot copy 65.6 us, Undo (swap + pointers + chase) 97.2 us; the Initialize Loop itself 2.80 ms, Initialize Song's delete 0.15 ms.
- Tests: crates/orb-engine/tests/edit_init.rs undo_puts_the_tracks_back (4 cases, lists equal), undo_while_playing (after Undo while playing the lists and every device's live parameters equal those of an engine that never had the command); crates/orb-ui/tests/edit_init.rs undo_gives_back_the_song_before_the_command (4 cases via the Edit menu: Undo disabled before, enabled after with the command's name, save byte-identical to the one before), pattern_commands_and_initialize_share_the_one_undo, undo_while_playing.
- Docs: ui_spec 12 (the original has no undo), 18 (Undo row, Initialize row, the paragraph 'Undo of the Initialize commands' marked orb's), song_load 14.
- Checks: cargo test --workspace 1179 passed, 0 failed, 6 ignored; clippy -D warnings macOS and cargo xwin Windows clean; nightly fmt --check clean. Dev app rebuilt: target/OpenRB-338.app 2026.1125.1358.

The user's answers (2026-11-25, via the supervisor): (1) the plugin's Edit menu hides the two Initialize items and the separator after them (menu.rs, the plugin flag; ui_spec 18 and 20 name the exception; crates/orb-plugin/tests/editor.rs the_edit_menu_has_the_standalones_items_without_shortcuts asserts they are absent). (2) task-307 decided: option 1, keep the bytes as read; saves need not match the original byte for byte. The edit_init UI test keeps its three masks; docs (song_load 14, OB-26, the test's comment) say the choice is decided, task-307 Done in this commit.

AC5: the user checked the commands on 2026-11-25 in the dev app (2026.1125.1430) and in the plugin bundle built from b3894ef.

Review round 1 (2026-11-25, sent back with six defects, all fixed in the one commit):
1. The playback criterion was overclaimed: removed and re-added as AC 6 with what is shown (tracks node for node 4 of 4, one export equal); the six non-matching exports (fresh sessions N 0..30000, first session N 0..60000) are recorded as open in docs/song_load.md section 14 with the cause [G]; follow-up task-308 (spike, engine/re, m-5, @agent-complex); the commit's Result restated.
2. crates/orb-player/tests/no_alloc.rs the_callback_does_not_allocate now sends InitLoop, InitSong (pools made unarmed), UndoInit and a second UndoInit, and counts the garbage handed back (the replaced snapshot, the lists the Undo let go of, none for the empty Undo): 0 allocator events.
3. Subject now names player: 'engine,ui,player: ...'.
4. OB-26: the chunk is 512 (GLOB) / 712 (USRI) bytes long, so 0..511 / 0..711 go to the file.
5. ui_spec 12: 'no undo' now [C] with the evidence (the handler sends only the message, 0x42C592..0x42C5F1; the DLL frees the removed nodes, sub_10014910 -> sub_10014484; no Undo item in SMNU 128 and no 'Undo' string in the exe).
6. song_load 14 states the range searched per export.
Checks: cargo test --workspace 1179 passed, 0 failed, 6 ignored; clippy macOS and cargo xwin Windows clean; nightly fmt --check clean.

Review round 2 (2026-11-25): the export and fit scripts (scratchpad fresh.sh, fitn.sh, fitn2.sh) are committed as tests/songs/edit_init/export.sh (rcs_client.py, no scratchpad paths) and fit.sh; the README holds the reference WAVs' SHA-256 and the N searched for each; task-308 points at them. Check of the committed export.sh on the test machine (all redc-win-run permits, my ReBirth PID 20708 only): Initialize Loop on init_loop.rbs, WAV 2a0622be..., fit.sh matched at N 6016 (open to export 9 s).

Correction to the Review round 2 note: it is complete in the file (it ends '... fit.sh matched at N 6016 (open to export 9 s).'); a viewer that cuts long lines showed it cut. In short: the committed export.sh, run once on the test machine for Initialize Loop on init_loop.rbs, gave a WAV that fit.sh matched at N 6016.
Review round 3 (2026-11-25): docs/song_load.md section 14 now has the fresh-session match (export.sh as committed, N 6016, about 9 s from open to export, [C] for the measured match; 9 s x 44100 / 64 is about 6200, which supports the idle-time [G]) and counts two equal exports and six open ones; the commit's Result says the same. The default rcs_client.py path in export.sh is kept (the project's convention, tools/rb_export.py, docs/test_machine.md).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Reverse-engineered and implemented the original's Edit > Initialize Loop from Pattern Mode (command 306, song msg 29) and Initialize Song from Pattern Mode (307, alert then msg 28), enabled in song mode only; Song asks with the original's text, Initialize / Cancel. Song: all events but tick 0 removed, tick 0 takes the pattern-mode copy. Loop: the state at the loop end inserted there, [start, end) cleared except tick 0, the pattern-mode copy inserted at the loop start; then a chase at the position. orb: Engine::init_song/init_loop, seq clear_tracks/clear_range/refresh, slot 8 SongDevice::insert_pattern, Command::InitSong/InitLoop, the Edit items and the question; the plugin's Edit menu does not have them (the user's decision, 2026-11-25). orb's addition (user, 2026-11-25): a one-level Undo of both (engine-kept snapshot, one swap; 0.07 ms snapshot, 0.1 ms Undo on 99000 events; no allocation in the callback). Test machine: tests/songs/edit_init (2 inputs, 4 original saves); tracks equal node for node 4 of 4; saves equal byte for byte except OB-26 stale bytes and the COMP bytes the original zeroes, kept as read (task-307, decided 2026-11-25); one export equal (N 32752), six exports matching at no N open in task-308. Tests: workspace 1179 pass; clippy macOS and Windows, fmt clean. Checked by the user on 2026-11-25; review round 1 fixed. Branch worktree-agent-aa9a478c0a04f2813, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
