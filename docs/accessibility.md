# Screen reader accessibility: what the program does

This document describes what OpenRB-338's screen reader mode does, part by part, as of
2026-11-28, and marks each part by its state:

- **Does, heard (N):** built, covered by tests, and heard working with NVDA by the user
  (checklist item N of [accessibility_plan.md](accessibility_plan.md), Part 3; the passes of
  2026-11-07 and 2026-11-28).
- **Does, not yet heard (N):** built and covered by tests; the by-ear check, item N, waits for
  the user (the plan's Part 2.1).
- **Fault heard (N): TASK-NNN:** built, but the user heard it go wrong; the bug task fixes
  it. A fault found outside the checklist's items has no N.
- **Will do (TASK-NNN):** an open task in the backlog.
- **Not built:** no task yet (the plan's Part 2.4 lists them for the user to decide).
- **Not tried** or **Not checked:** it may work, since it reads the same tree, but no one has
  tried it (other screen readers, braille; the plan's Part 2.4).

The first target is NVDA on Windows (the user's choice). AccessKit, the layer the mode is built
on, also serves Narrator and JAWS through UI Automation, VoiceOver on macOS and Orca on Linux,
but only NVDA on Windows is tested, and the fork is built for Windows only (the user,
2026-11-07).

Where else to read:
- The behavior in detail, with the names and the code, is in docs/ui_spec.md section 20.
- The guide for users is the README's "Using OpenRB-338 with a screen reader".
- What is done and what is to do, task by task, is the plan,
  [accessibility_plan.md](accessibility_plan.md).
- The program's measured state before the work, why the panel was silent, and the options
  weighed are in section 7, the history.

The section numbers 1.1, 3.2 to 3.6, 6.2 and 6.3 are cited by the code's comments and the
tests, so they stay as they are.

## 1. Where it stands

### 1.1 Measured

The test `crates/orb-ui/tests/a11y_baseline.rs` holds the headless counts of the accessibility
tree (`cargo test --release -p orb-ui --test a11y_baseline -- --nocapture`). On main of
2026-11-28:

| | Mode off | Mode on |
|---|---|---|
| Headless nodes | 205 | 600 |
| of them with a name | 198 | 593 |
| that can take the focus | 196 | 200 |
| with an unknown role | 0 | 0 |

With the mode on, the extra nodes are the nine sections (groups), the two drum grids and their
2 x 12 x 16 cells. The two 303 step editors and the two grids take the focus too.

The real window, read through UI Automation with `tools/uia_tree.ps1` (section 6.3), was last
measured on 2026-11-07, before the upstream merges:

| | 2026-11-05, before the work | 2026-11-07, mode off | 2026-11-07, mode on |
|---|---|---|---|
| Elements | 256 | 198 | 593 |
| that can take the focus | 254 | 194 | 198 |
| of those, with a name | 4 | 194 | 198 |
| Unnamed "Custom" elements | 250 | 0 | 0 |

Before the work, Tab stopped on 250 silent places. On 2026-11-07 it stopped on 198 places that
all say what they are, and the user's NVDA pass heard them reading as they should. The
real-window count has not been repeated since the merges of 2026-11-18 and 2026-11-26 (the
plan's Part 2.4).

### 1.2 At a glance

| Part | State | Section |
|---|---|---|
| Screen reader mode, Automatic on Windows | Does, heard (1, 19) | 3.1 |
| Automatic on macOS and Linux | Will do (TASK-199, on hold) | 3.1 |
| Nine sections, F6 and Tab | Does, heard (2, 3) | 3.2 |
| The row focus spoken | Not built | 3.2 |
| Knobs and faders in physical units | Does, heard (7) | 3.3 |
| An arrow key always changes what is heard | Does, heard (27) | 3.3 |
| Spin buttons and pattern selectors | Does, heard (4, 7) | 3.3 |
| The 303 step editor | Does, heard (8) | 3.3 |
| Pitch Keys Make Notes | Does, heard (24) | 3.3 |
| The drum grids | Does, heard (9) | 3.3 |
| The key map of the mode | Does, heard (5) | 3.4 |
| The numpad apart from the main keys | Does, heard (28) | 3.4 |
| Announcements and the read-outs F2 to F4 | Does, heard (6, 21) | 3.5 |
| The menu bar | Does, heard (13 to 17) | 3.6 |
| Left and Right from menu to menu | Does, heard (36) | 3.6 |
| A question opened from the menu | Does, heard for File > New (38); Initialize Song not yet | 3.6 |
| Menu items that are off | Fault heard (26, 29, 38): TASK-250 | 3.6 |
| The focus back after the menu bar | Does, heard (23) | 3.6 |
| The Mods menu's checked entry | Does, heard (25) | 3.6 |
| System file dialogs | Does, heard (11, 20) | 3.6 |
| The Export format kept | Does, not yet heard (26) | 3.6 |
| The mod chooser | Does, heard (12) | 3.6 |
| Dialogs, the error box, the unsaved-changes prompt | Does, heard (15, 16, 22) | 3.6 |
| The Initialize Song question | Does, not yet heard (29) | 3.6 |
| Options > Audio Output | Does, not yet heard (31, 35, 37) | 3.6 |
| Audio Output on ASIO's System default | Fault heard (31): TASK-251 | 3.6 |
| The 303 Pattern Editor windows | Does, heard (33) | 3.7 |
| The 808 and 909 Pattern Editor windows | Does, heard (34) | 3.7 |
| The classic look | Does, heard (32) | 3.8 |
| The names 808, 909, 303-1, 303-2 | Does, heard (30) | 3.2 |
| With the mode off, as upstream | Does, heard (18) | 3.9 |
| Narrator, JAWS | Not tried | 3.10 |
| Braille | Not checked | 3.10 |
| Meters in decibels | Not built | 3.5 |
| A MIDI input port | Not built (no task; the MIDI layer is kept for it) | 3.10 |

## 2. How it is built

The panel keeps its look. Screen reader mode adds an accessibility tree built from the panel
model: sections, one node per musical thing, pattern editors that speak, and announcements. A
mode resolves the keys that collide with a screen reader (section 7.3).

The principles:
- **One accessible node per musical thing**, not per painted part. The tempo is one spin
  button, not a display and two arrow buttons; a row's 32 patterns are one selector, not 12
  buttons.
- **Quiet while the music plays.** The playing position, the meters, the LEDs and the running
  lights are never announced by themselves; the user asks for them with a key. What the user
  does is confirmed, once.
- **The original's keys keep working.** The computer keyboard stays the fastest way to play
  and program, and the mode speaks what each key did.
- **The same calls as the mouse.** Every accessible action ends in the same model call as a
  click, so the engine, the bit-exact output and recording in song mode are not touched.
- **Mode off is upstream.** With the mode off, the app behaves as upstream, key for key, with
  the exceptions of section 3.9.

The accessible layer lives in `crates/orb-ui/src/a11y/`, plus
`crates/orb-ui/src/mod_chooser.rs` and small hooks in `controller.rs` (upstream's single place
for the panel's input), `app.rs`, `menu.rs`, `audio_window.rs` and the pattern editors' files.
Section 7.5 has the options weighed on 2026-11-05, of which this is option B.

## 3. What the program does

### 3.1 Turning the mode on

**Does, heard (1, 19).** Options > Screen Reader Mode has three choices: Automatic (the
default), On and Off. It is kept in the settings file as `screen_reader`. Automatic follows
Windows' screen reader flag (`SPI_GETSCREENREADER`), which NVDA sets. It is asked at the start
and whenever the window regains the focus.

**Will do (TASK-199, on hold).** On macOS and Linux, Automatic stays off, and the mode is
turned on with On. Detection of VoiceOver and AT-SPI is TASK-199, on hold while the fork is
Windows only.

### 3.2 Sections and the focus order

**Does, heard (2, 3).** F6 and Shift+F6 move between the sections, as in other Windows
programs. Tab and Shift+Tab move control by control through the whole panel in this order,
wrapping at its ends. The sections are named groups, so NVDA says a section's name as the focus
enters it:

1. Transport: the controls in this order: Play, Stop, Record, Loop on/off, song/pattern mode,
   tempo, shuffle, bar, loop start, loop length, rewind and fast forward.
2. 303-1 (the first 303): its pattern row (pattern selector, row on/off, shuffle, length),
   waveform, tune, cutoff, resonance, envelope mod, decay, accent, the step editor, and its
   mixer channel (level, pan, delay send, PCF and compressor assignment, distortion, mute).
3. 303-2: the same.
4. 808: pattern row, accent level, the voices' knobs, the slot switches, the step grid, mixer
   channel.
5. 909: pattern row, accent level, the voices' knobs, flam, the step grid, mixer channel.
6. Master and compressor.
7. PCF.
8. Delay.
9. Distortion.

Within a section the pattern selector comes first, then the panel's order. A machine's mixer
channel is in the machine's section, because in ReBirth a row is one machine with its channel
(decision TASK-179). Entering a machine's section gives that machine the row focus, so the
synth and drum keys go to it (docs/ui_spec.md 20.2 and section 10).

**Does, heard (30).** The names say 808, 909, 303-1 and 303-2, as upstream's visible
text does (TASK-239). They were TR-808, TR-909 and Bass Line 1 and 2.

**Not built.** The row focus is not spoken. Entering a machine's section says only the
section's name, not that the machine now has the row focus, and Ctrl+Up / Ctrl+Down move the
row focus silently. The proposal of 2026-11-05 meant both to be announced.

### 3.3 Controls

- **Knobs and faders. Does, heard (7).** They are sliders with the value 0 to 127 (the value
  the original sends). The value is spoken in a physical unit derived from the engine's code,
  as precisely as it allows ("Cutoff 1.24 kilohertz", "Decay 350 milliseconds", "Level minus
  6.5 decibels"), each mapping marked [C], [D] or [G] in docs/control_units.md. Pan reads
  "center", "left 20", "right 20". Options > Speak Raw Values adds the stored number. Home and
  End go to 0 and 127, and NVDA's own slider commands use the Increment, Decrement and SetValue
  actions.
- **The next value that sounds different. Does, heard (27).** On many knobs
  neighboring positions sound the same (the 808's bass drum tone has 8 values over 128
  positions). Up and Down go to the next position whose spoken value differs, and Page Up and
  Page Down at least 8 positions, on to a different value. Where no different value lies that
  way, nothing moves. With Speak Raw Values on, each press moves one position (TASK-213).
- **Toggles and switches. Does, heard (5, 7).** A checkbox or a two-state switch with its
  state in words ("Waveform: square", "Mute: on"). Space presses or toggles it.
- **Spin buttons. Does, heard (4).** A display with its arrow buttons is one spin button: the
  tempo ("128 BPM"), bar, loop start, loop length, pattern length ("16 steps"), delay steps and
  PCF pattern. Up and Down click its arrows. Page Up and Page Down are 10 clicks. Home and End
  go to the ends of the range on pattern length, delay steps and PCF pattern only, and do
  nothing on tempo, bar, loop start and loop length (TASK-205).
- **The pattern selector. Does, heard (7).** A row's 8 pattern and 4 bank buttons are one spin
  button named "Pattern" ("B3"). Up and Down move through the 32 patterns, A1 to D8, through
  the same calls a click makes, so selections are recorded in song mode.
- **The 303 step editor. Does, heard (8).** In each 303 section, a slider named "Step editor"
  speaks the edit step and its contents ("Step 5 of 16: C sharp, octave up, accent, slide", or
  "Step 6: rest"):
  - each programming key of the original (the pitch keys, P, `[`, `;`, `'`, `-`, Enter,
    Backspace) is answered with the step's new state, once;
  - Up and Down move the step, Home and End go to the first and the last, and Left and Right do
    nothing (decision TASK-209);
  - F2 reads the whole pattern;
  - it edits its own pattern: playback moving the row to another pattern never changes it.
    Options > Step Editor Follows Playback (off by default) makes it move along, silently, at
    the next key (docs/ui_spec.md 20.5).
- **Pitch Keys Make Notes. Does, heard (24).** As in the original, a pitch key on a
  rest sets its note but leaves it a rest. The Options item Pitch Keys Make Notes (off by
  default, screen reader mode only) makes a pitch key turn a rest into a note (TASK-211).
- **The drum grids. Does, heard (9).** In the 808 and 909 sections, a grid named "Steps" of 12
  instruments by the pattern's steps:
  - Up and Down choose the instrument (Accent first), Left and Right the step, Home and End
    the first and the last step;
  - each cell reads "Bass drum, step 5, on", and on the 909 also "accent" or "flam";
  - Space sets the step as a click does. On the 909 a click goes round off, note, accent;
    Ctrl+Space sets a note, Shift+Space an accent and Ctrl+Alt+Space a flam (Alt+Space is an
    accent too, but Windows also opens the system menu; decision TASK-208);
  - F2 reads the instruments that have steps ("Bass drum: 1, 5, 9, 13");
  - like the step editor, a grid stays on its pattern during playback (docs/ui_spec.md 20.6).
- **LEDs, meters and the running light.** They take no focus. Their state is read on request
  (F4, section 3.5).

### 3.4 Keys in screen reader mode

**Does, heard (5).** The keys were decided by the user on 2026-11-05 (TASK-178, TASK-182) and
refined later (TASK-205, TASK-207, TASK-209):

| Key | What it does |
|---|---|
| F1 | Help > Keyboard Shortcuts, which lists the keys of the mode |
| F2 | Reads the pattern of the focused section, or of the open pattern editor window |
| F3 | Reads the transport ("Playing, bar 5, 128 BPM, song mode, loop 1 to 8") |
| F4 | Reads the master levels and the compressor's gain reduction |
| F5 | Play and Stop |
| F6 / Shift+F6 | The next / previous section |
| F9 (Insert too) | Record on and off (Insert is NVDA's key unless pressed twice) |
| F10, or Alt alone | The menu bar |
| F11 | Loop on and off |
| F12 | Song or pattern mode |
| F7, F8 | Free |
| Tab / Shift+Tab | The next / previous control |
| Space | Presses or toggles the focused control, or sets a grid step; it does not play |
| Enter | In a 303's section that 303's Step (Program Synth mode); in any other section Play |
| Up, Down, Page Up, Page Down, Home, End | Change the focused value (section 3.3) |
| Left / Right | On a drum grid the step before / after; nothing elsewhere |
| Ctrl+Up / Ctrl+Down | The original's row focus, without moving where you are |
| Ctrl+Tab / Ctrl+Shift+Tab | The original's 303 tap: the playing step becomes a note / a rest |
| Ctrl+M | Mods > Choose Mod (the mod chooser) |
| Ctrl+I | File > Info; in the mod chooser, Mod Info of the selection |
| Ctrl+E | The pattern editor of the focused row (section 3.7) |
| Escape | Closes a window, a dialog or a menu |

Every other key of the original keeps its meaning, whatever has the focus, so the panel's
keyboard still plays and programs the synths (Ctrl+F) or selects patterns (Ctrl+G). The panel's
keys wait while a menu, a dialog or a window has the focus (TASK-202).

**Does, heard (28).** On Windows the numpad's keys are told apart from the main
keyboard's (a message hook, TASK-225), so they keep the original's transport meaning in every
mode: numpad - and + change the tempo, numpad Enter plays, numpad * is Record. With Num Lock
off, NVDA's desktop layout takes the numpad.

### 3.5 Announcements and the read-out keys

**Does, heard (6, 21).** A polite live region, without bounds and without the focus, speaks
what the user should hear without moving the focus. Each message is a new node, so a message
repeated is spoken again (docs/ui_spec.md 20.3). It speaks:
- the status line's messages ("Saved ...", "Exported ...", and what each Edit menu command
  did);
- a song opened, with its mod ("Opened Trakka Rekka.rbs, mod Standard ReBirth"), and a mod
  chosen;
- the keyboard mode switched (Ctrl+F, Ctrl+G);
- what a key of the original did when the focused control does not show it: "Playing",
  "Stopped", "Recording on", "Tempo 141 BPM", "303-1 pattern A2".

A value changed on the focused control is spoken by NVDA from the value change itself and is
not announced again. Nothing is announced by itself during playback; a test plays a song for
10 seconds with no input and checks that the announcer stays as it was.

The read-out keys:
- F2: the pattern row of the focused section ("303-1, pattern A1, 16 steps, shuffle off"),
  then its steps;
- F3: the transport;
- F4: the levels as the panel shows them ("Master left 7 of 10, right 6 of 10, gain reduction
  5 of 9").

**Not built.** The meters are read as lit segments, not in decibels: the original's meter scale
has not been read.

### 3.6 Menus, files, mods and windows

- **The menu bar. Does, heard (13 to 17).** F10, or Alt pressed and released alone, focuses
  File. The bar has six titles: File, Edit, View, Mods, Options and Help (Edit and View came
  from upstream in 2026-11). The arrows walk the bar and the menus, Enter chooses, and Escape
  closes. The Options menu's checkboxes and radio items, and View's Classic and Modern, carry
  their state (docs/ui_spec.md 18.2).
- **Left and Right between the menus. Does, heard (36).** The fault heard on
  2026-11-28 was that egui's spatial focus took Left in an open menu back to the title, after
  which the other menus could not be reached, and Right from Help left the bar for the
  Transport section. TASK-247 makes them go from menu to menu, wrapping, as in Windows, in
  screen reader mode (docs/ui_spec.md 18.2). Heard working on 2026-11-28.
- **Menu items that are off. Fault heard (26, 29, 38): TASK-250.** An item that is turned
  off (Export and the two Initialize items in pattern mode; Info, the pattern commands and the
  pattern editors without a song; Undo with nothing to undo) cannot take the focus in egui, so
  Up and Down skip it and NVDA never says it is there. On 2026-11-28 the user could not find
  Export or Initialize Song. Will do (TASK-250), the user's choice of Windows' way: in screen
  reader mode such an item is reached by Up and Down and read as "unavailable", and Enter on
  it runs nothing and says why ("Needs song mode; F12 switches"). With the mode off, nothing
  changes.
- **The focus after the menu bar. Does, heard (23).** Leaving the bar (Escape, an item
  that opens no window, F10 or Alt again) gives the focus back to the control that had it. A
  window opened from a menu returns it there when it closes (TASK-212).
- **The Mods menu's mark. Does, heard (25).** The playing kit's entry reads as checked, and
  so does the submenu of a playing variant (TASK-201). Heard on 2026-11-28 with no mods
  installed (the Standard ReBirth entry); the submenu of a variant has not been heard.
- **File dialogs. Does, heard (11, 20).** In the mode, Open, Save As, Export and Mods > Choose
  file use the system's dialog. Options > Set Songs Folder... and Set Mods Folder... replace
  the in-app browser's "Use as" buttons. A name the dialog did not confirm, of a file that
  exists, asks "Replace NAME?", with No as the default (TASK-203). With the mode off, the
  in-app browser is as upstream's.
- **The Export format. Does, not yet heard (26).** The system Export dialog offers the format
  last exported first, and a name typed without an extension gets it. It is kept across
  restarts (TASK-204). Export works in song mode only (F12 switches), as in the original.
- **The mod chooser. Does, heard (12).** Mods > Choose Mod... or Ctrl+M opens a dialog with a
  filter field and a tree of the Mods menu's entries, in the menu's order ("Standard ReBirth"
  first). Each entry is read with "variants: N", "has demo", "current". Right and Left open and
  close a name's variants, Enter loads, Ctrl+I shows Mod Info without loading it, and Escape
  closes. It opens in under 5 ms with the 311 entries of the rb-mods collection
  (docs/ui_spec.md 20.7).
- **The windows and prompts. Does, heard (15, 16, 22).** Keyboard Shortcuts, File > Info,
  About and Mods > Info are dialogs with their title. Their lines can be read one by one with
  Tab ("Ctrl+N: File > New"), and About's Copy version button is named. The error box is an
  alert named "Error", with the focus on OK. The unsaved-changes prompt is a dialog named by
  its question, with the focus on Save. The focus moves in when one opens, and comes back where
  it was when it closes (docs/ui_spec.md 20.4).
- **A question opened from the menu. Does, heard for File > New (38).** With unsaved changes, File >
  New chosen from the menu opened Save As at once, while Ctrl+N asked first (the fault the user
  heard on 2026-11-28). The Enter that chose the item was in the frame that draws the question
  and answered it (Save; on Initialize Song's question Enter is Initialize). Fixed in
  TASK-249, with the mode on and off: a question takes no key in the frame that draws it
  first, and then only new presses, not a held key's repeats (`answer_key` in `menu.rs`).
  Headless tests walk the bar with F10, the arrows and Enter for File > New, Edit >
  Initialize Song and File > Open (`tests/a11y_menu_question.rs`). File > New was heard
  working on 2026-11-28; Initialize Song could not be found (TASK-250, above).
- **The Initialize Song question. Does, not yet heard (29).** Edit > Initialize Song from
  Pattern Mode asks first. In the mode the question is a dialog named by its title, with the
  original's text as its description and the focus on Initialize; Tab reaches Cancel. After
  either answer, the focus goes back where it was (TASK-237). The item is on in song mode
  only (F12 switches), as in the original.
- **Options > Audio Output. Does, not yet heard (31, 35, 37).**
  In the mode the window is a dialog named Audio Output, with the focus on Driver as it opens:
  - Driver, Device, Sample rate and Buffer size are named lists that say what is chosen;
  - a change is announced once ("Sample rate: 48000 Hz");
  - the "In use" numbers are lines Tab reaches ("label: value"), and their refresh every 500 ms
    is never spoken;
  - the lists follow a Windows dialog's combo box (TASK-248; the user heard faults in item 31
    and chose these keys): each box's description says "Alt+Down opens the list", read after
    its value; Alt+Down, Enter and Space open the list, with the highlight on the chosen
    entry; in the open list Up, Down, Home and End move the highlight (the focus, so NVDA
    reads each entry), Enter, Space or Alt+Up choose it, Escape keeps the choice as it was; on a
    closed box Up, Down, Home and End choose at once, no wrap, each announced once as a
    change through the list is; the focus stays on the box after every close, and a second
    Escape closes the window; Help > Keyboard Shortcuts has a line for these keys;
  - two details the user kept as built (2026-11-28): Home and End work in the open list as
    well as on the closed box, and on a box whose chosen entry is not in the list (a device
    that is gone) Down goes to the first entry and Up to the last;
  - after the program falls back from a device that is gone, the window shows the device in
    use, not the lost one, and that is never saved over the user's choice (TASK-234,
    TASK-245);
  - **Fault heard (31): TASK-251.** ASIO has no default device, so Device at System default
    on ASIO opens the first installed driver. On the user's PC that is the MiniFuse ASIO
    Driver, whose device is not connected, and the user heard "Audio output: the output
    stream: sample clock or rate cannot be determined or is not present; using the System
    default with the default settings" (ASIO's own "no clock" error), after which the program
    played on WASAPI. Will do (TASK-251): System default on ASIO opens the first driver that
    opens, and a failure names the driver and says what the program did instead.

### 3.7 The pattern editor windows

Edit > 303-1, 303-2, 808 and 909 Pattern Editor, or Ctrl+E on a row, open a window for one
pattern (upstream's, made for the mouse). In screen reader mode each is a dialog that takes the
focus on Pattern as it opens, and gives it back to the panel control on Escape or close.
Playback neither moves the window to another pattern nor makes it speak. Every edit goes
through the window's own calls, so it is heard, marks the song modified, and is one Undo step.
With the mode off the windows are upstream's.

- **The 303 windows. Does, heard (33).** The controls, in Tab order: Pattern (a spin
  button, A1 to D8), Step editor, Length, Shuffle, Transpose up and down, Shift left and right,
  Copy, Paste and Clear. The Step editor is a list of the 16 steps with the panel step editor's
  keys and words, Pitch Keys Make Notes included. Transpose and Shift say what they did
  ("Pattern A1 transposed up"), and F2 reads the pattern (TASK-235, docs/ui_spec.md 20.8).
- **The 808 and 909 windows. Does, heard (34).** The controls, in Tab order: Pattern,
  Steps (a grid with the panel drum grid's keys and words), the 909's Entry (Cycle, Low (on),
  High (accent), Flam), Length, Shuffle, Shift left and right, Copy, Paste and Clear. Shift
  says "Pattern A1 shifted right", or "Pattern A1 unchanged" when nothing moved, and F2 reads
  the pattern (TASK-236, docs/ui_spec.md 20.9).

Some details of these windows were chosen by the agents and wait for the user's confirmation:
one Pattern control, no pitch mode, the keys in either keyboard mode, the Accent row first, the
window's own cursor, the Tab order, the Entry words (the plan's Part 2.2).

### 3.8 The classic look

**Does, heard (32).** With View > Classic (the original panel drawn from the mod's
pictures), screen reader mode reads and works as with the modern look: the same names,
sections, Tab order, pattern selectors, drum grids and step editor. Switching between the looks
keeps the focus where it is and says nothing. The classic panel's round 808 instrument knob is
not a stop and is not read, because Up and Down on the drum grid choose the instrument
(TASK-238; the plan's Part 2.2, choice 1).

### 3.9 With the mode off

**Does, heard (18).** With screen reader mode off, the app behaves as upstream, key for key:
Space plays, Tab taps the 303 while playing, and the in-app file browser is used. The existing
keyboard tests pass unchanged. The exceptions, decided by the user, apply in both modes:
- the panel's keys wait while a menu is in use (TASK-202);
- every start writes a run log, and Windows prefers DirectX 12 before Vulkan, so the program
  starts on more PCs (TASK-214, TASK-217);
- the numpad's keys keep the original's meaning (TASK-225).

The work stays in its own module with small hooks, so merging upstream stays simple, and it
could be offered upstream. Upstream's web version would not get it (eframe 0.37's web runner
drops the AccessKit update), and the fork has no web version (TASK-198).

### 3.10 Screen readers, systems and braille

- **NVDA on Windows. Does, heard (1 to 22).** The one combination the user has checked.
- **Narrator and JAWS. Not tried.** They read the same UI Automation tree, so they should get
  the names, roles and values; live regions and grid navigation may be spoken differently.
- **VoiceOver and Orca. Will do (TASK-199, on hold).** AccessKit serves them, but the fork is
  built for Windows only, and Automatic does not detect them. TASK-206, also on hold, is
  Linux's file dialog when it cannot open.
- **Braille. Not checked.** No one has checked what a braille display shows: neither the
  controls' names and values nor how NVDA presents the announcements in braille.
- **A MIDI input port. Not built.** The user kept orb-player's MIDI layer for a MIDI input
  port later (2026-11-27). When it comes it will need its own screen reader work.

## 4. Decisions

The user made the six up-front decisions on 2026-11-05 (TASK-177 to TASK-182) and many more
since. Each is a Done task whose description has the options weighed and the reasons. The plan
lists them all ([accessibility_plan.md](accessibility_plan.md), Part 1.1).

1. How screen reader mode turns on: by itself when the system reports a screen reader, plus an
   Options menu item (Automatic, On, Off), remembered.
2. Space and Enter: Space presses or toggles the focused control; F5 is Play and Stop; Enter
   keeps the original's meaning (the 303's Step, else Play).
3. Where a machine's mixer channel is: in the machine's section.
4. File dialogs in screen reader mode: the system's own.
5. How values are spoken: in physical units, as precise as the engine's code allows, each
   marked as exact, derived or estimated in docs/control_units.md; the stored number on
   request.
6. The key map: section 3.4, the transport on the function keys, the 303 tap on Ctrl+Tab.

## 5. The third-party mods and songs

The mod collection and the song collection are upstream's own repositories,
github.com/pingw33n/rb-mods and github.com/pingw33n/rb-songs, not part of openrb-338. They are
kept outside this repository, next to it (the user's decision, 2026-11-05: about 1.1 GB of
third-party files whose licenses are not stated):

- `C:\GIT\music\rb-collections\rb-mods\mods`, recrypted, the form orb loads;
- `C:\GIT\music\rb-collections\rb-songs\songs`.

The project's own check (task-058, task-129; docs/rbm.md section 11) reads them in place:

```text
ORB_THIRDPARTY_MODS_DIR=C:/GIT/music/rb-collections/rb-mods/mods \
ORB_THIRDPARTY_SONGS_DIR=C:/GIT/music/rb-collections/rb-songs/songs \
    cargo test --release -p orb-cli --test thirdparty -- --nocapture --test-threads 1
```

In the UI, Mods > Choose file... on that folder, then "Use as mods folder" (in screen reader
mode Options > Set Mods Folder...), makes the Mods menu list them.

The latest run, TASK-240 (2026-11-27, rb-mods bfbd8c9, rb-songs 6e0b92e): 326 mods load, and all
5685 songs render equal to the original's renders.

### 5.1 The first run (2026-11-05, Windows 11, main at cbca449)

At rb-mods fe08cb8 and rb-songs 4af1075 (both 2026-10-31): 320 mod files and 1,268 distinct
songs.

- `mods` test: passed. Every one of the 320 recrypted mods parses, and its embedded files and
  46 sample slots hash as the manifest says.
- `songs` test: all 1,268 distinct songs (842 RBS 4.2, 426 ReBirth 1.x) rendered by orb hash
  equal to the original ReBirth.dll's renders, including the 531 songs on third-party mods.
  The first run failed on one check: 4 song files with an accented letter in their path were
  "not in the manifest", because the manifest held them decomposed (NFD) and git checks them
  out composed (NFC). The test now compares the NFC forms (`same_path` in
  `crates/orb-cli/tests/thirdparty.rs`, TASK-176). This was a defect of the test's data, not
  of orb or of the mods.
- The UI's own path, checked with a temporary headless test (not committed): with the folder as
  the mods folder, the Mods menu listed 311 entries for the 320 files (mods with equal names
  merge into one entry with variants). `App::use_kit` loaded every file with no error, and all
  259 mods that carry a demo song opened it with no error and gave sound within 8 seconds of
  Play. That sound can include the mod's startup sound, so this shows that the mod's samples
  play, not that the demo sounds right; the songs test is the check of the sound. One mod name
  differs from its menu entry: `omega777.rbm` calls itself "omega" and sits as a variant under
  "Omega", because the names are compared ignoring ASCII case (`mods::names_match`).

## 6. Reference for developers

### 6.1 How egui 0.37 builds the accessibility tree

What follows was read in the egui 0.37.0 and AccessKit 0.25.2 sources this workspace uses
(`~/.cargo/registry/src/*/egui-0.37.0/src`).

- **Turning it on.** eframe's default feature `accesskit` builds the platform adapter
  (`accesskit_winit` 0.34.2, `accesskit_windows` 0.35.2 on Windows). The tree is only built
  after `Context::enable_accesskit` (`context.rs:4234`); egui-winit calls it when the platform
  asks for the tree, i.e. when a screen reader or another UI Automation client first looks at
  the window. Until then there is no cost. A headless test turns it on itself.
- **Where a frame's tree comes from.** At the start of a pass, egui makes a root node with the
  role Window (`context.rs:561`). A widget's `Response` adds a node for the widget's id when it
  reports `WidgetInfo` (`Response::widget_info`, `response.rs:951`; the node is filled by
  `fill_accesskit_node_from_widget_info`, `response.rs:1008`: role, label, value, disabled).
  Any code can add to or create a node with `Context::accesskit_node_builder(id, |node| ...)`
  (`context.rs:4203`), which holds the context's lock while the closure runs, so the closure
  must not call back into the context. A node's parent is the nearest enclosing `Ui` that has a
  node (`find_accesskit_parent`, `context.rs:665`), or the root. The finished tree leaves the
  pass in `FullOutput::platform_output.accesskit_update` (a `TreeUpdate`: the nodes, the
  focused node).
- **Why painted rectangles join the Tab order.** `ui.interact(rect, id,
  Sense::click_and_drag())` registers a widget that can take the focus:
  `Sense::click_and_drag` includes `FOCUSABLE` (`sense.rs:82`). egui then puts it in its Tab
  order and gives it a node with no role ("Unknown"; UI Automation: "Custom") and no name.
  This is where the 250 silent stops of 2026-11-05 came from (section 7.2).
- **Actions from the screen reader.** A screen reader's request (Click, Focus, Increment,
  Decrement, SetValue, ...) arrives as `Event::AccessKitActionRequest` in the frame's input.
  egui handles Focus and Click itself for its own widgets (`context.rs:1579`); other actions
  are read by the widget with `ui.input(|i| i.accesskit_action_requests(id, action))`
  (`input_state/mod.rs:882`). A node advertises what it supports with `add_action`.
- **Keyboard focus.** egui moves its focus on Tab and Shift+Tab, and with the arrow keys
  (`memory/mod.rs:588` to 600: Up, Down, Left, Right without modifiers); Escape clears it. The
  panel's own keys read the same key events independently, which is the source of the
  collisions of section 7.3; screen reader mode cancels egui's own moves on the panel
  (docs/ui_spec.md section 10).
- **Announcements.** A node with `set_live(Live::Polite)` (or `Assertive`) is a live region.
  `accesskit_windows` raises UI Automation's LiveRegionChanged event for an added named node
  (`adapter.rs:256`), and for an updated one only when its name changed (`adapter.rs:313`),
  which is why each announcement is a new node (section 3.5).
- **Roles that fit the panel.** AccessKit 0.25 has, among others, `Slider`, `SpinButton`,
  `CheckBox`, `RadioButton`, `RadioGroup`, `Button`, `Group`, `Region`, `Toolbar`, `Grid`,
  `Row`, `GridCell`, `Dialog`, `AlertDialog` (`accesskit-0.25.2/src/lib.rs`).
- **Deprecations in 0.37.** `egui::WidgetType` is now `egui::Role` (a re-export of
  `accesskit::Role`; some variants renamed, e.g. `TextEdit` is `TextInput`), and `Id::new` is
  replaced by `Id::unique` or `IdSalt::new`.
- **The web.** eframe 0.37's web runner ignores the AccessKit update
  (`eframe-0.37.0/src/web/app_runner.rs:403`, "not currently implemented").

### 6.2 A headless test of the tree

`crates/orb-ui/tests/common/a11y.rs` (task-183, A11Y-0) is the shared helper, used from a test
file with `#[path = "common/a11y.rs"] mod a11y;`:
- `AppTree` runs the `App` headless and keeps the last tree (`tree()`, `key`, `send`);
- `Frames` runs any UI closure (a small test widget);
- `Tree` finds nodes (`by_id`, `by_layout(kind, id)`, `by_name`, `by_role`, `focused`);
- `request` / `set_value_request` make the events a screen reader sends.

`tests/a11y_actions.rs` shows an Increment, Decrement and SetValue arriving in a widget, and
`tests/a11y_baseline.rs` holds the counts of section 1.1. The core of it:

```rust
let ctx = egui::Context::default();
egui_extras::install_image_loaders(&ctx);
ctx.enable_accesskit();
let mut app = App::new(Panel::new(), None);
app.menu = true;
let out = ctx.run_ui(raw_input, |ui| app.show(ui));
let update = out.platform_output.accesskit_update.unwrap(); // nodes and focus
// a screen reader's request, sent in the next frame's input:
let req = accesskit::ActionRequest {
    action: accesskit::Action::Increment,
    target_tree: accesskit::TreeId::ROOT,
    target_node: id.accesskit_id(),
    data: None,
};
let raw_input = RawInput { events: vec![Event::AccessKitActionRequest(req)], ..raw_input };
```

Run a few frames before reading the tree: the first frames load the SVG parts. Clear
`out.textures_delta` after each frame, as the other tests of `crates/orb-ui/tests` do.

### 6.3 The real window: `tools/uia_tree.ps1`

`tools/uia_tree.ps1` starts the release build (`--demo` by default: no sound, no song), finds
its main window and reads its UI Automation tree. It prints:
- the window's name and the number of elements;
- the elements by type and by whether they have a name;
- how many can take the focus, and the names.

`-Dump` lists every element, `-Arguments` passes other arguments (a song), and `-ScreenReader
on`, `off` or `auto` writes the mode into the throwaway settings file (`ORB_UI_CONFIG`), so the
user's own settings are untouched. It closes the program it started. Windows only; no screen
reader needs to run.

```text
cargo build --release -p orb-ui
pwsh tools/uia_tree.ps1
```

Two things it handles that are easy to get wrong: the process has a hidden window of the same
class and title as the main window (the script takes the visible one), and the first UI
Automation request gets a placeholder element while AccessKit starts, so the script asks again
with a fresh element. The main window shows only after its first frame, about 10 seconds after
a first start on the test machine.

The last output, on main at 6cc85ab (2026-11-07), `-ScreenReader on`:

```text
window: 'OpenRB-338 - Untitled'
elements: 593
by type, named:
   384  DataItem, True
    88  Slider, True
    62  Button, True
    23  CheckBox, True
    14  Spinner, True
     9  Group, True
     9  RadioButton, True
     2  DataGrid, True
     1  Menu, False
     1  Pane, False
can take the focus: 198, of them with a name: 198
```

The 384 DataItems are the cells of the two drum grids (2 x 12 instruments x 16 steps, named
"Bass drum, step 1, off", not focusable: the grid's active descendant is reported as the
focus). The 88 Sliders are the 77 knobs, the 9 faders and the two step editors. The nine Groups
are the sections, and the two DataGrids the grids ("Steps"). The 198 that take the focus are
the 194 controls, the two step editors and the two grids. With `-ScreenReader off`: 198
elements, 194 that take the focus, all named. The mod chooser and the other windows are not in
the tree until they are open. The outputs of each earlier task are in this document's history
(`git show df4d002:docs/accessibility.md`).

### 6.4 NVDA and this program, for developers who do not use it

- NVDA reads a program like this one in focus mode: every key that is not an NVDA command
  goes to the program, so the panel's letter keys work as they are.
- The NVDA key is Insert (and the keypad's Insert) by default; Caps Lock can be made one too
  (common with the laptop layout). NVDA keeps it: the program sees Insert only when the user
  presses it twice quickly.
- In the desktop layout with Num Lock off, NVDA uses the keypad keys for its review cursor and
  mouse; with Num Lock on they reach the program.
- NVDA speaks a focused control's name, role and value when it gets the focus, and the value
  again when it changes. A live region's text is spoken when it changes, without moving the
  focus.
- NVDA's Speech Viewer (NVDA menu > Tools) shows what NVDA says, as text, for checking by
  sight; its log (NVDA menu > Tools > View log) shows the events it received at the debug log
  level.

## 7. History: before the work

This section keeps the state of 2026-11-05, on main at cbca449, and the reasons for the
design. The file and line references are of that commit; the panel's input has moved since into
upstream's `controller.rs`.

### 7.1 What a screen reader got

- Headless, through egui's own accessibility tree: 261 nodes. 4 had a name: the menu titles
  File, Mods, Options and Help. 250 had the role "Unknown" and no name. No node had a value.
  After each of six presses of Tab, the focus sat on an "Unknown" node with no name.
- The real window, through UI Automation: 256 elements. 254 could take the keyboard focus, and
  4 of those had a name (the same four menu titles). The other 250 were "Custom" elements with
  no name, which NVDA read as nothing, or as "unknown".

So Tab walked through 250 silent stops. The panel could not be used with a screen reader: no
control said what it was or what its value was.

### 7.2 Why

- The panel is drawn, not built from widgets. `assets/ui/layout.tsv` listed 348 parts: 77
  knobs, 9 faders, 46 trigger buttons, 48 pattern grid buttons, 23 toggles, 9 radio buttons,
  18 repeat buttons, 4 wind buttons, 10 switches, 12 seven-segment displays, 38 LEDs, 19
  meters, 32 step parts, 2 click areas and the mod picture. Each was an SVG image painted into
  a rectangle (`paint_part`, `app.rs:419`).
- The mouse was taken by `App::hit` (`app.rs:892`) with `ui.interact(rect, id,
  Sense::click_and_drag())`, which puts each rectangle in the Tab order with nothing to
  describe it (section 6.1).
- The status line was painted text (`App::status_bar`, `app.rs:307`), so "Saved ..." and
  "Exported ..." were never spoken. The rows of the file browser were painted too
  (`browser.rs:881`): the file list was silent.
- The information that matters most was not a control at all: the notes and flags of a 303
  pattern (`Step303` in `model.rs`) and the 12 x 16 drum grids, of which the panel shows only
  the selected instrument's row.

### 7.3 The keys that collided, and how the mode resolves them

The panel's computer keyboard (`keys.rs`, docs/ui_spec.md section 10) reads every key press
whatever egui has focused. With a screen reader this collided in four places:

- Tab and Shift+Tab move egui's focus, and they are also the 303's tap keys: while a 303
  plays, Tab writes a note into the playing step. Navigating would edit the song. In the mode
  Tab navigates, and the tap is Ctrl+Tab.
- Up and Down moved the row focus, and egui also moves its own focus on the arrows. In the mode
  the arrows change the focused value, and the row focus is Ctrl+Up / Ctrl+Down.
- Insert is Record, but Insert is NVDA's key. In the mode F9 is Record too.
- With Num Lock off, NVDA's desktop layout uses the numpad, so its transport keys reached the
  app only with Num Lock on. That stays so; since TASK-225 the numpad is told apart from the
  main row's digits on Windows.

Also, auto-repeated key presses were ignored on purpose. In the mode the value keys and Tab
repeat.

### 7.4 What was already in place

- AccessKit was compiled in (eframe's default feature `accesskit`), waking up when the first
  UI Automation client asks for the window.
- A test showed that AccessKit could carry what was needed: a painted 20 x 20 rectangle given
  the role Slider, a name, a value and the Increment action received NVDA's kind of request
  and showed the new value in the next tree.
- AccessKit's Windows adapter raises the LiveRegionChanged event NVDA speaks, so
  announcements need no second speech path.
- The menus, the prompts and the windows were made of real egui labels and buttons, which
  carry names already.
- The window title carries the song's name and a `*` when it is modified; NVDA+T reads it.
- The original's computer keyboard is a ready-made, non-visual instrument for programming the
  303s and tapping the drums.
- Every mouse gesture ends in a method of `model::Panel`, so an accessible control can make
  exactly the events the mouse makes.
- The cost: with AccessKit on, the headless panel took 2.3 to 3.0 ms per frame, against 2.3
  ms with it off: within the noise.

### 7.5 The options weighed

- **A. Names and roles on the painted controls only.** The cheapest step, and part of what was
  built. On its own it would leave a flat Tab order of about 250 stops, LEDs and meters that
  change 30 times a second, and patterns that cannot be read.
- **B. An accessible layer in the same window.** Keep the panel as it looks, and add an
  accessibility tree built from the panel model, with a mode that resolves the key conflicts.
  This was proposed and built (sections 2 and 3).
- **C. A separate accessible program.** A second front end would duplicate every feature,
  drift away from the visual UI and from upstream, and need its own port for each platform.
  Rejected. The `orb` command line tool already works with a screen reader.
- **D. Speaking directly through NVDA's controller client (or Tolk).** Windows only, outside
  the accessibility tree (focus and values would still be missing), and a second voice path.
  Not needed, since AccessKit's live regions reach NVDA.

The proposal as written on 2026-11-05, and the document as it stood on 2026-11-09, are in its
history (`git show 2611c80:docs/accessibility.md`, `git show 59cc058:docs/accessibility.md`).
