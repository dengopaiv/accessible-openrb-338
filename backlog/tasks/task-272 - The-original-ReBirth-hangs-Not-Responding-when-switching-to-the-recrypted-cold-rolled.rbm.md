---
id: TASK-272
title: >-
  The original ReBirth hangs ("Not Responding") when switching to the recrypted
  cold-rolled.rbm
status: Done
assignee:
  - '@agent-complex'
created_date: '2026-11-21T12:56:00Z'
updated_date: '2026-11-21T17:53:00Z'
labels:
  - rbm
  - re
  - testmachine
milestone: m-5
dependencies: []
references:
  - docs/rbm.md
  - docs/test_machine.md
  - crates/rbm
  - tools/rbm.py
priority: medium
type: bug
ordinal: 258000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Reported by the user on 2026-11-21. On the Windows test machine, the original ReBirth 2.01 hangs in "Not Responding" when the user switches to the mod `cold-rolled.rbm` from their collection `~/rb-mods/mods`. Those are mods recrypted by `orb mod recrypt` (docs/rbm.md section 12).

Recrypted mods are meant to stay loadable by the original:
- docs/rbm.md 12.3 records the four shipped mods, recrypted, opening and exporting correctly in the original.
- crates/rbm/tests check that every file and slot sample of all third-party recrypted mods equals the original's.
So a hang means either orb's recrypted form of this mod is something the original can't handle, or the original has a defect this mod triggers.

**What to find out.**
1. **Reproduce on the test machine** with the original and the recrypted cold-rolled.rbm (docs/test_machine.md; take the machine's permits and touch only your own files and processes, since the machine is shared). Is the hang real (CPU-bound loop, or blocked), or a very slow load? Time it.
2. **Compare with the unrecrypted original of cold-rolled.rbm** if it can be found:
   - the collection's sources are credited in ~/rb-mods/README.md (audiovst.mnx2010.nl/prop and github.com/peffre/ReBirth-RB-338);
   - the user's earlier collection notes are in docs/rbm.md section 11.
   Does the original form hang too? If it does, it's a defect of the original: document it in docs/original_bugs.md and in docs/rbm.md, and note whether orb handles that mod.
3. **If only the recrypted form hangs:** find where in Rebirth.exe / ReBirth.dll it loops or blocks, and why. Use a debugger on the test machine, or IDA on copies of the user's databases (/Users/pingw33n/dev/openrb-338/ida/*.i64, never saved). Look at:
   - the mod reader's chunk walk (docs/rbm.md 12.3 notes that chunk count affects load time);
   - the key window;
   - this mod's size, chunk structure and sample count against the other 319.
4. **Survey the collection:** find whether other recrypted mods in ~/rb-mods/mods hang or load slowly in the original. A script on the test machine timing each mod switch would do; batch it if it's long.
5. **If orb's recrypt output is at fault:** fix `orb mod recrypt` (crates/rbm). Re-recrypting must stay byte-exact for the mods that work (pinned hashes in docs/test_machine.md 7 and tests/refs/thirdparty_mods.json), or the change must be justified and the manifests regenerated. Then say what the user must do with their collection (re-recrypt, and republish the rb-mods repository) as a follow-up task for the user.

Every fact about the original goes into docs/ with addresses, [C] or [G]. No test or tool may need the original's files: use ORB_CD_ISO, and skip without it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The hang is reproduced on the test machine and characterized (loop or block, or slow load, with timing), with the evidence in the notes
- [ ] #2 The original (unrecrypted) form of cold-rolled.rbm is tested in the original if it can be found, or the notes say why not
- [ ] #3 The cause is found with addresses and documented in docs/rbm.md (and docs/original_bugs.md if it is the original's defect)
- [ ] #4 The collection is surveyed for other mods that hang or load slowly in the original; the result is in the notes
- [ ] #5 If orb's recrypt is at fault it is fixed with tests and the working mods' outputs stay byte-exact (or the change is justified and the manifests regenerated), and a follow-up task tells the user what to do with the collection
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Get the unrecrypted cold-rolled.rbm (peffre/ReBirth-RB-338) and check that orb's recrypt of it equals the collection file.
2. On the test machine (all permits): a copy of the installation in C:\re\rb\t272\app with only the mod under test in its Mods; start it, switch to the mod through the Mods menu, time it, watch CPU time and 'Not Responding', and sample the threads' EIP and stack with a ctypes script (no debugger on the machine). Same with the original form. Save and restore the user's preferences file.
3. From the sampled addresses, read the code in copies of the IDA databases (never saved) and find the loop and why this mod triggers it; check the hypothesis on the mod's bytes.
4. Survey the 320 mods: a static check for the trigger across the collection, plus timed switches on the test machine for a batch (all if feasible).
5. Document in docs/rbm.md (and docs/original_bugs.md if it is the original's defect) with addresses; fix orb mod recrypt only if it is at fault, with tests and the manifests; follow-up task for the user if the collection must change.
6. Clean the test machine, release the permits, one commit.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-a0c4c3459d2b20a74

Original form: github.com/peffre/ReBirth-RB-338 'ReBirth Mods/cold-rolled.rbm', 2,944,094 bytes, SHA-256 0ec4e2deaff91857474b8631e8a54083aac314bb87e014a2ba14fcc36e936b9d. orb mod recrypt --iso (current main 0164080) of it gives 2,945,126 bytes, SHA-256 bd01adc6...bb01 = ~/rb-mods/mods/cold-rolled.rbm byte for byte (104 -> 105 chunks). Structure (rbm.py info): HEAD, 103 EMBF (pad + 102 files: 59 JPEGs, Demo.rbs, startup.aif, 41 slot AIFFs; TR808N1..N4 and one more slot built in), INFO; nothing unusual: every AIFF is COMM 18 + SSND with sizes consistent with the file, every JPEG baseline JFIF ending FF D9.
Test machine 2026-11-21: the CD is now drive D: (not E:); the installation's Mods holds 9 files put there by the user (cold-rolled, ReBirthday, AloNaNbox 2.0, TurnMod, mellow and the four recrypted shipped mods); not touched.

Reproduction attempts, 2026-11-21 15:39-15:57 (all 6 permits; the user's prefs file saved first, SHA-256 6309526e... = prf_before.prf, restored before each start and at the end):
- Tools (scratch, uploaded to C:\re\rb\t272): sampler.py (ctypes, 64-bit Python against the WOW64 process: per-thread CPU times, EIP and the code addresses on the stack; no debugger is installed on the machine) and winlist.py (the process's windows: class, title, visible, enabled, owner, IsHungAppWindow). The app was driven with rcs input (Mods menu by keys, Mod Selector by mouse).
- App copy C:\re\rb\t272\app (robocopy of the installation without Mods and the ISO), Mods = only the recrypted cold-rolled.rbm: Mods > Cold-Rolled from Standard switched in under 1 s (Cold-Rolled skin drawn; main thread back in GetMessage, win32u+11DC; process CPU 3.4 -> 3.7 s over 20 s; tasklist Status Running). Again from Standard while playing: same.
- The same copy with the installation's 9 mods (the user's set: AloNaNbox 2.0, MSM2, Metallicon, Orbit2, PBE2, ReBirthday, TurnMod, cold-rolled, mellow): Mods menu 3oNaNbox -> Cold-Rolled -> southpark -> Cold-Rolled -> Metallicon+ -> ... -> DJ TURN's MOD -> Cold-Rolled -> Standard -> Cold-Rolled (18 switches): every one Running, about 0.3-0.6 s CPU each, no hang. Removing the file, Update Mod List, putting it back, Update Mod List, switching: fine.
- Mod Selector (Ctrl+M) -> Cold-Rolled: splash and text shown, OK switches. Demo Song... saved the embedded Demo.rbs (22,520 bytes); opening that song switched the mod (song-info box, then the Cold-Rolled panel); playing it and switching Standard <-> Cold-Rolled while it plays (4 switches): fine.
- The installation's own Rebirth.exe (C:\ReBirth RB-338, at the user's request, 15:55, PID 10296): Mods > Cold-Rolled from Standard, and Mod Selector > Cold-Rolled > OK: both fine (Status Running, skin drawn).
So the hang is NOT reproduced with cold-rolled.rbm by any of these routes, in the copy or in the installation.
- Found instead: picking '3oNaNbox 2.0' (the installation's AloNaNbox 2.0.rbm, 1,966,580 bytes; not an orb recrypt: orb refuses AloNaNbox, docs/rbm.md 12.5) in the Mod Selector shows the app's own message box 'The Mod file could not be loaded.' BEHIND the Mod Selector. winlist: the box ('ReBirth RB-338', class 'Smuggler Modal Dialog Window') visible and enabled, owned by the frame, under the selector in the z-order; selector and frame disabled. Nothing on screen reacts and the selector shows no change: it looks hung, while the main thread runs the box's modal loop (sampled for 50 s at the same ESP: ntdll wait under sub_4809FB <- sub_498496 <- sub_475D5C <- a catch funclet of sub_41A0C5 at 0x41A217..; IsHungAppWindow 0, tasklist Running, CPU flat). Brought forward with SetWindowPos it reads 'The Mod file could not be loaded.'; Enter closes it and the selector goes back to Standard ReBirth. Through the Mods menu the same box appears in front of the panel.

Stopped on 2026-11-21 at about 16:05, at the user's request (through the supervisor): the hang did not reproduce, and the user will retry by hand. The task stays To Do: no criterion is claimed and there is no final summary.

State left behind:
- Test machine: every process I started is stopped (my app copy and the installation's Rebirth.exe that I started at 15:55; tasklist shows no Rebirth.exe). The VirtualStore prefs file is restored and checked: SHA-256 6309526e1574414dd3c093f97fedb5e7978428c789db94099c3af81ee8cbd51e. C:\re\rb\t272 is deleted, and the 6 permits of redc-win-run are released. Nothing in C:\ReBirth RB-338 was written; its Mods still holds the user's 9 files. The machine's other rcs process (win32_remote32.exe, started 2026-11-11) is not mine and was left alone.
- IDA: a scratch copy of Rebirth.exe.i64, opened headless and closed with save=false.

The mod under test is the recrypted ~/rb-mods/mods/cold-rolled.rbm: 2,945,126 bytes, SHA-256 bd01adc64cff82b5f29932d84ece426b3180715e01523f8b9c3cedbd61e1bb01. It is identical to the one in C:\ReBirth RB-338\Mods. Its original is from peffre/ReBirth-RB-338 'ReBirth Mods/cold-rolled.rbm': 2,944,094 bytes, SHA-256 0ec4e2de...6b9d. That original was not run in the app (criterion 2 is open).

Steps to repeat (docs/test_machine.md 2-3; all permits held):
1. rcs exec cmd.exe /c start '' /D 'C:\ReBirth RB-338' 'C:\ReBirth RB-338\Rebirth.exe', then rcs input --window 'ReBirth RB-338' key:enter for the MIDI box.
2. Mods menu: Alt+M, then Up 8 times from the wrap, which lands on Cold-Rolled with the 9-mod menu (Select Mod, Update Mod List, Standard, 3oNaNbox 2.0, Cold-Rolled, southpark, Metallicon+, MSM, Orbit, PBE, ReBirthday, DJ TURN's MOD). Then Enter. The switch took under 1 s, with the Cold-Rolled skin drawn.
3. Mod Selector: Ctrl+M, click the combobox arrow (824,310 when the frame is at about 164,157), then click Cold-Rolled (660,341), then OK (794,597). This worked too.
Watching: tasklist /V /FI 'IMAGENAME eq Rebirth.exe' /FO LIST gives Status and CPU Time. A ctypes thread sampler run through rcs exec gave EIP and stack (Wow64GetThreadContext + ReadProcessMemory), and a window lister gave visibility, enabled state, owner and IsHungAppWindow (EnumWindows). These scratch scripts are not committed.

Lead for the retry: if the screen 'freezes' after picking a mod in the Mod Selector, look for a message box hidden behind the selector ('The Mod file could not be loaded.', seen with 3oNaNbox 2.0 = AloNaNbox 2.0.rbm). Windows does not mark that case Not Responding, though. A real 'Not Responding' title, if it is seen again, means the main thread stopped pumping messages. Then a thread sample (or a dump from Task Manager, Create dump file) during the hang would show where.

2026-11-21: the user retried by hand on the test machine and could not reproduce the hang either; closed as not reproducible at the user's word.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Closed as not reproducible on 2026-11-21, at the user's word.

- **The agent (15:39-16:05):** no hang switching to the recrypted cold-rolled.rbm (SHA-256 bd01adc6...e1bb01, the same file as the installation's).
  - Tried both the Mods menu and the Mod Selector, from Standard and from each of the user's 9 mods (18 switches), after Update Mod List, and while a song played.
  - Ran it in a copy of the app and in the installed Rebirth.exe.
  - Every switch took under 1 s.
- **The user** then retried by hand on the test machine and couldn't reproduce it either.
- **Recrypt is fine:** `orb mod recrypt` of the original cold-rolled.rbm (github.com/peffre/ReBirth-RB-338) gives that recrypted file byte for byte.
- **A possible look-alike, noted for next time:** picking the installation's AloNaNbox 2.0 in the Mod Selector raises "The Mod file could not be loaded." behind the selector, so the app looks frozen while it's really waiting in a modal box.
- **If the hang comes back:** note the route (menu or selector), the mod switched from and whether a song was open or playing, and take a dump (Task Manager, Create dump file). The repeat steps are in the notes.

**Criteria:** none are met, since nothing could be reproduced; the task is closed without a cause. No code or docs changed, and the test machine was left clean.
<!-- SECTION:FINAL_SUMMARY:END -->
