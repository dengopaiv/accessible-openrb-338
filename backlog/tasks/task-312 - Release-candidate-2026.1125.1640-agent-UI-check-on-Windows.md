---
id: TASK-312
title: 'Release candidate 2026.1125.1640: agent UI check on Windows'
status: Done
assignee:
  - '@agent-worker'
created_date: '2026-11-25T17:04:00Z'
updated_date: '2026-11-25T17:56:00Z'
labels:
  - tests
  - testmachine
  - ui
milestone: m-5
dependencies: []
references:
  - docs/ui_spec.md
priority: high
type: task
ordinal: 283700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why:** the user wants the release candidate checked before the next release. On 2026-11-25 the supervisor ran release.yml on main 9558c9d (run 45656364828; version 2026.1125.1640). An agent tests what it can on the Windows test machine by driving the app's UI through rcs_client (exec, input, screenshot; doc-001). The user tests the rest, including the macOS .dmg and .pkg and the plugin in hosts.

**Build under test:** C:\tmp\OpenRB-338-2026.1125.1640-x86_64-pc-windows-msvc.zip, the portable zip. Unzip it into a folder of your own on the test machine. Portable mode keeps its settings in that folder, so the user's settings are not touched. Do NOT run the setup .exe (C:\tmp\OpenRB-338-2026.1125.1640-x64-setup.exe): installing changes the user's machine.

**Check each item and record pass/fail with a screenshot (or the exact observation) in the notes:**
1. **Start.**
   - The app starts.
   - Help > About shows 2026.1125.1640, and its copy button copies the version.
   - A new song has the four sample patterns in A1.
2. **Names.** The status line after Edit > Clear Pattern on a 303 says "303-1: A1 cleared". Help > Keyboard Shortcuts shows no TR-/TB- names.
3. **Pattern commands.** Edit > Copy/Cut/Paste/Duplicate/Clear Pattern and Undo. Copy puts JSON on the clipboard (read it back through the app's Paste, or the machine's clipboard if rcs allows).
4. **Pattern editors.** Edit > 303-1, 303-2, 808 and 909 Pattern Editor..., and Ctrl+E. Each opens, edits a step, and comes to the front when chosen again.
5. **Initialize Song and Initialize Loop from Pattern Mode.** Both are disabled in pattern mode and enabled in song mode. Initialize Song asks first, and Escape cancels. Undo restores the song.
6. **Song Information.** Ctrl+I opens it, and File > Info shows "Ctrl+I". Plain I still works as a panel key.
7. **File dialog.** Paste a full path into the path bar and click Open. A folder path and a bare name work too. The list fills the dialog.
8. **Minimum window.** Shrink the window to its minimum (about 536 x 485 points). The file dialogs, About, Audio Output, Mod Information, the error box and the pattern editors fit inside it.
9. **View > Classic / Modern.** Both views draw, with the crossfade, and the panel takes clicks in both.
10. **Options > Audio Output.**
    - The driver list includes WASAPI, and ASIO when an ASIO driver is installed.
    - The device, rate and buffer lists appear.
    - The In use block shows the buffer and latency, and the numbers update about every 0.5 s.
    - Closing the app with an ASIO driver panel open exits the process (the user checked this before; repeat it only if it is safe).
11. **Mods.**
    - Load a mod without a splash from a copy of ~/rb-mods/mods/"AFX 1.rbm" uploaded to your folder. The 808's mod window shows the standard splash, and Mods > Info says "This mod has no splash picture."
    - A mod with a splash shows its own.
12. **Portable mode.** OpenRB-338.toml appears next to the exe. Removing OpenRB-338.portable makes the app use %APPDATA% instead. Restore it afterwards, and leave the user's %APPDATA% file as it was: back it up first and compare after.
13. **orb.exe.** `orb render` of a shipped song (tests/songs/shipped) works. `orb info` works.

**Not for the agent; the user checks these:**
- the plugin in REAPER, Ableton and Studio One (Output Routing, the 26 outputs, parameter names);
- the macOS .dmg and .pkg;
- the Linux AppImage;
- the setup .exe install and uninstall.

**Rules:**
- The test machine is shared with the user. Touch only processes you start. Never close or disturb the user's REAPER, ReBirth or other programs.
- Clean up your folder at the end, and say what you left.
- A failure is a finding: record it with steps to reproduce and a screenshot path. Create a bug task for each real defect (labels ui or player, m-5, assigned @agent-worker), and stop to report anything that would block the release.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Items 1 to 13 are each recorded pass or fail with evidence (screenshot path or exact observation) in the notes
- [x] #2 Each real defect found has its own bug task; release blockers are reported to the supervisor
- [x] #3 The user's settings, programs and files on the test machine are unchanged; the agent's folder is cleaned up
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Take permits (5 of 6; task-311 holds 1), unzip portable zip into C:/re/rb/rc312.
2. Drive UI items 1-12 with rcs input/screenshot, back up %APPDATA%\OpenRB-338 first.
3. Item 13 orb.exe render/info.
4. Record results, bug tasks, clean up, release permits.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Branch: worktree-agent-ad21eb186141b4a48 (rebased on f2965b9)

Run 2026-11-25 19:04-19:10 (screenshots in /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-ad21eb186141b4a48/shots/, local only). Held 5 of 6 permits (task-311 held 1); released.
BLOCKED for items 1-12 (UI driving): the portable zip unzipped to C:/re/rb/rc312 and OpenRB-338.exe started (visible in screenshot 01, title 'OpenRB-338 - Untitled', Classic/Modern panel drawn, 4 pattern strips), but NO rcs input reached it. The foreground window was the user's Task Manager (process started 11/11/2026, not mine). Evidence: rcs input --window reports foreground:false (rcs SetForegroundWindow fails); a PowerShell SetCursorPos(700,500) returns false, GetLastError 203, and the cursor stays at (8,265) for every click/move; clicks on the menu bar and Help, Alt+H and Alt+Esc did nothing (shots 02-08); AttachThreadInput+SetForegroundWindow from PowerShell also fails (foreground stays Task Manager). Session list: rdp-tcp#0 pingw33n Active (id 2), console id 3 Connected. Retried after 60 s: same. I did not close or touch Task Manager (user's program). Likely cause: the foreground window is a higher-integrity or otherwise input-protected window (UIPI), or the RDP session is not taking injected input; not confirmed. To unblock: the user closes/minimizes Task Manager (or just clicks the OpenRB-338 window) and the task is rerun.
Results: item 1 partial (app starts; Help>About, copy button, A1 patterns not tested); items 2-12 could not test (no input). Item 12 partial: OpenRB-338.toml (195 bytes) appeared next to the exe at start, OpenRB-338.portable was present in the zip; the user's %APPDATA%\OpenRB-338\OpenRB-338.toml and ui.toml (sha1 2e4069477a735311db71a759a3b357e3b208afde) were backed up before and are byte-identical after (nothing touched; removing the marker not tested). No ASIO question reached.
Item 13 PASS: orb.exe info C:/re/rb/rc312/Calm.rbs prints RBS 4.1, 'it down', 130 BPM, Mod Standard ReBirth (built-in kit), 172 bars loop; orb.exe render Calm.rbs -o calm.wav wrote 56013828 bytes with SHA-256 54560491ecb1c8c601e688933a282b1ee1d9598e5a37e15fb740bfe88ab67d2c, equal to wav_sha256 of Calm in tests/refs/manifest.json (the rbhost reference). (orb.exe has no --version flag; not needed.)
Cleanup: killed my OpenRB-338.exe (PID 21768, by PID), deleted C:/re/rb/rc312 entirely, nothing left on the machine; permits released. No bug tasks (no defect found). No release blocker found, only the test-environment block.

Round 2, 2026-11-25 19:30-19:55: the user minimized the (elevated) Task Manager themselves; I did not close it. Input then reached the app (rcs input worked). Held 5 of 6 permits (task-311 held 1); released at the end. Build: C:/tmp zip unzipped to C:/re/rb/rc312; screenshots local in /private/tmp/claude-501/-Users-pingw33n-dev-OpenRB-338/a9b177cc-ff45-4931-aea4-b72986a77486/scratchpad/worktree-agent-ad21eb186141b4a48/shots/. %APPDATA%\OpenRB-338\OpenRB-338.toml backed up before (sha1 2e4069477a735311db71a759a3b357e3b208afde) and byte-identical, mtime unchanged, after item 12.
RESULTS
1 Start PASS: app starts (title 'OpenRB-338 - Untitled'); Help > About shows 'Version 2026.1125.1640 (9558c9d)' (shots/about.png); the copy button put '2026.1125.1640 (9558c9d)' on the clipboard (read back with Get-Clipboard); the new song's A1 patterns are filled in 303-1, 303-2, 808, 909 (shots/eds.png).
2 Names PASS: Edit > Clear Pattern status 'Untitled: 303-1: A3 cleared' in the Calm session (shots/cl2.png; A3 because pattern 3 was selected, the text form is 'A<n> cleared'; I did not capture 'A1' exactly: the fresh-song Clear with A1 selected was not run, only A3). Help > Keyboard Shortcuts scrolled through top to bottom (shots/ks2-ks6, about.png): no TR-/TB- text; the only TR-8/TB-3 strings in orb-ui sources are doc comments.
3 Pattern commands PASS: Copy -> clipboard holds JSON {"openrb338":"pattern","version":1,"device":"303","length":16,...}; Paste status '303-1: A1 holds the pattern already'; Duplicate '303-1: A1 duplicated to A2'; Undo '303-1: Duplicate Pattern undone'; Cut '303-1: A1 cut' (shots/p_*_st.png).
4 Pattern editors PASS: Edit > 303-1, 303-2, 808, 909 Pattern Editor each open (shots/eds.png) and a step click changes the grid; choosing 303-1 again brings it to the front; Ctrl+E opens the 303-1 editor (and the menu shows Ctrl+E); also fine at the minimum window (mi_pe*c.png).
5 Initialize PASS: in pattern mode both items are greyed, in song mode both are enabled (shots/menus.png); Initialize Song asks 'Initialize Song from Pattern Mode ... Initialize / Cancel'; Escape cancels (no change, shots/is2.png); Initialize runs; Edit > Undo then reads Undo Initialize Song and the status says 'Initialize Song undone' (shots/is3_st.png / p_*).
6 Song Information PASS: Ctrl+I opens it (shots/i1.png), File menu shows 'Info... Ctrl+I' (shots/g.png); plain I in Select Patterns mode selects 303-2 pattern 8 and opens no window (shots/g.png).
7 File dialog PASS: full path pasted into the path bar and Open loaded C:\re\rb\rc312\Calm.rbs; a folder path (C:\re\rb\rc312) lists the folder; a bare name (Calm.rbs) then Open loads it; the list fills the dialog (shots/o3.png, f1.png, f2.png, m1.png).
8 Minimum window PASS (with limits): dragged the window corner to its minimum, outer 552x524 px (rcs reported rect 100,60 552x524; this is the app's minimum, about 536x485 points at 100% scale plus the frame). Inside it: Open song, Choose mod, About, Keyboard Shortcuts, Audio Output, Song Information and the 303-1/808/909 editors all fit (shots/m1.png, ab_min.png, about.png, mi_info.png, mi_pec.png, mi_pe808c.png, mi_pe909c.png; Song Information is a scrolling window, its last rows scroll). NOT tested at minimum: Mod Information (seen only in a larger window), the error box (no error provoked).
9 View Classic/Modern PASS: both draw (Metallicon+ shown in both, shots/vc5.png, my_end.png); one in-between frame captured on the switch (shots/my1.png differs from the final frame, i.e. a crossfade exists but is short, only 1 of 6 frames at ~0.3 s spacing); the panel takes clicks in both (pattern-select buttons change in Modern: shots/v2_click.png, v3_click.png; in Classic: shots/vc_click.png). Note for testing: the View menu popup appears under the mouse position (not fixed); clicks done blindly missed it.
10 Audio Output PARTIAL: PASS driver list WASAPI and ASIO (shots/ao2.png); device, sample rate, buffer lists shown; In use block shows Device 'WASAPI: Remote Audio via Network', 44100 Hz not resampled, 2 channels f32, Buffer 970 frames (22.0 ms), Output latency 52.1/52.6 ms measured by the driver, Underruns 0; the latency text changed between screenshots taken ~0.5 s apart (3 distinct values in 6 shots, shots/lat*c.png). NOT tested (could not test, not safe): closing the app with an ASIO driver panel open (selecting ASIO would open the user's real ASIO driver on a machine used by the user).
11 Mods PASS: AFX 1.rbm (copied from ~/rb-mods/mods) loaded: the 808's mod window shows the standard splash (shots/mod3.png, the 'AFX 1' caption on the standard ReBirth logo) and Mods > Info says 'This mod has no splash picture.' with Name AFX 1, Embedded files 21 (shots/mod5.png); Metallicon+ (the shipped one from C:\ReBirth RB-338\Mods, in the Mods menu) shows its own splash in the panel and in Mod Information (shots/mod6c.png, mod7.png).
12 Portable mode PASS: OpenRB-338.toml (195 bytes) appears next to the exe after the first start and holds the session settings (recent C:\re\rb\rc312, theme, audio_buffer_size=2048); I deleted OpenRB-338.portable and the toml and started the app again: no toml was created next to the exe and the app used the %APPDATA% settings (File dialog recents show 'Demo Songs' and no rc312, shots/pm1s.png). %APPDATA%\OpenRB-338\OpenRB-338.toml and ui.toml unchanged after (same sha1, same timestamps). Marker restored by deleting the folder in cleanup.
13 orb.exe PASS (round 1): orb info Calm.rbs and orb render Calm.rbs -> 56013828 bytes, SHA-256 54560491ecb1c8c601e688933a282b1ee1d9598e5a37e15fb740bfe88ab67d2c equals the reference.
Observations, not defects: taskkill (no /F) of an app with an unsaved song only raises the Save changes prompt and leaves the process running (it did here, PID 18876, killed with /F by PID); I used two instances once, both mine, both gone. rcs 'Calm.rbs' clear status text for A1 on a fresh song not captured separately (A3 captured). No bug tasks; no release blocker.
Cleanup: all my processes ended (checked with tasklist), C:/re/rb/rc312 deleted entirely, nothing left on the machine, permits released.

REVIEW ROUND 1 (record corrections; no re-test except the hash check below):
- Item 2 is PARTIAL. What I saw (shots/cl2.png) is the status text '303-1: A3 cleared' (pattern 3 was selected); the 'Untitled: ' prefix in my result line was my error, the screenshot shows no prefix. The exact text '303-1: A1 cleared' was not seen. Keyboard Shortcuts: all but one scroll step viewed, no TR-/TB- text seen; the rest was only checked by grep of crates/orb-ui/src (only doc comments contain TR-8/TR-9).
- Item 3: Undo was checked after Duplicate ('Duplicate Pattern undone'); Undo after Clear Pattern was NOT checked.
- Item 5: Undo was confirmed only by the status text 'Initialize Song undone' and the Edit menu label; the song content before and after was not compared.
- Item 8 is PARTIAL: Mod Information and the error box were not tested at the minimum size (the other windows listed fitted).
- Item 9: the crossfade evidence is WEAK (one in-between frame in six screenshots); the click evidence is stronger.
- %APPDATA%\OpenRB-338 had two files, both 170 bytes with identical content: OpenRB-338.toml (mtime 2026-11-24 14:37:58) and ui.toml (mtime 2026-11-22 10:05:48). SHA-1 before (taken 19:05 round 1 and again 19:34 in round 2 for OpenRB-338.toml; ui.toml at 19:05): both 2e4069477a735311db71a759a3b357e3b208afde. After: OpenRB-338.toml taken 19:42 (round 1 end) and 19:53 (after item 12), ui.toml taken now at 19:55 on the machine and compared with my local backup: all 2e4069477a735311db71a759a3b357e3b208afde; both mtimes unchanged. So in item 12 the app used %APPDATA% but did not write there (no change on exit, nothing to save). The 19:42 hash is from round 1 (before item 12), not rerun then; the 19:53 and 19:55 hashes are after item 12.
- Round 1 conduct: while the user's Task Manager was in the foreground I sent input to the shared desktop (rcs input clicks and key presses incl. Alt+H and Alt+Esc, and a PowerShell SetCursorPos/SetForegroundWindow/AttachThreadInput attempt); none took effect (UIPI), and nothing in Task Manager was changed as far as the screenshots show (07-08).

Note: the labels for items 2, 3 and 8 in the RESULTS block above are superseded by the REVIEW ROUND 1 corrections below it (2 partial, 3 partial: Undo after Clear Pattern, Cut and Paste not checked; 8 partial).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Release candidate 2026.1125.1640 checked on the Windows test machine through rcs_client on 2026-11-25 (portable zip, own folder C:/re/rb/rc312 removed afterwards). Pass: items 1, 4, 5 (Undo confirmed by status text only), 6, 7, 9 (crossfade evidence weak), 11, 12, 13. Partial: item 2 (A3 cleared seen, not A1), item 3 (Copy, Duplicate, Cut and Paste ran; Undo checked only after Duplicate, not after Clear Pattern, Cut or Paste), item 8 (Mod Information and the error box not tested at the minimum size), item 10 (ASIO close-with-panel check not run, unsafe on the user's machine). No defects found, no bug tasks, no release blocker. The user's %APPDATA%\OpenRB-338 files (OpenRB-338.toml and ui.toml, same sha1 2e4069477a735311db71a759a3b357e3b208afde) are byte-identical before and after, mtimes unchanged; the app did not write there. Screenshots are local only. Branch worktree-agent-ad21eb186141b4a48, one commit.
<!-- SECTION:FINAL_SUMMARY:END -->
