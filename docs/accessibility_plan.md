# Screen reader accessibility: the plan

**State on 2026-11-28.** Everything built so far is merged and covered by tests: the panel,
the menus, the dialogs, the mod chooser, and since the night of 2026-11-27 also upstream's
newer windows (Audio Output, the Initialize Song question, the four pattern editors) and the
classic look. The user's NVDA passes heard items 1 to 22 of the checklist working on
2026-11-07, and eight more on 2026-11-28. That second pass found three faults, now bug tasks
TASK-247 to TASK-249 (Part 2.1). All three are built and wait for their checks, items 36,
37 and 38. What is left besides them is mostly the user's: 7 checks by ear, 12 design
choices that agents made and the user has not confirmed (2.2), and the known gaps that have no
task yet (2.4). One small test task is open, and two tasks for other systems are on hold
(2.3).

The analysis behind the plan and a description of what the program does, part by part, are in
[accessibility.md](accessibility.md). The behavior in detail is in docs/ui_spec.md section 20.
The guide for users is the README's "Using OpenRB-338 with a screen reader". The backlog is the
source of truth for each task's status (`backlog task list -m m-7 --plain`).

This document has four parts:

1. [Done](#part-1-done): the decisions, the work, round by round, and what was measured.
2. [To do](#part-2-to-do): the checks by ear, the choices to confirm, the open tasks, and the
   known gaps.
3. [The NVDA checklist](#part-3-the-nvda-checklist): the by-ear test of screen reader mode,
   with each item marked heard or waiting.
4. [Reference](#part-4-reference): the ground rules, the risks, the upstream merges, and how
   the plan became tasks.

A task is in Part 1 when its backlog status is Done, even when its by-ear check is still
waiting. That check is then in Part 2.1, and the task's row in Part 1 points there.

## The milestone

**m-7: screen reader accessibility (NVDA first).** A blind musician can use OpenRB-338 with NVDA
on Windows, everything the panel does: play and stop, set every knob, fader and switch, choose
patterns, program the 303s and the drum machines and read their patterns back, record in song
mode, open, save and export songs, choose mods, and read every message and window. All of it
from the keyboard, with speech that says what each control is, what its value is and what a key
just did, and stays quiet while the music plays unless asked.

**Scope.** The desktop UI (`crates/orb-ui`), on Windows with NVDA first. The same
accessibility tree serves Narrator and JAWS (UI Automation), VoiceOver (macOS) and Orca (Linux)
through AccessKit, but only NVDA on Windows is tested, and the fork is built for Windows only
(the user, 2026-11-07).

Not in scope:
- the engine: no change at all, so the rendered audio stays byte for byte the original's;
- the `orb` command line tool, which already works with a screen reader;
- MIDI: orb-player's MIDI layer stays for a MIDI input port later (the user, 2026-11-27), with
  no task yet;
- any web version: the fork is a local editor only (the user, 2026-11-05, TASK-198).

**Done when** every task of m-7 is Done. Four are open on 2026-11-28: TASK-244 (the user's
NVDA pass, which also waits for the checks of the three fault fixes TASK-247 to TASK-249,
built), TASK-227, and TASK-199 and TASK-206, which are on hold.

---

## Part 1: Done

### 1.1 Decisions

Each decision is a Done task whose final summary records the choice and the date. The options
that were weighed are in its description. The decisions of m-7 are listed with those outside the
milestone that shaped it (marked "outside m-7"), and one work task's own choices.

| Task | Date | Decided |
|---|---|---|
| TASK-177 (A11Y-D1) | 2026-11-05 | Screen reader mode turns on by itself when Windows reports a screen reader. Options > Screen Reader Mode has Automatic, On and Off, kept in the settings. |
| TASK-178 (A11Y-D2) | 2026-11-05 | In the mode, Space presses or toggles the focused control and F5 is Play/Stop. Enter keeps the original's meaning (refined by TASK-207). |
| TASK-179 (A11Y-D3) | 2026-11-05 | A machine's mixer channel is part of the machine's section. |
| TASK-180 (A11Y-D4) | 2026-11-05 | The system's file dialogs in the mode. The browser's "Use as songs / mods folder" become Options items. |
| TASK-181 (A11Y-D5) | 2026-11-05 | Values in physical units, as exact as the engine's code allows, each marked by how exact it is. A quantity without a unit is a percent. |
| TASK-182 (A11Y-D6) | 2026-11-05 | The key map: F1 help, F2 to F4 read-outs, F5 Play/Stop, F6 sections, F9 Record, F10 menu bar, F11 Loop, F12 mode. Tab moves between controls. Ctrl+Up / Ctrl+Down move the row focus, Ctrl+Tab is the 303 tap. |
| TASK-198 (outside m-7) | 2026-11-05 | A local editor only: no web version, nothing online. |
| TASK-202 | 2026-11-06 | The panel's keys wait while a menu is in use, in both modes (an exception to "mode off is upstream"). |
| TASK-203 | 2026-11-06 | A name the system dialog did not confirm, of a file that exists, asks "Replace NAME?", with No as the default. |
| TASK-205 | 2026-11-06 | On a spin button, Page Up / Page Down are 10 clicks. Home / End go to the range's ends only on the short ranges. |
| TASK-207 | 2026-11-06 | Enter follows the focused control's section: in a 303's section it is Step, elsewhere Play. |
| TASK-208 | 2026-11-06 | Alt+Space opens Windows' system menu, so Shift+Space is a 909 accent key too (built as TASK-210). |
| TASK-209 | 2026-11-06 | The step editor and the grids stay on their pattern during playback. Options > Step Editor Follows Playback (off) makes them move along silently. Up / Down move a 303's step, and Left / Right do nothing there. |
| (TASK-195, a work task's own choices, no decision task) | 2026-11-07 | The mod chooser: Ctrl+M opens it. The filter and the list are two stops. The list is a tree with the variants under their name, and Ctrl+I shows Mod Info. |
| TASK-222 | 2026-11-09 | Releases are a portable zip, with no installer. |
| TASK-229 (outside m-7) | 2026-11-18 | From upstream's 107 commits: no landing page, no plugin, and none of upstream's backlog from task-176 on. |
| TASK-232 (outside m-7) | 2026-11-26 | From upstream's 120 commits: everything new comes in, and each new window gets its own m-7 task. The portable modes are combined. The spoken names say 808, 909 and 303 (TASK-239). |
| TASK-242 | 2026-11-27 | The 303 pattern editor in the mode: the whole window. The header's controls are buttons and fields, and the notes are a list of 16 steps with the panel step editor's keys. |
| TASK-243 | 2026-11-27 | The 808 and 909 pattern editors in the mode: the whole window. The header's controls are buttons and fields, and the grid has the panel drum grid's keys. |

TASK-220 (2026-11-10) took upstream's pull request 4, Save writes the engine's tracks. It is
not part of m-7.

### 1.2 The work, round by round

Rows from TASK-246 on give no hash, because a task's commit is amended as it lands; find it
with `git log --grep "Task: task-NNN"`. Each row says what the task gives the user and whether
its by-ear check is heard. "Heard" means
the user heard it working with NVDA, and the number is its checklist item (Part 3). "Waiting"
means the check is in Part 2.1. "None" means there is nothing to hear. With screen reader mode
off, every task leaves the app as upstream, except where Part 4's ground rules say otherwise.

**Round 1: the first build (2026-11-05 to 2026-11-07).**

| Task | What | Commit | By ear |
|---|---|---|---|
| TASK-183 (A11Y-0) | A headless test harness of the accessibility tree; `tools/uia_tree.ps1` reads the real window | 4073a58 | none |
| TASK-185 (A11Y-1) | Screen reader mode: the setting, and Windows' screen reader flag for Automatic | 650d17f | heard, 1 and 19 |
| TASK-184 (A11Y-11) | The menu bar from the keyboard: F10 or Alt alone, arrows, Enter, Escape | 582a030 | heard through 13 to 17, which reach their windows by the menus |
| TASK-186 (A11Y-2) | A name, a role and a value for every panel control | 8df3684 | heard, 3 |
| TASK-194 (A11Y-9) | The system's file dialogs; Options > Set Songs / Mods Folder; TASK-203's question | fe950d3 | heard, 11 and 20 |
| TASK-192 (A11Y-14) | Knobs and faders spoken in physical units (docs/control_units.md); Options > Speak Raw Values | e48d24e | heard, 7 |
| TASK-187 (A11Y-3) | Sliders, spin buttons and pattern selectors a screen reader can drive | 3a82419 | heard, 4 and 7 |
| TASK-188 (A11Y-4) | Nine sections, F6 and Tab through them, the key map, Space and Enter | 40c1850 | heard, 2 and 5 |
| TASK-189 (A11Y-5) | Announcements and the read-outs F2, F3 and F4; silence while the music plays | b8ee68a | heard, 6 and 21 |
| TASK-193 (A11Y-8) | Dialogs and windows: roles, the focus in and back, lines to read one by one | 988bcd5 | heard, 15 to 17 and 22 |
| TASK-190 (A11Y-6) | The 303 step editor, spoken step by step; F2 reads the pattern | 7c09ffa | heard, 8 |
| TASK-191 (A11Y-7) | The drum grids, cell by cell; F2 reads the pattern | d05000d | heard, 9 |
| TASK-210 | Shift+Space as an accent key on the 909's grid | 7267c55 | heard, 9 |
| TASK-195 (A11Y-10) | The mod chooser: Ctrl+M, a filter and a tree of the mods | 6cc85ab | heard, 12 |
| TASK-196 (A11Y-12) | The documentation: the README's guide, ui_spec section 20, Help > Keyboard Shortcuts | df4d002 | heard, 17 |
| TASK-197 (A11Y-13) | The user's first NVDA pass: items 1 to 22 work | (backlog) | itself |

**Round 2: wishes from the first pass (2026-11-07 to 2026-11-10).**

| Task | What | Commit | By ear |
|---|---|---|---|
| TASK-247 | Menu bar: Left and Right from menu to menu, as in Windows, never out of the bar | (see git log) | waiting, 36 |
| TASK-212 | After the menu bar, the focus returns to the control that had it | 3f3fdc7 | heard, 23 |
| TASK-211 | Options > Pitch Keys Make Notes: a 303 pitch key turns a rest into a note | b8adbc9 | heard, 24 |
| TASK-201 | The Mods menu reads the playing kit's entry as checked | 78129ba | waiting, 25 |
| TASK-204 | The system Export dialog keeps the format last used | 771ce7c | waiting, 26 |
| TASK-213 | An arrow key on a knob always moves to a value that sounds different | 3b5c2bd | heard, 27 |
| TASK-225 | The numpad's keys apart from the main keyboard's (a Windows message hook) | e909060 | heard, 28 |

**Round 3: starting on more Windows PCs (2026-11-07).** The program did not start on the
user's Windows 10 PC. None of these tasks needed an NVDA check: the user's check was that the
program starts (TASK-218). The details are in docs/building.md: "The run log, for a start that
fails", "The graphics on Windows: DirectX 12 first" and "Debugging OpenRB-338 on another
Windows PC".

| Task | What | Commit |
|---|---|---|
| TASK-214 | A run log of each start; a failed start leaves a trace | 2b76d08 |
| TASK-215 | A Windows debug kit: a build with symbols and `tools/win_diag.ps1` | 91c1e8d |
| TASK-216 | A test that failed in debug builds only (a test fix) | 7db8369 |
| TASK-217 | DirectX 12 before Vulkan on Windows (Intel's Vulkan driver crashed the start) | 4c75ec2 |
| TASK-218 | The user's check: the Windows 10 PC starts, 3 runs of 3 | (backlog) |
| TASK-219 | `tools/win_diag.cmd`, so the user need not type the execution policy | 7541709 |

**Round 4: releases (2026-11-09 to 2026-11-11).** No NVDA check.

| Task | What | Commit |
|---|---|---|
| TASK-221 | The release's README.txt names the fork | 8d7f41b |
| TASK-223 | A portable Windows zip: settings and logs beside the program | 4551c19 |
| TASK-228 | CHANGELOG.md, in the repository and the zip | 0251417 |

**Round 5: upstream's work and the plugin (2026-11-18 to 2026-11-27).** No NVDA check of its
own. The merges moved the panel's input into upstream's `controller.rs`. Items 1 to 22 were
heard before that, so Part 2.1 suggests hearing them again.

| Task | What | Commit |
|---|---|---|
| TASK-230 | Merged upstream 6858253: the panel redesign, the Edit menu, MIDI mapping | c9ae5ed |
| TASK-233 | Merged upstream f83c54f: the skins, the pattern editors, Audio Output, portable mode; the fork's screen reader code ported into `controller.rs` | fdd2ccc |
| TASK-231 | Removed upstream's VST3 plugin | 7052216 |
| TASK-241 | Removed the code the app shared with the plugin; orb-player's MIDI layer kept | 1ab21c3 |

**Round 6: upstream's new windows in screen reader mode (2026-11-27 and 2026-11-28).** Each of
these added one criterion to TASK-244, the user's open NVDA pass.

| Task | What | Commit | By ear |
|---|---|---|---|
| TASK-237 | Edit > Initialize Song from Pattern Mode: the question is read and takes the focus | 74d9e13 | waiting, 29 (TASK-249 is built) |
| TASK-239 | The spoken names say 808, 909, 303-1 and 303-2, as upstream's visible text does | 10ee4a4 | heard, 30 |
| TASK-234 | Options > Audio Output: named lists, the in-use lines, one announcement per change | 5d2d073 | fault heard, 31: TASK-248; waiting again, 31 |
| TASK-238 | Screen reader mode with View > Classic, as with the modern look | d9f1d25 | heard, 32 |
| TASK-235 | The 303-1 and 303-2 Pattern Editor windows | 05ba65b | heard, 33 |
| TASK-236 | The 808 and 909 Pattern Editor windows | 224eae9 | heard, 34 |
| TASK-245 | Audio Output shows the device in use after a fallback from a lost one | 59cc058 | waiting, 35 |
| TASK-248 | Audio Output's lists as a Windows combo box: Alt+Down, Enter and Space open, Up and Down on a closed box choose, Escape keeps the focus, a spoken hint | (see git log) | waiting, 37 |
| TASK-249 | A question opened from the menu (File > New, Edit > Initialize Song) waits for a new key: the Enter that chose the item no longer answers it | (see git log) | waiting, 38 |

**The documents (2026-11-28).** TASK-246 rewrote this plan into its done and to-do parts,
docs/accessibility.md into a description of what the program does with each part's state, and
the README's screen reader section to match. No NVDA check.

**In more detail.** What each accessibility task gives, and where it is described:

- **The harness (TASK-183).** `crates/orb-ui/tests/common/a11y.rs` builds the app's
  accessibility tree headless (no window, no GPU), finds nodes, reads names, roles, values and
  states, and sends a screen reader's actions. `a11y_baseline.rs` keeps the counts.
  `tools/uia_tree.ps1` reads the real window through UI Automation (docs/accessibility.md,
  section 6.3).
- **The mode (TASK-185).** Options > Screen Reader Mode (Automatic, On, Off) is the
  `screen_reader` key of the settings file. Automatic follows Windows' screen reader flag
  (`SPI_GETSCREENREADER`), asked at the start and whenever the window regains the focus.
- **The menu bar (TASK-184, TASK-212, TASK-201).** F10, or Alt pressed and released alone,
  focuses File. The bar has six titles: File, Edit, View, Mods, Options and Help. The arrows
  walk it, Enter chooses, and Escape closes. Checkboxes and radio items carry their state, and
  so do the Mods menu's entries. Leaving the bar gives the focus back to the control that had it
  (docs/ui_spec.md 18.2).
- **The panel's controls (TASK-186, TASK-187, TASK-192, TASK-213).** Every control has a
  spoken name, its section, a role and a value. Knobs and faders are sliders spoken in units
  ("1.24 kilohertz"), and an arrow key always moves to a value that sounds different. Each
  display with its arrows is one spin button. Each row's pattern buttons are one pattern
  selector ("Pattern B3"). Every change goes through the same model call as a click
  (docs/ui_spec.md 20 and 20.1, docs/control_units.md).
- **Sections and keys (TASK-188, TASK-207, TASK-225).** The nine sections are named groups,
  walked with F6 and Tab. A machine's section gives that machine the row focus. The function
  keys run the transport and the read-outs. The numpad keeps the original's meaning in every
  mode (docs/ui_spec.md 10 and 20.2).
- **Announcements (TASK-189).** A polite live region speaks the status messages, a song
  opened, a mod chosen, the keyboard mode and the transport keys. Nothing is spoken by itself
  during playback, which a test checks. F2 reads the pattern, F3 the transport and F4 the
  levels (docs/ui_spec.md 20.3).
- **Dialogs and windows (TASK-193, TASK-237, TASK-234, TASK-245).** Keyboard Shortcuts, File >
  Info, About and Mods > Info are dialogs with their title. The error box is an alert, and the
  unsaved-changes prompt and the Initialize Song question are dialogs with the focus on their
  default button. Audio Output is a dialog with four named lists and the in-use lines. The
  focus goes in when one opens and comes back when it closes (docs/ui_spec.md 20.4).
- **Files and mods (TASK-194, TASK-203, TASK-204, TASK-195).** Open, Save As, Export and Mods >
  Choose file use the system's dialogs. Export keeps the format last used. Ctrl+M opens the mod
  chooser, a filter and a tree of the 311 entries, opened in under 5 ms (docs/ui_spec.md
  20.7).
- **The 303 step editor (TASK-190, TASK-209, TASK-211).** In each 303 section, "Step editor"
  speaks the edit step and its contents ("Step 5 of 16: C sharp, octave up, accent, slide").
  Each programming key is answered, and F2 reads the whole pattern. It stays on its pattern
  during playback (docs/ui_spec.md 20.5).
- **The drum grids (TASK-191, TASK-210).** In each drum section, "Steps" is a grid of the
  instruments by the steps. Each cell reads "Bass drum, step 5, on". Space sets a step as a
  click does, and on the 909 Ctrl, Alt, Ctrl+Alt and Shift choose what is set (docs/ui_spec.md
  20.6).
- **The pattern editor windows (TASK-235, TASK-236).** Edit > 303-1, 303-2, 808 and 909
  Pattern Editor (or Ctrl+E on a row) open a window for one pattern. In the mode it is a
  dialog: the focus lands on Pattern, and Tab reaches every control by name. The 303's notes
  are a list of 16 steps with the panel step editor's keys. The drums' steps are a grid with
  the panel drum grid's keys. F2 reads the window's pattern. Playback neither moves the window
  nor makes it speak (docs/ui_spec.md 20.8 and 20.9).
- **The classic look (TASK-238).** With View > Classic the panel reads and works as the modern
  one: the same names, sections, Tab order, selectors, grids and step editor. Switching the
  look keeps the focus and says nothing. The classic panel's 808 instrument knob is not a stop
  (Part 2.2, choice 1).
- **The names (TASK-239).** The spoken names say 808, 909, 303-1 and 303-2 (they were TR-808,
  TR-909, Bass Line 1 and 2), in the sections, F2, and Help > Keyboard Shortcuts.

Also done along the way, outside m-7:
- TASK-176: the third-party songs test compares paths in NFC.
- TASK-200: a crash test that failed in a Windows release build.
- TASK-224 merged upstream's pull request 4.
- TASK-226 took the beat cookbook (docs/beats.md) from upstream's pull request 6.
- TASK-240 took upstream's third-party manifests: 326 mods and 5685 songs, all equal to the
  original's renders.

### 1.3 What was measured

The real window, read through UI Automation with `tools/uia_tree.ps1` (docs/accessibility.md
6.3), last on 2026-11-07, before the upstream merges:

| | Before (2026-11-05, cbca449) | Mode off (2026-11-07) | Mode on (2026-11-07) |
|---|---|---|---|
| Elements | 256 | 198 | 593 |
| Can take the focus | 254, of them 4 named | 194, all named | 198, all named |

Before the work, Tab walked through 250 silent stops. On 2026-11-07 every stop had a name. With
the mode on, the 593 elements include the 384 cells of the two drum grids.

The headless tree, which the test `a11y_baseline.rs` holds, on main of 2026-11-28 (after the
merges and the new windows):

| | Mode off | Mode on |
|---|---|---|
| Nodes | 205 | 600 |
| Named | 198 | 593 |
| Can take the focus | 196 | 200 |

The two tables come from different views of the tree (egui's nodes, UI Automation's elements)
and do not compare one to one. Repeating the real-window count after the merges is in Part
2.4.

---

## Part 2: To do

### 2.1 Checks by ear, and the faults they found

Agents cannot hear NVDA, so each of these was checked by tests only. The steps for each are in
the checklist (Part 3).

**Waiting for the user.**

| Item | Task | What to hear |
|---|---|---|
| 25 | TASK-201 | The Mods menu reads the playing kit's entry as checked (with no mods installed, the Standard ReBirth entry) |
| 26 | TASK-204 | The Export dialog offers the format last used, also after a restart |
| 29 | TASK-244, criterion 1 (TASK-237) | The Initialize Song question is read, and the focus comes back; TASK-249 is built, so it can be heard now |
| 38 | TASK-244, criterion 10 (TASK-249) | File > New and Edit > Initialize Song chosen from the menu ask their question, which NVDA reads, and wait for your answer |
| 31 | TASK-244, criterion 3 (TASK-234) | Options > Audio Output, once more (the user heard faults there; TASK-248 fixed them, 37 is the new keys) |
| 35 | TASK-244, criterion 7 (TASK-245) | Audio Output after a lost device reads the device in use |
| 36 | TASK-244, criterion 8 (TASK-247) | Left and Right go from menu to menu in the bar, wrapping |
| 37 | TASK-244, criterion 9 (TASK-248) | Audio Output's lists: the spoken hint, Alt+Down, Up and Down, Escape keeping the focus |

**Heard working on 2026-11-28:** items 23 (TASK-212), 24 (TASK-211), 27 (TASK-213), 28
(TASK-225), 30 (TASK-239), 32 (TASK-238), 33 (TASK-235) and 34 (TASK-236).

**Faults found by ear on 2026-11-28**, each now a bug task (Part 2.3), each adding its own
by-ear check to TASK-244 when it is built:

- TASK-247 (reported in item 26's place; item 26 itself waits), built, waiting for item 36: in
  an open menu Left went back to the bar and the other menus could not then be reached, and
  Right from Help left the bar for the Transport section. The user chose Windows' way: Left
  and Right go from menu to menu, wrapping, never out of the bar.
- TASK-248 (item 31), built, waiting for item 37: Audio Output's lists opened only with
  Space, which nothing said, and Escape out of an open list lost the focus. The user chose
  Windows' combo keys (Alt+Down, Enter, Up and Down on a closed list), a spoken hint, and a
  line in Keyboard Shortcuts; all three are in, and Escape keeps the focus on the box.
- TASK-249, built, waiting for item 38 (item 29 can now be heard): with unsaved changes, File
  > New chosen from the menu opened Save As at once; Ctrl+N asked first, as it should. The
  cause was the Enter that chose the item: the question is drawn in the same frame, and
  read that Enter as its answer (Save, which on an untitled song opens Save As; on Initialize
  Song's question Enter is Initialize). A question now takes no key in the frame that draws
  it first, and only new presses after it, not a held key's repeats. It holds in both modes
  and for every question the menu opens.

Items 29, 31 and 35 are TASK-244's acceptance criteria. When they are heard, the user checks
them there and TASK-244 is Done. Items 25 and 26 have no open task: their tasks are Done, and
only this checklist holds their marks.

**Worth repeating.** Items 1 to 22 were heard on 2026-11-07. The merges of 2026-11-18 and
2026-11-26 moved the screen reader code into upstream's `controller.rs` and changed the menu
bar (Edit and View were added). The tests pass, but the items have not been heard since. Going
through them again in the same pass would show anything the tests do not catch.

### 2.2 Choices waiting for the user

The agents of task-235, task-236 and task-238 had to settle these to finish, and recorded
them in their tasks' notes for the user's review. Each is built as described. The user answers
in a session: a choice kept is recorded as decided, and a choice changed
becomes a task.

The classic look (task-238):

1. The classic panel has a round 808 knob for choosing the drum instrument. In screen reader
   mode it is hidden and is not a stop, because Up and Down on the drum grid choose the
   instrument already. The alternative is to name it and make it a stop.

The 303 Pattern Editor windows (task-235):

2. One Pattern spin button (A1 to D8), as on the panel, instead of the window's bank buttons,
   number and arrows.
3. No pitch mode in the window. The panel's step editor has one; the window has the step
   editor's other keys.
4. The window's programming keys work in either keyboard mode, also when the panel's keys are
   set to select patterns (Ctrl+G).
5. The window opens on the pattern the step editor last chose (its pinned pattern), not on the
   one the song is playing. Task-235's notes leave this one to the user.
6. The steps are one control you move through, as the panel's step editor, not 16 separate
   items.

The 808 and 909 Pattern Editor windows (task-236):

7. The grid's rows start with Accent, as on the panel. Upstream's window, as drawn with the
   mode off, has AC at the bottom; in the mode the window's picture follows the grid, Accent at
   the top.
8. The window has its own cursor. It starts at Bass drum, step 1 each time the window opens,
   and moving it does not change the panel's selected instrument.
9. The Tab order: Pattern, Steps, the 909's Entry, Length, Shuffle, Shift left, Shift right,
   Copy, Paste, Clear. Steps comes second, as the 303 window's Step editor.
10. The cells say the panel grid's words (on, accent, flam, accent and flam). The 909's Entry
    switch says both upstream's names and the grid's words: "Cycle", "Low (on)", "High
    (accent)", "Flam".
11. Holding Ctrl or Alt does not change the Entry display. The keys pressed with Space choose
    what is set, as on the panel.
12. Setting one hi-hat clears the other without saying so, as on the panel.

### 2.3 Open tasks

| Task | Priority | What | State |
|---|---|---|---|
| TASK-244 | medium | The user's NVDA pass of the night's work: items 29, 31, 35, 36, 37 and 38 left | waiting for the user |
| TASK-227 | low | App-level tests of the numpad hook's queue (a key held while a dialog is open; numpad * Record; numpad Enter with a 303 focused) | for an agent |
| TASK-199 | low | Screen reader detection on macOS (VoiceOver) and Linux (AT-SPI) | on hold: Windows only |
| TASK-206 | low | A system file dialog that cannot open reads as a cancel (Linux) | on hold: Windows only |

TASK-199 and TASK-206 can be checked only on macOS and Linux. They are on hold by the user's
choice of 2026-11-07: "We will not have a linux, mac or currently at least anything other than
windows."

### 2.4 Known gaps, with no task yet

These were found while checking the plan against the code, or are limits the work already
named. None has a task. The user decides whether each becomes one.

- **The row focus is not spoken.** Ctrl+Up and Ctrl+Down move the row focus (where the pitch
  and drum keys go) without a word, and entering a machine's section says only the section's
  name. The proposal of 2026-11-05 (docs/accessibility.md section 3.2) meant both to be announced.
  F2 with the focus outside the panel's sections reads the row that has the row focus.
- **Braille.** No one has checked what a braille display shows: neither the controls' names
  and values nor how NVDA presents the announcements (live regions) in braille.
- **The meters** are read as lit segments by F4 ("7 of 10"), not in decibels: the original's
  meter scale has not been read.
- **The drum grids** are not real tables to the screen reader. Each cell carries its instrument
  and step in its own name, and the arrow keys are the program's. NVDA's table commands do
  nothing there.
- **Other screen readers.** Narrator and JAWS read the same UI Automation tree, but no one has
  tried them. Automatic detection is Windows only (TASK-199, on hold).
- **The real-window count** (`tools/uia_tree.ps1`, Part 1.3) has not been repeated since the
  upstream merges. The headless counts in `a11y_baseline.rs` are current.
- **A MIDI input port**, when it comes (the user kept the MIDI layer for it on 2026-11-27),
  will need its own screen reader work: choosing the port, and what is spoken when notes
  arrive.

---

## Part 3: The NVDA checklist

Run it with NVDA (note its version) and the release build, with screen reader mode on
(Automatic turns it on while NVDA runs). Marks:
- (x): heard working;
- (!): a fault was heard; its bug task is named;
- ( ): waiting.

Items 1 to 22 were heard on 2026-11-07 (TASK-197), items 23, 24, 27, 28, 30 and 32 to 34 on
2026-11-28. Item 31 had faults (TASK-248, built); 25, 26, 29, 31, 35, 36, 37 and 38 are waiting (Part 2.1).

1. Start the program: the window title is spoken, and the focus is on the first control of the
   Transport section. (x)
2. F6 through all sections and back with Shift+F6: each section's name is spoken. (x)
3. Tab through the Transport section: each control's name, role and value are spoken. (x)
4. Change the tempo with Up and Down: the new tempo is spoken; Page Up and Page Down. (x)
5. F5 plays and stops; nothing is spoken by itself while it plays. Space on a focused toggle
   toggles it and does not play. F9 Record, F11 Loop and F12 mode are spoken when pressed.
   (The user's note: Loop is F11, F12 is song and pattern; recording is in song mode.) (x)
6. F3 reads the transport while playing and while stopped. (x)
7. In 303-1: set the cutoff with the arrows and with NVDA's own slider commands; set the
   waveform; mute channel 1; choose pattern B3. (x)
8. Program a 303 pattern with the keys alone; each step is spoken; F2 reads it back
   (docs/ui_spec.md 20.5). On the focused "Step editor":
   - each programming key's new state is spoken once, by the value change (`-`, `P`, `[`, a
     new note), not twice;
   - the same note pressed again is spoken (by the announcer);
   - Up, Down, Home and End speak the new step, and Left and Right nothing;
   - in pitch mode (Tab three times, Space) a pitch key speaks the step it set ("Step 1: E")
     and then the next step's value, both, in that order, neither cut off;
   - play a song in song mode with the editor focused: nothing is spoken when the song moves
     to another pattern, with Options > Step Editor Follows Playback off and on; F2 says "Step
     editor on pattern ..." while off. (x)
9. In the 808: walk the grid, set a bass drum on steps 1, 5, 9, 13, and F2 reads it back. The
   same on the 909 with an accent and a flam. By ear:
   - Tab onto the grid speaks "Steps" and the cell ("Bass drum, step 1, off");
   - each Up, Down, Left and Right speaks the new cell;
   - each Space speaks the cell's new state;
   - Ctrl+Space, Alt+Space and Ctrl+Alt+Space on the 909 give a note, an accent and a flam
     (Alt+Space also opens the system menu, TASK-208); Shift+Space gives an accent and no menu
     (TASK-210);
   - while a song plays in song mode with the focus on a grid, nothing is spoken when the
     machine moves to another pattern, with Options > Step Editor Follows Playback off and on;
     F2 says "Grid on pattern ..." while off. (x)
10. Record in song mode: a pattern change and a knob move are recorded; play them back. (x)
11. File > Open in the system dialog; open a song from rb-songs; its name and mod are spoken.
    (x)
12. Choose a mod with the mod chooser, with the filter; Mods > Info reads the mod's text;
    Mods > Demo Song. By ear:
    - Ctrl+M speaks the dialog's name "Choose Mod" and the filter ("Filter mods", edit);
    - typing narrows the list, and the count line is readable;
    - Down moves to the list, and NVDA speaks "Mods, tree view" and the selected item with its
      level and "collapsed" for a mod with variants ("Name, variants: 3, collapsed, 5 of
      311");
    - Right speaks "expanded" and then the first file; Left goes back;
    - Up and Down speak each item with "has demo" and "current";
    - Enter chooses, and "Mod X" is spoken;
    - Ctrl+I opens Mod Info, and Escape returns to the list;
    - Tab and Shift+Tab move between the two stops;
    - Escape closes, and the focus is back where it was. (x)
13. File > Info: read and edit the song's title. (x)
14. Save, Save As, and Export Loop as Audio File: the status messages are spoken. (x)
15. Trigger an error (open a file that is not a song): the error box is spoken and OK closes
    it. (x)
16. Close with unsaved changes: the prompt is spoken, and the focus is on Save. (x)
17. Help > Keyboard Shortcuts: the list can be read line by line. (x)
18. Turn the mode off: the original keys behave as before (Tab taps the 303 while playing).
    (x)
19. With NVDA closed (no screen reader running), Options > Screen Reader Mode set to Automatic
    leaves the mode off. (x)
20. In screen reader mode, Save As, Export Loop as Audio File, Mods > Choose file and Options >
    Set Songs Folder / Set Mods Folder open the system dialog, NVDA reads it, and a file saved
    or exported through it is written as through the browser. Type a name with a dot and no
    extension (`v1.2 mix`) over an existing `v1.2 mix.rbs`: the question "Replace v1.2
    mix.rbs?" is spoken, and No is the default. (x)
21. The announcements are spoken without moving the focus:
    - a status message (Save);
    - a song opened (its name and mod), and a mod chosen;
    - the keyboard mode (Ctrl+F, Ctrl+G);
    - the transport keys (F5, F9, F11, F12, Enter, Space on Play and Stop, + and -, a pattern
      key in Select Patterns mode);
    - the read-outs F2, F3 and F4.

    Press F3 twice in a row: the same message is spoken both times. (x)
22. The windows (docs/ui_spec.md 20.4), each opened from the panel with the mode on:
    - Keyboard Shortcuts, File > Info, Help > About and Mods > Info are announced as dialogs
      with their title, and NVDA reads the first line as the focus enters;
    - Tab reads About's, Keyboard Shortcuts' and Mod Info's lines one by one ("Ctrl+N: File >
      New", "Name: Standard ReBirth");
    - Escape closes each, and the focus returns to the control that had it;
    - the error box (item 15) is announced as an alert, "Error", with its message read when
      the focus lands on OK; Escape or Enter closes it, and the focus returns, also to the
      window under it;
    - the unsaved-changes prompt (item 16) is read with its question, and the focus is on
      Save. (x)
23. The focus after the menu bar (TASK-212, docs/ui_spec.md 18.2), in screen reader mode with
    the focus on a panel control (a knob, Play):
    - F10 (or Alt alone), then Escape: NVDA reads the control again;
    - F10, Down, one Escape: the same;
    - F10 and F10 again, and Alt and Alt again, on a title and with a menu open: the menu
      closes, and NVDA reads the control;
    - Options > Step Editor Follows Playback chosen with Enter: the control is read;
    - Help > Keyboard Shortcuts: the window is read, and Escape returns to the control, not to
      "window". (x)
24. Pitch Keys Make Notes (TASK-211, docs/ui_spec.md 20.5), in screen reader mode on a 303's
    step editor on an empty pattern:
    - with Options > Pitch Keys Make Notes off (the default), the pitch key `V` answers "Step 1:
      rest, D", and the step stays a rest;
    - turn the item on (Enter on the checkbox; NVDA reads its state): `V` answers "Step 1: D",
      the editor reads "Step 1 of 16: D" with no "rest", and F5 plays the step;
    - `-` makes it a rest again;
    - in pitch mode each key makes its step a note, the step it set is said, and F5 plays all
      of them;
    - a step that is already a note only changes its note, and Clear makes every step a rest;
    - the item is saved: restart, and it is still on. (x)
25. The Mods menu's marks (TASK-201), with a mod playing (or the built-in kit): open the Mods
    menu and walk it with Up and Down. NVDA reads the playing kit's entry as a checked menu
    item ("Zed, menu item, checked") and every other mod entry as not checked; the Standard
    ReBirth entry is checked when the built-in kit plays. A mod of a shared name: the name's
    entry (a submenu, read as a button) and, inside it, the playing file read as checked. ( )
26. The Export format (TASK-204), in screen reader mode with a song in song mode:
    - File > Export Loop as Audio File opens the system dialog with WAV as the type;
    - save as `a.flac` (pick FLAC in the type list or type the extension);
    - export again: the type list offers FLAC first, and NVDA reads it as the file type;
    - a name typed without an extension (`b`) is saved as `b.flac`;
    - restart the program: the dialog still offers FLAC first;
    - with the mode off, the in-app browser starts on WAV as before. ( )
27. The next value that sounds different (TASK-213), in screen reader mode with Speak Raw
    Values off:
    - focus the 808 Bass drum tone and press Up repeatedly from 0: every press is spoken with
      a new value ("42.9 hertz", then the next one), and Down comes back the same way;
    - Page Up lands on a different value;
    - in the top group of values (120 to 127) Up and Page Up do not move;
    - the 303's Cutoff moves one position per press;
    - with Options > Speak Raw Values on, each press moves one position and speaks the number.

    With the mode off, none of this applies. (x)
28. The numpad keys (TASK-225), with Num Lock on, in screen reader mode and Program Synth mode
    (the default):
    - numpad - lowers the tempo and numpad + raises it, and the new tempo is announced as for
      the tempo buttons (a held key keeps repeating, and the end value is spoken on release);
    - the main key right of 0 is still Note/Pause on a focused 303;
    - numpad Enter plays even with a 303's section focused; numpad / switches Loop; numpad *
      starts Record like Insert; numpad 0 stops; numpad . does nothing;
    - in Select Patterns mode, numpad 4 / 5 still wind and numpad 1 / 2 locate, while the
      main-row digits select patterns;
    - with Num Lock off, NVDA's desktop layout takes the numpad (numpad + is its Say all with
      review), and the digits become Insert, End and the arrows, as Help > Keyboard Shortcuts
      says. (x)
29. Edit > Initialize Song from Pattern Mode (TASK-237), in screen reader mode: NVDA reads the
    title and the original's text with Initialize when the question opens; Tab reaches Cancel;
    after either answer the focus is back where it was. ( )
30. The names (TASK-239): F6 through the sections says 303-1, 303-2, 808, 909 (not Bass Line
    1 / 2, TR-808, TR-909); F2 on a drum grid starts with 808 or 909; Help > Keyboard
    Shortcuts in screen reader mode shows the same names. (x)
31. Options > Audio Output (TASK-234):
    - the window is announced as Audio Output, and the focus lands on Driver;
    - Driver, Device, Sample rate and Buffer size are each read with their name and the chosen
      value;
    - Down, Up and Enter in an open list change it, and NVDA says what changed once ("Sample
      rate: 48000 Hz");
    - Tab reaches the In use lines and reads them as "label: value";
    - the numbers refreshing every 500 ms are never spoken;
    - Escape closes an open list first, then the window, and the focus returns. (!) The lists
      opened only with Space, which nothing said, and Escape out of a list lost the focus:
      TASK-248 fixed both (item 37); hear this item again with it. ( )
32. The classic look (TASK-238):
    - with View > Classic in screen reader mode, F6 and Tab go through the sections and
      controls with the same names as the modern look;
    - the pattern selectors, the drum grids (F2 reads them) and the 303 step editor work;
    - the 808's round instrument knob is not read (Up and Down on the drum grid choose the
      instrument);
    - View > Classic and View > Modern each say only the menu item, and the focus stays on the
      control it was on. (x)
33. The 303 Pattern Editor (TASK-235): Edit > 303-1 Pattern Editor... (and Ctrl+E on a 303) in
    screen reader mode:
    - NVDA says the window's title and lands on Pattern (A1);
    - Tab reads Step editor with its step ("Step 1 of 16: rest, C"), Length, Shuffle,
      Transpose up and down, Shift left and right, Copy, Paste and Clear by name;
    - on the Step editor, Up, Down, Home and End are read once each, and Left and Right say
      nothing;
    - C on a rest says "Step 1: rest, C" once; `-` and P are read once as the new value;
    - Pattern and Length changes are read once; Transpose up says "Pattern A1 transposed up";
    - F2 reads the pattern; while the song plays nothing is spoken;
    - Escape closes the window, and NVDA is back on the panel control. (x)
34. The 808 and 909 Pattern Editors (TASK-236): Edit > 909 Pattern Editor... (and Ctrl+E on the
    909's grid) in screen reader mode:
    - NVDA says the window's title and lands on Pattern (A1);
    - Tab reads Steps with its cell ("Bass drum, step 1, off"), Entry (Cycle), Length,
      Shuffle, Shift left and right, Copy, Paste and Clear by name;
    - on Steps, the arrows, Home and End read the new cell once;
    - Space, Ctrl+Space, Shift+Space and Ctrl+Alt+Space read the cell's new state once (on,
      accent, flam);
    - Entry's Up and Down are read once ("High (accent)");
    - Shift right says "Pattern A1 shifted right" (on an empty pattern "Pattern A1
      unchanged");
    - F2 reads the pattern; while the song plays nothing is spoken;
    - Escape closes the window, and NVDA is back on the panel control.

    The same in the 808 Pattern Editor, which has no Entry and where Space toggles. (x)
35. Audio Output after a lost device (TASK-245): after the program falls back from a device
    that is gone, NVDA reads the Device list as the device in use (System default), not the
    lost one. ( )
36. Left and Right in the menu bar (TASK-247, docs/ui_spec.md 18.2), in screen reader mode:
    F10, then Right and Left on the menu names: NVDA says each name once (File, Edit, View,
    Mods, Options, Help), and Right from Help says File, Left from File says Help, never a
    control of the panel. With a menu open (Down on a name), Right and Left close it and open
    the next or previous one, and NVDA says that menu's first item once; in the Mods menu,
    Right on a name that is a list of files opens the list and says its first file, Left
    closes it and says the name again. ( )
37. Audio Output's lists as a Windows combo box (TASK-248, docs/ui_spec.md, Options > Audio
    Output), in screen reader mode:
    - NVDA reads each of Driver, Device, Sample rate and Buffer size with its value, then
      "Alt+Down opens the list";
    - Alt+Down, Enter and Space open the list; NVDA says the highlighted entry, which starts
      on the chosen one;
    - Up and Down in the list move the highlight and NVDA says each entry once; it chooses
      nothing; at the ends it stays;
    - Enter, Space or Alt+Up choose the highlighted entry: NVDA says the change once ("Sample rate:
      48000 Hz") and reads the box again, because the focus stays on it;
    - Escape closes the list, keeps the choice as it was, says nothing new, and NVDA reads the
      box again; a second Escape closes the window and the focus returns to the panel;
    - on a closed box Up, Down, Home and End choose the previous, next, first and last entry
      at once, each said once, no wrap;
    - Help > Keyboard Shortcuts in screen reader mode has a line "In Options > Audio Output"
      with these keys. ( )
38. A question opened from the menu (TASK-249), in screen reader mode, with the song changed
    and not saved:
    - F10, Down, Enter on File > New: NVDA reads the question "Save changes to ...?" with the
      focus on Save. Nothing is saved and no Save As opens until you answer: Enter or S on
      Save, D on Don't Save, Escape to cancel; Ctrl+N asks the same;
    - in song mode, Edit > Initialize Song from Pattern Mode the same way: NVDA reads the
      question and nothing is initialized until you press Enter on Initialize or Escape. ( )

---

## Part 4: Reference

### Ground rules of every task of the milestone

- With screen reader mode off, the app behaves exactly as upstream, key for key, except where
  the user decided otherwise:
  - the panel's keys wait while a menu is in use (TASK-202);
  - every start writes a run log (TASK-214);
  - Windows prefers DirectX 12 before Vulkan (TASK-217);
  - the numpad's keys have the original's meaning in every mode (TASK-225), while the main
    keyboard's keys do not change.

  The existing tests of `crates/orb-ui/tests` (keyboard.rs in particular) pass unchanged.
- Every accessible action goes through the same call as the mouse or the original's key:
  - on the panel, the `model::Panel` methods (`press`, `arrow`, `set_knob`, `set_fader`,
    `set_switch`, `select_instrument`, `set_step808`, `set_step909`, `tap_303`, `tap_drum`),
    reached through upstream's `controller.rs`;
  - in the pattern editor windows, the windows' own acts (`App::editor_act`,
    `App::drum_editor_act`, then `App::write_pattern`).

  So the events the engine hears, recording in song mode included, are the same.
- The accessible layer lives in `crates/orb-ui/src/a11y/`, plus
  `crates/orb-ui/src/mod_chooser.rs` and small hooks in `controller.rs`, `app.rs`, `menu.rs`,
  `audio_window.rs` and the editors' files.
- Nothing is announced by itself during playback: not the position, the meters, the LEDs, the
  running lights, or a pattern change by the song. What the user does is confirmed, and
  nothing is said twice.
- A setting that makes something move along (Step Editor Follows Playback) moves it silently.
  Up and Down move a list, and Left and Right do nothing where there is nothing beside it.
- Each task adds headless tests of the accessibility tree for what it builds. It updates
  docs/ui_spec.md and Help > Keyboard Shortcuts (`crates/orb-ui/src/key_help.rs`) for what it
  changes, and a line in CHANGELOG.md's next section. It also updates this plan (its row in
  Part 1, its check in Parts 2.1 and 3) and the state mark of its part in
  docs/accessibility.md.
- A check that needs NVDA by ear is not done by the agent. The task narrows it to what the
  tests show, and the by-ear part goes to the user's open pass task (TASK-244 now) and to this
  checklist.
- Lines of Rust are at most 100 columns, and `cargo clippy --all-targets -- -D warnings` is
  clean.

### Risks and how the plan meets them

- **egui changes.** AccessKit's integration in egui moves between versions (0.37 deprecated
  `WidgetType` and `Id::new`). The accessible layer stays in one module and talks to egui
  through a few calls. An egui upgrade from upstream is checked with the harness's tests.
- **Upstream changes the panel.** The merge of f83c54f moved all of the panel's input into
  upstream's `controller.rs`, so the fork's hooks moved there too (TASK-233). Each upstream
  merge is its own task, decided by the user first, and the a11y tests show what it broke.
- **What NVDA does with a custom tree.** Live regions, value changes of a focused slider and
  grid navigation are spoken differently by NVDA, JAWS and Narrator. Every such behavior is in
  the NVDA checklist.
- **Frame cost.** The tree is rebuilt each frame, and the panel repaints at 30 frames per
  second while playing. `crates/orb-ui/tests/frame_cost.rs` measures it. If the cost grows,
  the nodes that do not change can be cached.
- **Units that are only estimates.** docs/control_units.md marks each unit [C], [D] or [G], and
  Options > Speak Raw Values gives the exact stored number next to it.
- **Space no longer plays in screen reader mode.** This breaks a habit from ReBirth (D2). F5
  plays, Enter still plays outside a 303's section, and Help > Keyboard Shortcuts says so.
- **Speech over music.** The rule "nothing by itself during playback" is checked by a test
  (TASK-189), not left to hope.

### The upstream merges

The fork started from upstream's cbca449 (its tag v2026.1103.0917). Each merge is a two-parent
merge commit, so later merges start from the right base.

- **32a642d (TASK-224, 2026-11-10).** Upstream's pull request 4: every Save writes the
  engine's tracks (TASK-220, the user's choice).
- **6858253 (TASK-230, 2026-11-18).** The panel redesign, a double click that resets a knob,
  the Edit menu (Undo, Cut, Copy, Paste, Duplicate Pattern), the merging settings file, the
  optimized test profile, the MIDI Standard Mapping layer, and fixes. Left out (TASK-229): the
  landing page and the plugin.
- **f83c54f (TASK-233, 2026-11-27).** The skin layer and View > Classic, the pattern editors,
  Copy Pattern as JSON, Clear Pattern, Initialize Song / Loop, Options > Audio Output with
  ASIO, Ctrl+I for File > Info, log-scale meters, a new song's sample patterns, OpenRB-338.toml
  and upstream's portable mode. Each new window got its own m-7 task (round 6 of Part 1.2).
- **Not taken.** The landing page (removed in TASK-230), the VST3/CLAP plugin (TASK-231 and
  TASK-241), and upstream's backlog from task-176 on and its milestones from m-7 on, whose
  numbers collide with the fork's (AGENTS.md, "This fork"). Pull request 6's orb-mcp is not
  wanted here (only its beat cookbook, TASK-226).

### How the plan became tasks

The plan was written here on 2026-11-05 as decisions (A11Y-D1 to D6) and work (A11Y-0 to
A11Y-14). It was then created in the backlog with the CLI (1.55.3) as it stood (commit
2611c80): milestone m-7, the decisions as Done spikes, and the work as tasks with ordinals from
200000. Since then the backlog is the source of truth. The plan ids remain only as names
(docs/ui_spec.md section 20 refers to them):

- Decisions: A11Y-D1 = TASK-177, A11Y-D2 = TASK-178, A11Y-D3 = TASK-179, A11Y-D4 = TASK-180,
  A11Y-D5 = TASK-181, A11Y-D6 = TASK-182.
- Work: A11Y-0 = TASK-183, A11Y-11 = TASK-184, A11Y-1 = TASK-185, A11Y-2 = TASK-186,
  A11Y-3 = TASK-187, A11Y-4 = TASK-188, A11Y-5 = TASK-189, A11Y-6 = TASK-190,
  A11Y-7 = TASK-191, A11Y-14 = TASK-192, A11Y-8 = TASK-193, A11Y-9 = TASK-194,
  A11Y-10 = TASK-195, A11Y-12 = TASK-196, A11Y-13 = TASK-197.

The full text each task was created with (its why, what, references and acceptance criteria) is
in the backlog task, and in this document's history (`git show
3e2f7d2:docs/accessibility_plan.md`).

On Windows, a script that passes multi-line text to the CLI must run `node .../backlog.md/cli.js`
directly. Through npm's `backlog.cmd` shim, cmd.exe cuts an argument at its first newline and
drops the rest of the command line.

**How the work is run** (the user's rules, AGENTS.md "This fork"):
- one agent at a time;
- each task's branch is reviewed by agent-reviewer before it is merged;
- a reviewed branch is merged as one commit, the full tests and the corpus compare run on
  main, and main is pushed to the fork;
- no release is made until the user asks.
