# Screen reader accessibility: the plan

**State on 2026-11-09: everything planned is built and merged, and the user's NVDA pass
(TASK-197) found every item of its checklist working. The follow-ups of 2026-11-07 are done too,
and so is the start on the user's Windows 10 PC (TASK-214 to TASK-219). What is left, TASK-199
and TASK-206, is on hold: the fork is Windows only for now (the user, 2026-11-07).**

The analysis behind the plan, with the measurements and the options, is in
[accessibility.md](accessibility.md); the behavior in detail is in docs/ui_spec.md section 20,
and the guide for users is the README's "Using OpenRB-338 with a screen reader". The backlog is
the source of truth for each task's status (`backlog task list -m m-7 --plain`); this document
says what the milestone is, what of it is finished and what is still to do.

The document has three parts:

1. [Finished](#part-1-finished): the decisions, the work, and what was measured.
2. [To do](#part-2-to-do): the open tasks, and the NVDA checklist (items 1 to 22 pass; 23 to 27,
   the follow-ups' by-ear checks, wait for the user).
3. [Reference](#part-3-reference): the ground rules, the risks, and how the plan was turned into
   tasks.

## At a glance

### Finished

| Plan id | Task | What | Commit |
|---|---|---|---|
| A11Y-D1..D6 | TASK-177..182 | The six up-front decisions | 2611c80 |
| A11Y-0 | TASK-183 | Test harness of the accessibility tree, the UI Automation probe | 4073a58 |
| A11Y-1 | TASK-185 | Screen reader mode: the setting and the detection on Windows | 650d17f |
| A11Y-11 | TASK-184 | The menu bar from the keyboard | 582a030 |
| A11Y-2 | TASK-186 | A name, a role and a value for every panel control | 8df3684 |
| A11Y-9 | TASK-194 | System file dialogs in screen reader mode | fe950d3 |
| A11Y-14 | TASK-192 | Physical units for every knob and fader | e48d24e |
| A11Y-3 | TASK-187 | Sliders, spin buttons, pattern selectors a reader can drive | 3a82419 |
| A11Y-4 | TASK-188 | Sections, the focus order and the key map | 40c1850 |
| A11Y-5 | TASK-189 | Announcements and the read-out keys F2 to F4 | b8ee68a |
| A11Y-8 | TASK-193 | Dialogs and windows: roles, focus, announcements | 988bcd5 |
| A11Y-6 | TASK-190 | The 303 step editor, spoken | 7c09ffa |
| A11Y-7 | TASK-191 | The drum grids, navigable and readable | d05000d |
| (new) | TASK-210 | Shift+Space as the accent key on a TR-909 grid | 7267c55 |
| A11Y-10 | TASK-195 | A mod chooser with a filter | 6cc85ab |
| A11Y-12 | TASK-196 | Documentation of screen reader mode | df4d002 |
| A11Y-13 | TASK-197 | The NVDA pass by the user: all 22 checklist items work | (backlog) |
| (new) | TASK-212 | Menu bar: the focus returns where it was when the bar is left | 3f3fdc7 |
| (new) | TASK-211 | Options > Pitch Keys Make Notes: a 303 pitch key turns a rest into a note | b8adbc9 |
| (new) | TASK-201 | Mods menu: the playing kit's entry read as "checked" | 78129ba |
| (new) | TASK-204 | The Export dialog keeps the format the user picks | 771ce7c |
| (new) | TASK-213 | Knobs and faders: an arrow key moves to the next value that sounds different | 3b5c2bd |
| (new) | TASK-214 | A run log of each start; a failed start leaves a trace | 2b76d08 |
| (new) | TASK-215 | A Windows debug kit: OpenRB-338 with symbols and tools/win_diag.ps1 | 91c1e8d |
| (new) | TASK-216 | A system_dialog test that failed in debug builds (test only) | 7db8369 |
| (new) | TASK-217 | Windows: DirectX 12 before Vulkan (Intel's Vulkan driver crashed the start) | 4c75ec2 |
| (new) | TASK-219 | Debug kit: win_diag.cmd, so the execution policy need not be typed | 7541709 |
| (new) | TASK-218 | The user's check: the Windows 10 PC starts OpenRB-338, 3 runs of 3 | (backlog) |

Decisions made along the way, all Done: TASK-198 (a local editor only), TASK-202, TASK-203,
TASK-205, TASK-207, TASK-208, TASK-209 (section 1.1).

### To do

| Task | What | Priority | Who |
|---|---|---|---|
| TASK-199 | Screen reader detection on macOS (VoiceOver) and Linux (AT-SPI); on hold | low | agent |
| TASK-206 | A system file dialog that cannot open reads as a cancel (Linux); on hold | low | agent |

The milestone is done when the m-7 tasks above are done (Part 3, "Done when"); TASK-199 and
TASK-206 are on hold by the user's choice (2026-11-07), since they can be checked only on macOS
and Linux, and the fork builds and ships for Windows only for now (the user, 2026-11-07: "We
will not have a linux, mac or currently at least anything other than windows"). In the backlog
m-7 stands at 39 of 41 done.

Also open, outside the backlog: the by-ear checks of the follow-ups, NVDA checklist items 23 to
27 (TASK-212, TASK-211, TASK-201, TASK-204, TASK-213; section 2.2). Their tests pass; the
user has not marked them yet.

## The milestone

**m-7: screen reader accessibility (NVDA first).** A blind musician can use OpenRB-338 with NVDA
on Windows, everything the panel does: play and stop, set every knob, fader and switch, choose
patterns, program the 303s and the drum machines and read their patterns back, record in song
mode, open, save and export songs, choose mods, and read every message and window. All of it
from the keyboard, with speech that says what each control is, what its value is and what a key
just did, and stays quiet while the music plays unless asked.

**Scope.** The desktop UI (`crates/orb-ui`), on Windows with NVDA first; the same accessibility
tree serves Narrator and JAWS (UI Automation), VoiceOver (macOS) and Orca (Linux) through
AccessKit, with less testing. Not in scope: the engine (no change at all: the rendered audio
stays byte for byte the original's), the `orb` command line tool (already usable with a screen
reader), MIDI, and any web version: the fork is a local editor only (the user, 2026-11-05,
TASK-198; upstream's m-6 is archived here).

---

## Part 1: Finished

### 1.1 Decisions

Each decision is a Done task (`--type spike -l decision,ui -a @dengopaiv`) whose final summary
records the choice and the date; the options that were weighed are in its description.

**The six up-front decisions (2026-11-05).**

- **A11Y-D1 = TASK-177, how screen reader mode turns on.** Automatically when the system reports
  a screen reader, with the Options menu item "Screen Reader Mode" (Automatic, On, Off), kept in
  `ui.toml`. (Not "always on": that would change the keys for every user.)
- **A11Y-D2 = TASK-178, Space and Enter on a focused control.** In screen reader mode Space
  presses or toggles the focused control (a button, a toggle, a switch, a step of a drum grid)
  and is no longer Play/Stop; F5 is Play/Stop. Enter keeps the original's meaning (the 303's
  Step in Program Synth mode, else Play; refined by TASK-207 below). With the mode off, Space is
  Play/Stop as before.
- **A11Y-D3 = TASK-179, where a machine's mixer channel is.** In the machine's own section: each
  machine's section ends with its mixer channel.
- **A11Y-D4 = TASK-180, file dialogs.** In screen reader mode the system's file dialogs; the
  in-app browser's "Use as songs / mods folder" become Options menu items. With the mode off
  the in-app browser is unchanged.
- **A11Y-D5 = TASK-181, how values are spoken.** In physical units (Hz, ms, dB), "as precise as
  possible as we infer them from the code", each marked by how exact it is; a quantity with no
  physical unit as a percent of its range; switches, pan and the displays in their own words.
  The node's numeric value stays 0..127.
- **A11Y-D6 = TASK-182, the key map of screen reader mode.** The function keys: F1 the key help,
  F2 / F3 / F4 read the pattern / the transport / the levels, F5 Play/Stop, F6 / Shift+F6 the
  next / previous section, F7 and F8 free, F9 Record (Insert too), F10 the menu bar (and Alt
  alone), F11 Loop, F12 song / pattern mode ("transport on F9 to F12", away from F6). Tab /
  Shift+Tab the next / previous control. Up, Down, Page Up, Page Down, Home, End change the
  focused value. Ctrl+Up / Ctrl+Down the original's row focus. Ctrl+Tab / Ctrl+Shift+Tab the
  original's 303 tap (Tab / Shift+Tab). Every other key of the original keeps its meaning,
  whatever has the focus. (Left / Right: see TASK-209.)

**Decided along the way.**

- **TASK-198 (2026-11-05): a local editor only.** No web version, nothing that needs a server
  or a network service; upstream's m-6 archived (AGENTS.md "This fork").
- **TASK-202 (found in TASK-184): the panel's keys while a menu is in use.** All panel keys
  wait while a menu is open or a menu title has the focus, in both modes (the one exception
  to "mode off behaves as upstream").
- **TASK-203 (found in TASK-194): a name the system dialog did not confirm.** When the path
  written differs from the one the dialog returned (an extension added) and that file exists,
  OpenRB-338 asks "Replace <name>?", No by default.
- **TASK-205 (found in TASK-187): Page Up / Page Down / Home / End on a spin button.** Page
  Up / Page Down are 10 clicks; Home / End go to the end of the range only where it is short
  (pattern length, delay steps, PCF pattern), not on tempo and the transport positions.
- **TASK-207 (found in TASK-188): Enter outside a 303's section.** Enter follows the section of
  the focused control: in a 303's section that 303's Step (Program Synth mode), anywhere else
  Play. The pitch and drum keys still go to the row focus.
- **TASK-209 (found in TASK-190): the step editor during playback, and its arrows.** The editor
  stays on the pattern being edited; following playback is the setting Options > Step Editor
  Follows Playback, off by default, and it never speaks. Up / Down move the step, Left / Right
  do nothing on the step editor. The same for the drum grids (where Left / Right move the step
  and Up / Down the instrument).
- **TASK-208 (found in TASK-191): Alt+Space opens Windows' system menu.** Option B: Shift+Space
  is an accent key on the TR-909 grid as well, Alt+Space stays (the user, 2026-11-06, "Add
  shift+space"; built as TASK-210).
- **TASK-195's own choices (2026-11-07):** Ctrl+M opens the mod chooser; the filter and the list
  are two focus stops; the list is a tree with the variants under their name; Ctrl+I shows
  Mod Info of the selection.

### 1.2 The work

Each item: what it gives the user now, where it is described, and what moved to the NVDA pass.
With screen reader mode off, every item leaves the app as upstream (but TASK-202).

- **A11Y-0 = TASK-183, test harness and probe.** `crates/orb-ui/tests/common/a11y.rs` builds the
  accessibility tree of the app headless (no window, no GPU, on CI's three systems), finds nodes,
  reads names, roles, values and states, and sends a screen reader's actions; the baseline
  test `a11y_baseline.rs` keeps the counts. `tools/uia_tree.ps1` reads the real window through UI
  Automation (docs/accessibility.md 6.3).
- **A11Y-1 = TASK-185, screen reader mode.** Options > Screen Reader Mode (Automatic, On, Off),
  `screen_reader` in `ui.toml`; Automatic follows Windows' screen reader flag
  (`SPI_GETSCREENREADER`), asked at start and when the window regains the focus. Other systems:
  TASK-199. Checked by hand that Automatic turns on with NVDA; the other half is checklist
  item 19.
- **A11Y-11 = TASK-184, the menu bar from the keyboard.** F10, or Alt pressed and released
  alone, focuses File; the arrows walk the bar and the menus, Enter chooses, Escape closes; the
  Options menu's checkboxes and radios carry their state; the Mods menu's entries too, since
  TASK-201. (docs/ui_spec.md section 18.)
- **A11Y-2 = TASK-186, names, roles, values.** Every control of the panel has a spoken name, its
  section's name, a role and its value or state; LEDs, meters and click areas take no focus.
  (docs/ui_spec.md 20.)
- **A11Y-9 = TASK-194, system file dialogs.** In screen reader mode Open, Save As, Export and
  Mods > Choose file use the system's dialog; Options > Set Songs Folder... / Set Mods Folder...
  replace the browser's folder buttons; TASK-203's question. The hand check is checklist
  item 20. Follow-ups: TASK-204, TASK-206.
- **A11Y-14 = TASK-192, physical units.** Every knob and fader is spoken in a unit derived from
  the engine's own tables ("1.24 kilohertz", "350 milliseconds", "minus 6.5 decibels"), each
  mapping marked [C], [D] or [G] in docs/control_units.md; Options > Speak Raw Values adds the
  0..127 number. The engine is unchanged (corpus golden test).
- **A11Y-3 = TASK-187, controls a reader can drive.** Knobs and faders are sliders (Up/Down 1,
  Page Up/Down 8, Home 0, End 127, and NVDA's own slider commands); each display with its arrows
  is one spin button (tempo, bar, loop start and length, pattern length, delay steps, PCF
  pattern; TASK-205's keys); each row's 8 pattern and 4 bank buttons are one pattern selector
  ("Pattern B3"). Every change goes through the same panel call as a click, so the engine and
  recording hear the same. (docs/ui_spec.md 20.1.)
- **A11Y-4 = TASK-188, sections, focus order, keys.** The sections (Transport, Bass Line 1 and 2,
  TR-808, TR-909, each with its pattern row and mixer channel, Master and compressor, PCF, Delay,
  Distortion) are named groups; F6 / Shift+F6 and Tab / Shift+Tab walk them; the row focus
  follows the machine section entered; D6's key map, D2's Space and TASK-207's Enter. (docs/ui_spec.md
  20.2 and section 10.)
- **A11Y-5 = TASK-189, announcements and read-outs.** A polite live region speaks the status
  messages, a song opened, a mod chosen, the keyboard mode, and the transport keys; nothing is
  spoken by itself during playback (a test). F2 reads the pattern of the focused section, F3
  the transport, F4 the levels. (docs/ui_spec.md 20.3.)
- **A11Y-8 = TASK-193, dialogs and windows.** Keyboard Shortcuts, File > Info, About and Mods >
  Info are dialogs with their title, the error box an alert, the unsaved-changes prompt a dialog
  with the focus on Save; the focus moves in when one opens and returns when it closes; their
  lines can be read one by one. (docs/ui_spec.md 20.4.)
- **A11Y-6 = TASK-190, the 303 step editor.** In each 303 section a "Step editor" whose value is
  the edit step and its contents ("Step 5 of 16: C sharp, octave up, accent, slide"); every
  programming key's result is spoken; Up / Down (and Enter / Backspace) move the step, Home /
  End to the first / last; F2 reads the whole pattern compactly; TASK-209's setting. A how-to is
  in docs/ui_spec.md 20.5. A rest stays a rest when a pitch key sets its note, as in the
  original: TASK-211 adds the option Pitch Keys Make Notes.
- **A11Y-7 = TASK-191, the drum grids.** In each drum section a grid of the instruments by the
  steps: Up / Down the instrument, Left / Right the step, Space sets the step as a click does
  (on the TR-909 with Ctrl, Alt, Ctrl+Alt, and Shift+Space from TASK-210); each cell reads
  "Bass drum, step 5, on"; F2 reads the instruments that have steps; TASK-209's setting covers
  the grids. A how-to is in docs/ui_spec.md 20.6.
- **TASK-210, Shift+Space.** On a TR-909 grid Shift+Space sets an accent without the system
  menu that Alt+Space also opens (TASK-208).
- **A11Y-10 = TASK-195, the mod chooser.** In screen reader mode, Mods > Choose Mod... or Ctrl+M
  opens a dialog with a filter field and a tree of the Mods menu's entries in the menu's order
  ("Standard ReBirth" first), each read with "variants: N", "has demo", "current"; Right / Left
  open and close a name's variants, Enter loads (a name: its preferred variant), Ctrl+I shows
  Mod Info of the selection without loading it, Escape closes. It opens in under 5 ms with the
  311 entries of the rb-mods collection. (docs/ui_spec.md 20.7.)
- **A11Y-12 = TASK-196, documentation.** The README's "Using OpenRB-338 with a screen reader";
  docs/ui_spec.md section 20 and section 10 checked against the code; Help > Keyboard Shortcuts
  complete for the mode, with tests (the keys of `keys::map_screen_reader` by behavior, the
  keys handled elsewhere by name); the measurement below.
- **TASK-212, the focus after the menu bar.** Asked by the user after the NVDA pass. In screen
  reader mode, leaving the menu bar (Escape, an item that opens no window, a second F10 or lone
  Alt) gives the focus back to the panel control that had it; a window opened from a menu
  returns it to that control when it closes. With the mode off, the menu bar is as before.
  (docs/ui_spec.md 18.2; NVDA checklist item 23.)
- **TASK-211, Pitch Keys Make Notes.** Asked by the user after checklist item 8. An Options item,
  off by default and only in screen reader mode: a pitch key on a 303 rest step also makes it a
  note (the engine gets the rest cleared, then the note); a note only changes its pitch; `-`,
  `=` and Clear as before. (docs/ui_spec.md 20.5; NVDA checklist item 24.)
- **TASK-201, the Mods menu's mark.** The entries (Standard ReBirth, each mod, each variant
  file) are checkable menu items, so the playing kit's entry reads "checked"; the variants
  submenu of a playing variant is marked too. Both modes; the menu looks and acts as before.
  (NVDA checklist item 25.)
- **TASK-204, the Export format.** In screen reader mode the system Export dialog lists the
  last format exported first (the default extension on Windows) and gives a name typed without
  an extension that format; it is kept in ui.toml (`export_format`) across restarts. The in-app
  browser is unchanged. (NVDA checklist item 26.)
- **TASK-213, a key press that always changes the value.** Found by the user: on many knobs
  neighboring positions have the same spoken value (tr808.bd_tone: 8 values over 128
  positions), so an arrow press was silent. In screen reader mode Up / Down and NVDA's
  Increment / Decrement go to the next position whose spoken value differs, Page Up / Page Down
  at least 8 positions and on to a different value; where no different value lies that way,
  nothing moves. With Speak Raw Values on, one position per press as before. (NVDA checklist
  item 27.)

Also done along the way, outside m-7: TASK-176 (milestone m-5), the third-party songs test
compares song paths in NFC (commit 4d76af3): 1268 songs equal to the original's renders.
And TASK-200 (no milestone): crates/orb-ui/tests/crash.rs no longer fails in a Windows release
build, whose crash log cannot name `orb_ui::crash` without debug info; there it checks for a
`std::panicking` frame, everywhere else still for `orb_ui::crash`. `cargo test --release -p
orb-ui` now passes in full.

**The start on Windows 10 (TASK-214 to TASK-219, 2026-11-07).** OpenRB-338 did not start on
the user's Windows 10 PC (Intel HD Graphics 530). TASK-214 made every start write a run log
(`run-YYYYMMDD-HHMMSS.log`: the stages, the libraries' messages, a watchdog, a native crash
line), TASK-215 a debug kit (a build with symbols, `tools/win_diag.ps1` that starts the program
several times and writes a report), and TASK-219 `tools/win_diag.cmd`, so the user need not
type PowerShell's execution policy. The kit's report showed a crash in Intel's Vulkan driver,
3 starts of 3; TASK-217 makes OpenRB-338 prefer DirectX 12 on Windows unless `WGPU_BACKEND` is
set (`crates/orb-ui/src/graphics.rs`), and with it the user's PC started 3 runs of 3
(TASK-218). TASK-216 fixed a test that failed in debug builds only. All of it is in
docs/building.md: "The run log, for a start that fails", "The graphics on Windows: DirectX 12
first" and "Debugging OpenRB-338 on another Windows PC".

### 1.3 What the work measured

The real window, read through UI Automation with `tools/uia_tree.ps1` (docs/accessibility.md
1.1 and 6.3):

| | Before (2026-11-05, cbca449) | Mode off (2026-11-07) | Mode on (2026-11-07) |
|---|---|---|---|
| Elements | 256 | 198 | 593 |
| Can take the focus | 254, of them 4 named | 194, all named | 198, all named |

Before the work, Tab walked through 250 silent stops; now every stop has a name. With the mode
on, the 593 elements include the 384 cells of the two drum grids.

---

## Part 2: To do

### 2.1 The NVDA pass (A11Y-13 = TASK-197): done

The user (@dengopaiv) went through the checklist below with NVDA on Windows and the release
build on 2026-11-07: every item works ("It worked"). TASK-197 is Done. Found on the way, as
wishes rather than defects: TASK-211. The checklist stays here as the by-ear test of screen
reader mode, to repeat after a change.

### 2.2 The NVDA checklist (A11Y-13)

With NVDA (note its version) and the release build, screen reader mode on (by itself, A11Y-1).
Marks: (x) works, ( ) fails or not yet confirmed, no mark: not yet tried.

1. Start the program: the window title is spoken, the focus is on the first control of the
   Transport section. (x)
2. F6 through all sections and back with Shift+F6: each section's name is spoken. (x)
3. Tab through the Transport section: each control's name, role and value are spoken. (x)
4. Change the tempo with Up and Down: the new tempo is spoken; Page Up and Page Down. (x)
5. F5 plays and stops; nothing is spoken by itself while it plays. Space on a focused toggle
   toggles it and does not play. F9 Record, F11 Loop and F12 mode are spoken when pressed.
   (The user's note: Loop is F11, F12 is song and pattern; recording is in song mode.) (x)
6. F3 reads the transport while playing and while stopped. (x)
7. In Bass Line 1: set the cutoff with the arrows and with NVDA's own slider commands; set the
   waveform; mute channel 1; choose pattern B3. (x)
8. Program a 303 pattern with the keys alone; each step is spoken; F2 reads it back
   (TASK-190's deferred check, docs/ui_spec.md 20.5). On the focused "Step editor": each
   programming key's new state is spoken once, by the value change (`-`, `P`, `[`, a new note),
   not twice; the same note pressed again is spoken (by the announcer); Up, Down, Home and
   End speak the new step, Left and Right nothing; in pitch mode (Tab three times, Space) a
   pitch key speaks the step it set ("Step 1: E") and then the next step's value, both, in
   that order, neither cut off. Play a song in song mode with the editor focused: nothing is
   spoken when the song moves to another pattern, with Options > Step Editor Follows Playback
   off and on; F2 says "Step editor on pattern ..." while off. (x)
9. In the TR-808: walk the grid, set a bass drum on steps 1, 5, 9, 13, F2 reads it back; same
   on the 909 with an accent and a flam. (TASK-191 checked the tree headless: the grid's active
   cell, its name after each key, F2's text.) By ear: Tab onto the grid speaks "Steps" and the
   cell ("Bass drum, step 1, off"); each Up, Down, Left and Right speaks the new cell; each
   Space speaks the cell's new state (its name changes while it keeps the focus); Ctrl+Space,
   Alt+Space and Ctrl+Alt+Space on the 909 give a note, an accent and a flam (Alt+Space also
   opens the system menu, TASK-208); Shift+Space gives an accent and no menu (TASK-210). While
   a song plays in song mode with the focus on a grid, nothing is spoken when the machine moves
   to another pattern, with Options > Step Editor Follows Playback off and on; F2 says "Grid on
   pattern ..." while off. (x)
10. Record in song mode: a pattern change and a knob move are recorded; play them back. (x)
11. File > Open in the system dialog; open a song from rb-songs; its name and mod are spoken.
    (x)
12. Choose a mod with the mod chooser, with the filter; Mods > Info reads the mod's text;
    Mods > Demo Song. (TASK-195 checked the tree headless: roles, names, levels, expanded,
    the keys, the kits.) By ear: Ctrl+M speaks the dialog's name "Choose Mod" and the filter
    ("Filter mods", edit); typing narrows the list and the count line is readable; Down moves to
    the list and NVDA speaks "Mods, tree view" and the selected item with its level and
    "collapsed" for a mod with variants ("Name, variants: 3, collapsed, 5 of 311"); Right
    speaks "expanded" and then the first file; Left goes back; Up and Down speak each item
    with "has demo" and "current"; Enter chooses and "Mod X" is spoken; Ctrl+I opens Mod Info
    and Escape returns to the list; Tab and Shift+Tab move between the two stops; Escape closes
    and the focus is back where it was. (x)
13. File > Info: read and edit the song's title. (x)
14. Save, Save As, and Export Loop as Audio File: the status messages are spoken. (x)
15. Trigger an error (open a file that is not a song): the error box is spoken and OK closes it.
    (x)
16. Close with unsaved changes: the prompt is spoken, the focus is on Save. (x)
17. Help > Keyboard Shortcuts: the list can be read line by line. (x)
18. Turn the mode off: the original keys behave as before (Tab taps the 303 while playing). (x)
19. With NVDA closed (no screen reader running), Options > Screen Reader Mode set to Automatic
    leaves the mode off (the other half of A11Y-1's check; A11Y-1 verified only that Automatic
    turns the mode on with NVDA running). (x)
20. In screen reader mode, Save As, Export Loop as Audio File, Mods > Choose file and
    Options > Set Songs Folder / Set Mods Folder open the system dialog, NVDA reads it, and a
    file saved or exported through it is written as through the browser (TASK-194's deferred
    hand check; item 11 covers Open). Type a name with a dot and no extension (`v1.2 mix`) over
    an existing `v1.2 mix.rbs`: the question "Replace v1.2 mix.rbs?" is spoken, No is the
    default. (x)
21. TASK-189's announcements are spoken without moving the focus: a status message (Save),
    a song opened (its name and mod), a mod chosen, the keyboard mode (Ctrl+F, Ctrl+G), the
    transport keys (F5, F9, F11, F12, Enter, Space on Play and Stop, + and -, a pattern key
    in Select Patterns mode); and the read-outs F2, F3 and F4. Press F3 twice in a row: the
    same message is spoken both times. (TASK-189 checked only that the window raises a UI
    Automation LiveRegionChanged event for each, a repeated message included.) (x)
22. TASK-193's windows (docs/ui_spec.md 20.4), each opened from the panel with the mode on:
    Keyboard Shortcuts, File > Info, Help > About and Mods > Info are announced as dialogs
    with their title, and NVDA reads the first line (the content) as the focus enters; Tab
    reads About's, Keyboard Shortcuts' and Mod Info's lines one by one ("Ctrl+N: File > New",
    "Name: Standard ReBirth"); Escape closes each and the focus returns to the control that
    had it. The error box (item 15) is announced as an alert, "Error", with its message read
    when the focus lands on OK (if NVDA does not read the message, the message needs
    `App::announce`); Escape or Enter closes it and the focus returns, also to the window
    under it. The unsaved-changes prompt (item 16) is read with its question and the focus
    is on Save. (x)
23. TASK-212's focus return (docs/ui_spec.md 18.2), in screen reader mode with the focus on a
    panel control (a knob, Play): F10 (or Alt alone), then Escape: NVDA reads the control
    again. F10, Down, one Escape: the same. F10 and F10 again, and Alt and Alt
    again, on a title and with a menu open: the menu closes and NVDA reads the control. Options
    > Step Editor Follows Playback chosen with Enter: the control is read. Help > Keyboard
    Shortcuts: the window is read, and Escape returns to the control, not to "window". ( )

24. TASK-211's option (docs/ui_spec.md 20.5), in screen reader mode on a 303's step editor on
    an empty pattern: with Options > Pitch Keys Make Notes off (the default), the pitch key `V`
    answers "Step 1: rest, D" and the step stays a rest. Turn the item on (Enter on the
    checkbox; NVDA reads its state): `V` answers "Step 1: D", the editor reads "Step 1 of 16: D"
    with no "rest", and F5 plays the step. `-` makes it a rest again. In pitch mode each key
    makes its step a note, the step it set is said, and F5 plays all of them. A step that is
    already a note only changes its note. Clear makes every step a rest. The item is saved:
    restart, and it is still on. ( )

25. TASK-201's marks in the Mods menu, with a mod playing (or the built-in kit): open the Mods
    menu and walk it with Up and Down. NVDA reads the playing kit's entry as a checked menu
    item ("Zed, menu item, checked") and every other mod entry as not checked; the Standard
    ReBirth entry is checked when the built-in kit plays. A mod of a shared name: the name's
    entry (a submenu, read as a button) and, inside it, the playing file read checked. ( )
26. TASK-204's export format, in screen reader mode with a song in song
    mode: File > Export Loop as Audio File opens the system dialog with WAV as the type; save
    as `a.flac` (pick FLAC in the type list or type the extension); export again: the type
    list offers FLAC first and NVDA reads it as the file type; a name typed without extension
    (`b`) is saved as `b.flac`. Restart the program: the dialog still offers FLAC first. With
    the mode off the in-app browser starts on WAV as before. ( )
27. TASK-213's jump to the next value that sounds different, in screen reader mode (Speak Raw
    Values off): focus the TR-808 Bass drum tone and press Up repeatedly from 0: every press
    is spoken with a new value ("42.9 hertz", then the next one), and Down comes back the same
    way; Page Up lands on a different value; in the top group of values (120 to 127) Up and Page Up
    do not move.
    The 303's Cutoff moves one position per press. Turn Options > Speak Raw Values on: each
    press moves one position and speaks the number. With the mode off nothing of this applies.
    ( )

### 2.3 Open tasks

Each is in the backlog with its description and acceptance criteria. The user's round of
2026-11-07 finished TASK-212, TASK-211, TASK-201, TASK-204, TASK-200 and TASK-213 (found on
the way), one agent at a time; TASK-199 and TASK-206 are on hold (they can be checked only on
macOS and Linux). The by-ear checks of the round are checklist items 23 to 27, not yet marked
(2026-11-09). The Windows 10 round that followed (TASK-214 to TASK-219, section 1.2) needed no
NVDA check: the user's check was that the program starts (TASK-218).

- **TASK-199 (low): detection on macOS and Linux.** Automatic turns the mode on only on Windows
  today; VoiceOver and AT-SPI detection.
- **TASK-206 (low): a system dialog that cannot open (Linux).** Without the XDG portal, rfd
  returns nothing and Open seems to do nothing; tell the user and fall back to the browser.

---

## Part 3: Reference

### Ground rules of every task of the milestone

- With screen reader mode off, the app behaves exactly as upstream, key for key, except where
  the user decided otherwise: the panel's keys wait while a menu is in use (TASK-202). Outside
  the keys, two changes of the Windows 10 round apply in both modes: the run log of each start
  (TASK-214) and DirectX 12 before Vulkan on Windows (TASK-217). The
  existing tests of `crates/orb-ui/tests` (keyboard.rs in particular) pass unchanged.
- Every accessible action goes through the same `model::Panel` method as the mouse or the
  original's key (`press`, `arrow`, `set_knob`, `set_fader`, `set_switch`,
  `select_instrument`, `set_step808`, `set_step909`, `tap_303`, `tap_drum`), so the events
  the engine hears, recording in song mode included, are the same.
- The accessible layer lives in `crates/orb-ui/src/a11y/` (and `mod_chooser.rs`), with small
  hooks elsewhere.
- Nothing is announced by itself during playback (position, meters, LEDs, running lights,
  a pattern change by the song); what the user does is confirmed.
- Each task adds headless tests of the accessibility tree for what it builds, updates
  docs/ui_spec.md and Help > Keyboard Shortcuts (`crates/orb-ui/src/key_help.rs`) for what it
  changes, and checks the frame cost with `crates/orb-ui/tests/frame_cost.rs`.
- A check that needs NVDA by ear is not done by the agent: it is narrowed to what tests show,
  and the by-ear part goes to the NVDA checklist (2.2).
- Lines of Rust at most 100 columns, `cargo clippy --all-targets -- -D warnings` clean.

**Done when.** All tasks of m-7 are Done (the NVDA pass, TASK-197, is).

### Risks and how the plan meets them

- **egui changes.** AccessKit's integration in egui moves between versions (0.37 already
  deprecated `WidgetType` and `Id::new`). The accessible layer stays in one module and talks to
  egui through a few calls; an egui upgrade (upstream's) is checked with the harness's tests.
- **What NVDA does with a custom tree.** Live regions, value changes of a focused slider and
  grid navigation are spoken differently by NVDA, JAWS and Narrator. Every such behavior is in
  the NVDA checklist.
- **Frame cost.** The tree is rebuilt each frame; the panel repaints at 30 frames per second
  while playing (task-153). Each task checked `frame_cost.rs`; if the cost grows, the nodes that
  do not change are cached.
- **Upstream.** With the mode off nothing changes, so rebasing on upstream stays easy and the
  work can be offered upstream as it is.

  Checked on 2026-11-09: upstream's main is still cbca449 (its tag v2026.1103.0917), the
  commit the fork started from, so there is nothing to merge. Open on upstream:
  - Pull request #4 ("Save engine tracks for all songs", opened 2026-11-09 by a contributor, not
    merged) is an implementation of TASK-143, which is open in this backlog too: every Save
    writes the engine's tracks, so an unedited save no longer gives back the file byte for
    byte. It changes `document.rs`, `link.rs`, `menu.rs` (`App::song_bytes` takes `&mut self`;
    `composed_song` becomes `song_for_save`), tests and docs/ui_spec.md's Save sections. Merged
    into the fork's main it gives no text conflict (`git merge-tree`), and the fork adds no
    caller of the changed functions; it was not built or tested here.
  - Issue #2 asks whether OpenRB-338 can work without `assets/Standard.rbm` (ReBirth's own drum
    samples and panel pictures, which Debian cannot redistribute). It applies to the fork as
    well, whose repository and Windows zips carry the same file; no answer upstream yet.
  - Issues #1 and #3 (a VST, LV2 or CLAP plug-in) and #5 (no sound on macOS) do not touch the
    fork: it is a local, Windows-only program.
- **Units that are only estimates.** docs/control_units.md marks each unit [C], [D] or [G], and
  Options > Speak Raw Values gives the exact stored number next to it.
- **Space no longer plays in screen reader mode.** A habit from ReBirth (D2). F5 plays, Enter
  still plays outside a 303's section, and Help > Keyboard Shortcuts says so.
- **Speech over music.** The rule "nothing by itself during playback" is a test (A11Y-5), not a
  hope.

### How the plan became tasks

The plan was written here on 2026-11-05, before the Backlog.md CLI was available, as decisions
(A11Y-D1..D6) and work (A11Y-0..A11Y-14), then created in the backlog with the CLI (1.55.3)
as it stood (commit 2611c80): milestone m-7, the decisions as Done spikes, the work as tasks
with ordinals from 200000. Since then the backlog is the source of truth, and the plan ids
remain only as names (docs/ui_spec.md section 20 refers to them):

- Decisions: A11Y-D1 = TASK-177, A11Y-D2 = TASK-178, A11Y-D3 = TASK-179, A11Y-D4 = TASK-180,
  A11Y-D5 = TASK-181, A11Y-D6 = TASK-182.
- Work: A11Y-0 = TASK-183, A11Y-11 = TASK-184, A11Y-1 = TASK-185, A11Y-2 = TASK-186,
  A11Y-3 = TASK-187, A11Y-4 = TASK-188, A11Y-5 = TASK-189, A11Y-6 = TASK-190,
  A11Y-7 = TASK-191, A11Y-14 = TASK-192, A11Y-8 = TASK-193, A11Y-9 = TASK-194,
  A11Y-10 = TASK-195, A11Y-12 = TASK-196, A11Y-13 = TASK-197.

The full text each task was created with (its why, what, references and acceptance criteria) is
in the backlog task and in this document's history (`git show 3e2f7d2:docs/accessibility_plan.md`).

On Windows, a script that passes multi-line text to the CLI must run `node .../backlog.md/cli.js`
directly: through npm's `backlog.cmd` shim, cmd.exe cuts an argument at its first newline and
drops the rest of the command line.

**How the work is run** (the user's rules, AGENTS.md "This fork" and the session memory): one
agent at a time, and the user is asked before a task starts; reviews look at code and test
defects only; a reviewed branch is merged and pushed to the fork, and its worktree and branch
are removed.
