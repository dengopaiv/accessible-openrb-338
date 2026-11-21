---
id: TASK-258
title: >-
  Plugin note mode: changing a REAPER send while playing leaves the 303 sliding
  until the mode is switched
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T06:27:00Z'
updated_date: '2026-11-21T10:36:00Z'
labels:
  - player
milestone: m-7
dependencies:
  - TASK-244
references:
  - crates/orb-plugin/src/core.rs
  - docs/plugin.md
  - docs/plugin_usage.md
priority: high
type: bug
ordinal: 252000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21 while testing task-256's branch in REAPER (it may predate task-256; find out). Setup: track 1 holds the OpenRB-338 instrument; four tracks hold MIDI clips made by dragging the plugin's patterns out (one per device) and send MIDI to track 1. While REAPER plays, adding or removing a send on the 303 track makes the 303 sound wrong: as if a note is stuck and every following note slides. It stays that way until the plugin's mode is switched (Note to Latch or Gate and back). The user's screen recording shows it (audio matters).

Likely cause to check first: removing a MIDI send mid-note drops the note-off, so the 303 voice keeps a held note and every later note overlaps it, which the 303 plays as a slide (legato). Adding a send mid-note gives a note-off without its note-on, or a note-on into an already-held key. Check what REAPER sends when routing changes (all-notes-off CC123, CC120, nothing), what nice-plug delivers to the plugin, and how note mode tracks held notes per device (a held-note set or counter that can go out of step). Switching modes evidently resets that state; find what it resets.

The fix must make the plugin recover on its own: e.g. honour CC123/CC120 and the host's transport stop/reset, and not let a stray or missing note-off leave the 303 sliding forever (decide with care what a real 303 in note mode should do with overlapping notes: slide only while the previous key is really held). If the right behavior is a choice for the user, create a `Decide:` task. Reproduce in a test with the plugin core (a note-on with no note-off, then later notes; a note-off with no note-on), not only in REAPER. The user checks the fix in REAPER before the agent review.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The cause is found and written in the task notes, with whether it happens on main before task-256
- [x] #2 A test reproduces the stuck slide from a missing or stray note-off and passes after the fix
- [x] #3 The plugin recovers without a mode switch: on all-notes-off or all-sound-off, on transport stop, and from the stuck state the test reproduces
- [x] #4 docs/plugin.md describes how note mode treats held, missing and stray note-offs
- [x] #5 The user confirms in REAPER that adding or removing a send while playing no longer leaves the 303 sliding
- [x] #6 A case in task-244's REAPER harness (a note-on whose note-off never arrives, then more 303 notes in note mode) passes after the fix
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Cause: note mode's 303 keeps held notes as instances (Tb303::key_on/key_off, task-200, f50af89); a lost note-off leaves an instance held for good (a replay of that key adds a second instance, its note-off removes the oldest), so every later note-on is legato (slide) and every release slides back to the stuck key. Only slot 3 (Play) and slot 6 (Stop) clear it: what the mode switch sends. The plugin ignores CC123/CC120 and the host's stop. Check it predates task-256 (the code is task-200's; task-256 did not touch it).
2. Engine: one held entry per key (a note-on of a held key re-presses it: the old entry goes, the press is legato/tie as now); Tb303::key_all_off (slide clear, note-off, held forgotten when any is held); ChannelDevice::live_notes_off, Engine::live_notes_off (ignored while the song plays); Command::NotesOff(Device) in orb-player.
3. Plugin core: in note mode, CC123 or CC120 on channel 1..4 sends NotesOff for that device (then the message goes on to the Standard Mapping as before, which maps neither); the host's playing->stopped edge (per block, in sync) sends NotesOff for the four devices.
4. Tests: engine unit tests (key re-press, all-off); plugin tests in tests/note_mode.rs: the stuck slide from a missing note-off (fails before), recovery by a replay of the key, by CC123/CC120 (bit-equal to the note-offs), by the host's stop; a stray note-off changes nothing (bit-equal).
5. REAPER case (task-244 harness): 303_slide drag-out with an orphan note-on (no note-off) of a key the clip plays later, the item extended past the clip; REAPER equals the in-process host bit for bit and the 303 is silent after the clip's last note-off before the item's end (stuck before the fix). The in-process events get REAPER's item-end CC123 the fixtures already write.
6. docs/plugin.md 9.1 (held, missing, stray note-offs, CC123/120, stop), 7.1 the case; full cargo test, clippy, nightly fmt, corpus via cargo test.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a4e5cc6fd49dd41fc

The user's screen recording (frames via an AVFoundation swift script, audio via afconvert, in the scratchpad): REAPER 7.86, track 1 the VST3 instrument in note mode, tracks 303 #1 B7, 303 #2 B1, 808 A4, 909 A1 with dragged-out clips, each sending MIDI All->All to track 1; the user deletes and re-adds the send of 303 #1 in its routing window while the 1-bar loop plays.

Cause (criterion 1). Note mode's 303 held one entry per note-on: Tb303::key_on pushed (note, accent) onto held[] even when the key was already held, Tb303::key_off removed the oldest entry of the key (crates/orb-engine/src/tb303.rs, task-200, f50af89, 2026-11-16). A removed send drops the note-off of the note in flight; that key stays held. Every later note-on then sees held_len > 0 and plays legato (slide flag set, no attack: the slide), and every release of the sounding note slides back to the stuck key, so the gate never closes. A replay of the stuck key (the looped clip) added a second entry and its note-off removed only one: the extra entry never went away. Nothing else cleared it: the plugin ignored CC 123/120 (only the Standard Mapping reads CCs, and maps neither) and the host's stop (note mode plays with the host stopped). The mode switch cleared it because, with the host playing, Note -> Latch/Gate makes sync() send Play (slot 3, Tb303::start sets held_len = 0) and back to note mode ensure_stopped() sends Stop (slot 6, Tb303::silence sets held_len = 0). A send added during a note gives a note-off of a key not held: key_off returns, harmless.
Predates task-256: the held-note code and the plugin's MIDI path are task-200's (f50af89) and task-218's; task-256 (82e8836) did not touch key_on/key_off/held or Engine::message (git show 82e8836, grep). The new tests failed on the pre-fix engine and core (below).

What REAPER sends: REAPER's MIDI was not observed live. Inferred from the render harness, not observed: REAPER seems to close a note its item left open at the item's end. With the pre-fix code the tb303_stuck_note render differed from the in-process host (which sends only the item-end CC 123, ignored pre-fix) from frame 242569 (item end 242550), up to 0.092; the pre-fix plugin reacts to a note-off of the held key there and not to the CC, so REAPER most likely sent such a note-off. A removed send would then lose exactly that note-off.

Fix. (1) A key is held once: key_on of a held key removes its old entry first (MIDI's model, and Gate Pattern Mode's Held::press already); legato as before (a tie for the same key). The drag-out never overlaps two notes of one pitch (clip.rs: a tie is one longer note; a last-step slide ends at the clip end), so this changes nothing the plugin writes; the tie unit test now presses the key again without a note-off between. (2) Tb303::key_all_off (slide clear, note_off, held forgotten; nothing when none held), ChannelDevice::live_notes_off, frame Engine::live_notes_off (ignored while the song plays), orb-player Command::NotesOff(Device). (3) Plugin core: in note mode CC 123 or CC 120 on channel 1..4 sends NotesOff for that device (the message then goes to the Standard Mapping as before); sync() sends NotesOff for the four devices on the first block whose host record stops after one that played (note mode, not the preview). CC 120 is a release, not a hard cut.

Tests. Engine (src/tb303/note_tests.rs): a_key_is_held_once, a_lost_note_off_heals_at_the_keys_next_note_off, all_off_lets_every_key_go, the tie test adapted. Plugin (tests/held_notes.rs): a_lost_note_off_keeps_the_303_sliding_until_the_key_is_let_go, all_notes_off_and_all_sound_off_let_the_held_notes_go (bit-equal to the note-offs; another channel's CC does not), the_hosts_stop_lets_the_held_notes_go, a_stray_note_off_changes_nothing (bit-equal). With tb303.rs and core.rs at main's (git checkout main -- those two, test files kept): 3 of 4 plugin tests fail (held_notes.rs:113 the key's next note-off, :165 the host's stop, :134 CC 123 not as the note-offs); the stray test passes before and after.
REAPER case tb303_stuck_note (tests/reaper/cases.rs, criterion 6): the 303_slide drag-out, two loops at the clip's times, plus a note-on of key 39 at tick 0 with no note-off (39 is played and let go later in each loop), the item 1.5 s longer than the clips; render 6 s. The in-process events now also carry the item-end CC 123 REAPER writes (the generator writes it from the events; the ten old fixtures are byte-identical). Check: REAPER = in-process bit for bit, and the master from 0.5 s after the clips to the item's end (frames 198450..242550) peaks below 1e-3: 0 in both formats. Pre-fix: the REAPER test failed on that case (above).
Run: ORB_REAPER=/Applications/REAPER.app/Contents/MacOS/REAPER cargo test -p orb-plugin --test reaper -- --nocapture (2 pass, 69.7 s after the rebase onto 624c438, with task-260's load-log changes).
Workspace: cargo test 984 pass, 0 fail (incl. corpus_golden: pattern playback unchanged, note mode only); clippy --all-targets -D warnings clean; cargo +nightly fmt --all --check clean.
Follow-up: task-269 (Gate Pattern Mode ignores CC 123/120 for its held gate notes).
Build for the user's check (criterion 5): cargo xtask bundle orb-plugin --release puts OpenRB-338.vst3 and OpenRB-338.clap in target/bundled/; copy them to ~/Library/Audio/Plug-Ins/VST3/ and ~/Library/Audio/Plug-Ins/CLAP/ (the user's folders, not /Library), restart REAPER.

User check (criterion 5), 2026-11-21, relayed by the supervisor: the user installed the 6454565 bundles (built in this worktree) and checked in REAPER: "it's better", the sliding is gone. One case remains: deleting the send while a 303 note is held. The plugin then gets no more MIDI from that source, so the held key sounds on (standard MIDI behavior) until the host's stop or an all-notes-off. Options offered: A keep as is, B clear held notes on a host loop wrap or position jump, C a timeout. The user decided A (2026-11-21), recorded as task-270 (Decide: a note whose MIDI source is removed mid-note, Done). Documented in docs/plugin.md 9.1 and docs/plugin_usage.md 4. REAPER's wording comes from the strings of /Applications/REAPER.app/Contents/MacOS/REAPER (7.86): the action "Send all-notes-off and all-sounds-off to all MIDI outputs/plug-ins", and the MIDI reset preferences "Reset on:" with "On stop (default, recommended)" or "Reset on playback start", plus "Don't send note-offs or pitch reset messages on stop/reset". The preferences page that holds them is not confirmed, so the docs do not name one. cargo test -p orb-ui: 358 pass (plugin_usage.md is the Help page).

Review round 1 (send back, no code defects): rebased onto main 3c8fa04 (the commit now touches only task-258's files, task-269 and task-270); docs/plugin.md 7.1 states REAPER's item-end note-off as an inference from the render difference; Decide task task-270 created (A/B/C, the user's A of 2026-11-21, Done) and referenced from docs/plugin.md 9.1 and the commit; the local video path and the machine's audio outputs removed from the task text; docs/plugin.md 9.1 gives the reason CC 120 is a release, not a hard cut; the ragged lines in 7 reflowed. No code change; cargo test -p orb-ui after the docs.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Cause: note mode's 303 held a key once per note-on, and each note-off released only one entry (Tb303::key_on/key_off, task-200). A note-off lost when a REAPER send was removed mid-note therefore left the key held for good: every later note slid from it, and every release slid back to it. Only Play or Stop of the song cleared it, which is what a mode switch sends while the host plays. The plugin ignored CC 123/120 and the host's stop. The bug predates task-256: it is task-200's code, which 82e8836 did not touch.
Fix: a key is held once (pressing a held key again replaces its entry), so a lost note-off heals at the key's next note-off. In note mode, CC 123 and CC 120 on channels 1..4 and the host's stop let go of every held key (Command::NotesOff -> Engine::live_notes_off -> Tb303::key_all_off), with the same output, bit for bit, as the note-offs. A stray note-off changes nothing.
Docs: docs/plugin.md 9.1 and 7.1, docs/plugin_usage.md 4.
User check, 2026-11-21: the sliding is gone in REAPER. A note held while its send is deleted sounds on until the stop or an all-notes-off, which is standard MIDI behavior. The user decided to keep that (option A, no loop/jump clearing, no timeout); it is documented with REAPER's all-notes-off action and its reset preferences.
Tests: the engine note tests (3 new; the tie test changed to the new model) and crates/orb-plugin/tests/held_notes.rs (4 tests; 3 of them fail on the pre-fix code). The REAPER case tb303_stuck_note fails on the pre-fix code and passes after the fix: REAPER equals the in-process host bit for bit, and the tail is silent. cargo test: 984 pass, corpus unchanged. The REAPER test passes, orb-ui 358 pass, clippy and nightly fmt are clean.
Branch worktree-agent-a4e5cc6fd49dd41fc, one commit on main 624c438. Follow-up: task-269 (Gate Pattern Mode and CC 123/120).
<!-- SECTION:FINAL_SUMMARY:END -->
