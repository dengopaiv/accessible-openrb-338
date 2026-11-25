---
id: TASK-312
title: 'Release candidate 2026.1125.1640: agent UI check on Windows'
status: To Do
assignee:
  - '@agent-worker'
created_date: '2026-11-25T17:04:00Z'
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
- [ ] #1 Items 1 to 13 are each recorded pass or fail with evidence (screenshot path or exact observation) in the notes
- [ ] #2 Each real defect found has its own bug task; release blockers are reported to the supervisor
- [ ] #3 The user's settings, programs and files on the test machine are unchanged; the agent's folder is cleaned up
<!-- AC:END -->
