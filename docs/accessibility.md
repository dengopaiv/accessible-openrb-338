# Screen reader accessibility: a proposal

Status, 2026-11-07: the proposal of section 3 is implemented, as screen reader mode (Options >
Screen Reader Mode), by TASK-184 to TASK-196 and TASK-210 (docs/accessibility_plan.md, part 1;
the behavior is docs/ui_spec.md section 20, the user's guide the README's "Using OpenRB-338
with a screen reader"), and the user's pass with NVDA (TASK-197) found every item of its
checklist working by ear. The follow-ups are in the plan's part 2. The text below is the proposal as written on 2026-11-05, kept
for its reasons; section 1.1 has the measurement of that day and of 2026-11-07. The first
target is NVDA on
Windows (the user's choice); AccessKit, the layer proposed here, also serves Narrator and JAWS
through UI Automation, VoiceOver on macOS and Orca on Linux, so the other platforms come along,
with less testing.

The document has these parts: what a screen reader gets today (measured), the options, the
proposal, the decisions made on it, the third-party mods and songs, and a reference for
developers (egui's accessibility API and how to measure). The work itself is planned, task by
task, in [accessibility_plan.md](accessibility_plan.md).

## 1. What a screen reader gets today

### 1.1 Measured

Two measurements, on main at cbca449:

- Headless, through egui's own accessibility tree (`ctx.enable_accesskit()`, then the panel
  drawn as in `crates/orb-ui/tests/keyboard.rs`): 261 nodes. 4 have a name: the menu titles
  File, Mods, Options and Help. 250 have the role "Unknown" and no name. No node has a value.
  After each of six presses of Tab, the focus sits on an "Unknown" node with no name.
- The real window on Windows, read through UI Automation from PowerShell
  (`System.Windows.Automation`, `OpenRB-338.exe --demo`; `tools/uia_tree.ps1` repeats this
  measurement, section 6.3): 256 elements. 254 can take the
  keyboard focus; 4 of those have a name (the same four menu titles). The other 250 are
  "Custom" elements with no name. NVDA reads these as nothing, or as "unknown".

So Tab already walks through 250 silent stops. The panel cannot be used with a screen reader:
no control says what it is or what its value is.

Since task-186 (A11Y-2) every panel control has a name, a role and its value or state, and the
focusable elements are 248, all named (section 6.3, docs/ui_spec.md section 20). The other
gaps of this section were closed by the later tasks of the plan; the measurement again, after
them:

Measured again on 2026-11-07, main at 6cc85ab (after TASK-195, the last task to change the
tree), the same two ways. The program is the release build; `tools/uia_tree.ps1` was run with
`-ScreenReader on`, `off` and `auto` (NVDA was running, so `auto` is on).

| | 2026-11-05, cbca449 (before) | 2026-11-07, 6cc85ab, mode off | 2026-11-07, 6cc85ab, mode on |
|---|---|---|---|
| Headless nodes | 261 | 203 | 598 |
| of them with a name | 4 | 196 | 591 |
| with a value | 0 | 112 | not counted |
| that can take the focus | not measured | 194 | 198 |
| Real window (UI Automation) elements | 256 | 198 | 593 |
| of them that can take the focus | 254 | 194 | 198 |
| of those, with a name | 4 | 194 | 198 |
| Unnamed "Custom" elements | 250 | 0 | 0 |

With the mode on, the 593 elements are 384 DataItems (the cells of the two drum grids), 88
Sliders (77 knobs, 9 faders, the two step editors), 62 Buttons, 23 CheckBoxes, 14 Spinners, 9
Groups (the sections), 9 RadioButtons, 2 DataGrids, the menu bar and the pane. The 198 that
take the focus are the 194 controls (section 6.3), the two step editors and the two grids;
the grid cells take no focus themselves. With the mode off the tree is the one of TASK-187 (the
two 303 edit steps are Text labels): no unnamed element is left, and the Tab order is
egui's. Tab, which stopped on 250 silent places, now stops on 198 places that all say what
they are. The user's NVDA pass (TASK-197, 2026-11-07) found them reading as they should.

The numbers come from `cargo test --release -p orb-ui --test a11y_baseline -- --nocapture`
(headless) and `tools/uia_tree.ps1` (section 6.3; the test `a11y_baseline` holds the counts).

### 1.2 Why

- The panel is drawn, not built from widgets. `assets/ui/layout.tsv` lists 348 parts: 77
  knobs, 9 faders, 46 trigger buttons, 48 pattern grid buttons, 23 toggles, 9 radio buttons,
  18 repeat buttons, 4 wind buttons, 10 switches, 12 seven-segment displays, 38 LEDs, 19 meters,
  32 step parts, 2 click areas and the mod picture. Each is an SVG image painted into a
  rectangle (`paint_part`, `crates/orb-ui/src/app.rs:419`).
- The mouse is taken by `App::hit` (`crates/orb-ui/src/app.rs:892`): `ui.interact(rect, id,
  Sense::click_and_drag())`. That sense includes "focusable", so egui puts each rectangle in
  the Tab order, but nothing describes it (no `WidgetInfo`, no AccessKit node data). This is
  where the 250 silent stops come from.
- The status line is painted text (`App::status_bar`, `crates/orb-ui/src/app.rs:307`), so
  "Saved ..." and "Exported ..." are never spoken.
- The rows of the file browser are painted too (`crates/orb-ui/src/browser.rs:881`, a
  `Sense::click` rectangle and `painter().text`): the file list is silent.
- The information that matters most is not a control at all: the notes and flags of a 303
  pattern (`Step303` in `crates/orb-ui/src/model.rs`) and the 12 x 16 drum grids, of which the
  panel shows only the selected instrument's row.

### 1.3 Keys that collide

The panel's computer keyboard (`crates/orb-ui/src/keys.rs`, docs/ui_spec.md section 10) reads
every key press whatever egui has focused (`App::keyboard`, `crates/orb-ui/src/app.rs:704`;
only a focused text field is excluded). With a screen reader this collides in four places:

- Tab and Shift+Tab move egui's focus to the next silent node, and at the same time they are
  the 303's tap keys (`keys.rs:165`): while a 303 plays, Tab writes a note into the playing
  step (`Panel::tap_303`, `model.rs:971`). Navigating would edit the song.
- Up and Down move the row focus (`keys.rs:142`), and egui also moves its own keyboard focus
  on the arrow keys.
- Insert is Record (`keys.rs:144`), but Insert is NVDA's default modifier key. NVDA keeps it,
  so Record cannot be reached without pressing Insert twice.
- With Num Lock off, NVDA's desktop layout uses the numeric keypad (0 to 9, `/`, `*`, `-`,
  `+`, Enter) for its review cursor and mouse. The keypad's transport keys reach the app only
  with Num Lock on. The main-row digits still work in Program Synth mode, because egui cannot
  tell them from the keypad's (`keys.rs`, module comment).

Also, auto-repeated key presses are ignored on purpose (`app.rs:725`). Holding a key to sweep
a value would need them.

### 1.4 What is already in place

- AccessKit is compiled in: eframe's default feature `accesskit` is on (`crates/orb-ui/Cargo.toml`
  takes `eframe = "0.37.0"` with defaults; `Cargo.lock` has `accesskit_windows` 0.35.2). It
  wakes up when the first UI Automation client (NVDA) asks for the window, and costs nothing
  before that.
- AccessKit can carry what is needed. A test drew a 20 x 20 rectangle with `ui.interact`, as the
  panel does, and gave it a node with the role Slider, the name "Cutoff", a numeric value 0 to
  127, the text "64 of 127" and the actions Increment, Decrement and SetValue. An Increment
  request (the kind NVDA sends) reached the app as
  `input.accesskit_action_requests(id, Increment)`, and the next tree held the new value 65.
- AccessKit's Windows adapter raises UI Automation's LiveRegionChanged event
  (`accesskit_windows-0.35.2/src/adapter.rs:261`), which NVDA speaks. Announcements need no
  second speech path.
- The menu bar, the menus, the unsaved-changes prompt, the error box and the Info, About,
  Keyboard Shortcuts and Mod Info windows are made of real egui labels and buttons, which
  already carry names. They need focus handling and roles, not a rewrite.
- The window title carries the song's name and a `*` when it is modified (`App::update_title`,
  `crates/orb-ui/src/menu.rs:1084`); NVDA+T reads it.
- The original's computer keyboard is a ready-made, non-visual instrument: in Program Synth
  mode the pitch keys, accent, slide, up, down, note/pause, step and back program a 303, the
  drum keys tap the 808 and 909; in Select Patterns mode four rows of keys select patterns.
- The panel model is apart from the drawing. Every mouse gesture ends in a method of
  `model::Panel`: `press`, `arrow`, `set_knob`, `set_fader`, `set_switch`, `select_instrument`,
  `set_step808`, `set_step909`, `tap_303`, `tap_drum`. An accessible control can call the same
  methods, so it produces exactly the events the mouse produces. The engine, the bit-exact
  output and recording in song mode are not touched.
- Cost: with AccessKit on, the headless panel took 2.3 to 3.0 ms per frame, against 2.3 ms with
  it off (600 frames each, two runs). That is within the noise, and the tree has to be
  measured again once it has the full set of nodes.

## 2. Options

### A. Names and roles on the painted controls only

Give each of the 348 rectangles a name, a role and a value, and nothing else. This is the
cheapest step and it is part of the proposal. On its own it leaves a flat Tab order of about
250 stops, LEDs and meters that change 30 times a second, and patterns that cannot be read.
Not enough.

### B. An accessible layer in the same window (proposed)

Keep the panel as it looks, and add to it an accessibility tree built from the panel model:
sections, composite controls, pattern editors and spoken feedback, with a screen reader mode
that resolves the key conflicts. Section 3 describes it.

### C. A separate accessible program

A second front end (native Win32 controls, or a text interface) next to the egui one. It would
duplicate every feature, drift away from the visual UI and from upstream, and need its own
port for each platform. Rejected. The `orb` command line tool (render, play, info, mod
recrypt) already works with a screen reader and stays as it is.

### D. Speaking directly through NVDA's controller client (or Tolk)

Windows-only, outside the accessibility tree (focus and values would still be missing), and a
second voice path. Not needed, since AccessKit's live regions reach NVDA. It could be added
later for one purpose if live regions turn out too slow for some announcement.

## 3. The proposal

### 3.1 Principles

- The panel keeps its look. The accessible layer sits in a module of its own
  (`crates/orb-ui/src/a11y/`) and calls the same `Panel` methods as the mouse.
- One accessible node per musical thing, not per painted part: the tempo is one spin button,
  not a display and two arrow buttons; a row's 32 patterns are one selector, not 12 buttons.
- Quiet while the music plays. The playing position, the meters, the LEDs and the running
  lights are never announced by themselves; the user asks for them with a key. What the user
  does is confirmed.
- The original's keys keep working: the computer keyboard stays the fastest way to play and
  program, and the accessible layer speaks what each key did.
- A "Screen reader mode" setting (Options menu, kept in `ui.toml`). With it off, the app
  behaves exactly as upstream, key for key, so the existing keyboard tests stay valid and the
  work can be offered upstream. It turns on by itself when Windows reports a running screen
  reader (the system's screen reader flag, `SPI_GETSCREENREADER`; whether NVDA sets it is to
  be checked during the work, plan task A11Y-1).

### 3.2 Sections

F6 and Shift+F6 move between sections, as in other Windows programs; Tab and Shift+Tab move
between the controls of a section. Entering a machine's section gives that machine the row
focus, so the synth and tap keys go to it, and says so ("Bass Line 2, keyboard focus").

1. Transport: Play, Stop, Record, Loop on/off, song/pattern mode, tempo, shuffle, bar, loop
   start, loop length, rewind and fast forward.
2. Bass Line 1 (the first 303): its pattern row (pattern selector, row on/off, shuffle,
   length), waveform, tune, cutoff, resonance, envelope mod, decay, accent, the step editor,
   and its mixer channel (level, pan, delay send, PCF and compressor assignment, distortion,
   mute).
3. Bass Line 2: the same.
4. TR-808: pattern row, accent level, the 11 voices' knobs, the 5 slot switches, the step grid,
   mixer channel.
5. TR-909: pattern row, accent level, the voices' knobs, flam, the step grid, mixer channel.
6. Master and compressor.
7. PCF.
8. Delay.
9. Distortion.

A machine's mixer channel goes into the machine's section, because in ReBirth a row is one
machine with its channel (decision 3, section 4).

Done in task-188: the sections are Group nodes in screen reader mode, and Tab goes through
the whole panel in this order, wrapping at its ends; within a section the panel's order, with
the pattern selector first and the transport in the order listed above (docs/ui_spec.md
section 20.2).

### 3.3 Controls

- Knobs and faders: Slider with the value 0 to 127 (the value the original sends:
  `behavior::knob_send`, `fader_send`). Up and Down change it by 1, Page Up and Page Down by
  8, Home and End go to 0 and 127, and NVDA's own slider commands use the Increment,
  Decrement and SetValue actions. The value is spoken in physical units derived from the
  engine's code, as precisely as it allows ("Cutoff 1.24 kilohertz", "Decay 350
  milliseconds", "Level minus 6.5 decibels"; decision 5, plan task A11Y-14), with a setting
  to add the stored 0 to 127 number. Pan reads "center", "left 20", "right 20". The new value
  goes through `Panel::set_knob(id, behavior::knob_from_engine(v))`, so the mouse and the
  keyboard store the same values.
- Toggles and switches: CheckBox or a two-state button with its state in words ("Waveform:
  square", "Delay: on", "Mute: on").
- Repeat buttons with their display: one SpinButton each, for tempo ("128 BPM"), bar, loop
  start, loop length, pattern length ("16 steps"), delay steps and PCF pattern. Up and Down
  call `Panel::arrow` with the up or down button's id, as a click does. Page Up and Page Down
  are 10 clicks; Home and End click to the end of the range on pattern length, delay steps and
  PCF pattern only, and do nothing on tempo, bar, loop start and loop length (TASK-205, the
  user, 2026-11-06).
- A row's pattern grid (8 pattern buttons and 4 bank buttons): one selector, "Pattern B3".
  Up and Down move through the 32 patterns, A1 to D8, through the same `Panel::press` calls a
  click makes, so selections are recorded in song mode as they are now.
- Done in task-187 for the sliders, spin buttons and pattern selectors (the selector is a
  SpinButton named "Pattern" whose value is "B3"), with a screen reader's Click on the buttons,
  toggles and switches acting as one mouse down: docs/ui_spec.md section 20.1.
- The 303 step editor: one node that speaks the step being edited ("Step 5 of 16: C sharp,
  up, accent, slide", or "Step 6: rest") after each of the existing programming keys (the
  pitch keys, P, `[`, `;`, `'`, `-`, Enter, Backspace). Up and Down move the edit step (Left
  and Right do nothing there, decision task-209).
  Done in task-190 (docs/ui_spec.md section 20.5): a Slider named "Step editor" in place of
  the edit step label; the focused editor's value change speaks a key's effect, the announcer
  where the value cannot (the same note again, a pitch key in pitch mode, the focus
  elsewhere). It edits its own pattern: playback moving the row to another pattern never
  changes it without input (Options > Step Editor Follows Playback, off by default, makes it
  move along, silently, at the next input; decision task-209).
- The 808 and 909 step grids: Grid role, 12 instrument rows by 16 steps. Up and Down choose
  the instrument (`Panel::select_instrument`), Left and Right the step, Space sets or cycles
  the step as a click does (`set_step808`; `step909_new_value` and `set_step909`, with Ctrl
  and Alt as on the 909's mouse). Each cell reads "Bass drum, step 5, on", and on the 909 also
  "accent" or "flam". Done in task-191 (docs/ui_spec.md section 20.6): the Grid node is a
  focusable child `Ui` of the machine's section whose `active_descendant` is the cell under
  the cursor, so the screen reader speaks each cell as the focus. Alt+Space also opens the
  window's system menu on Windows, so Shift+Space is an accent key too (decision TASK-208). As the step editor, a grid
  edits its own pattern (`Panel::drum_pin`): playback moving the row to another pattern never
  changes the grid without input, and the same setting makes it move along, silently, at the
  next input (decision task-209).
- LEDs, meters and the running light: not in the Tab order. Their state is read on request
  (3.4).
- The mod picture: its name is read with the song's information.

### 3.4 Keys in screen reader mode

Decided by the user on 2026-11-05 (decisions 2 and 6). No function key is used by OpenRB-338
or by the original's key map, so all twelve were free.

- F1: Help > Keyboard Shortcuts.
- F2: read the focused section's pattern (a 303 pattern as its steps; a drum machine as each
  used instrument with its steps, "Bass drum: 1, 5, 9, 13"). F3: read the transport
  ("Playing, bar 5 of 16, 128 BPM, song mode, loop 1 to 8"). F4: read the levels (meters, gain
  reduction).
- F5: Play and Stop.
- F6 and Shift+F6: next and previous section.
- F9: Record on and off (Insert keeps working for whoever does not use it as the NVDA key).
- F10, or Alt alone: the menu bar.
- F11: Loop on and off. F12: song or pattern mode.
- F7 and F8 are free.
- Tab and Shift+Tab: next and previous control.
- Space: press or toggle the focused control (a button, toggle, switch or grid step). In
  screen reader mode Space no longer plays; F5 does.
- Enter: what it does in the original, whatever has the focus: the 303's Step in Program
  Synth mode with a 303 focused, else Play. Programming a 303 works exactly as in ReBirth.
  Refined by the user on 2026-11-06 (task-207): "a 303 focused" is the section of the focused
  control, so Enter in the Transport or an effect section is Play whatever the row focus.
- Up, Down, Page Up, Page Down, Home, End: change the focused value. Left and Right: move in
  the step editor and the grids.
- Ctrl+Up and Ctrl+Down: the original's row focus, without moving the navigation focus.
- Ctrl+Tab and Ctrl+Shift+Tab: tap a note or a rest into the playing step of the focused 303
  (the original's Tab and Shift+Tab).
- Every other key of the original keeps its meaning, whatever has the focus.
- Help > Keyboard Shortcuts lists all of the above when the mode is on.

Done in task-188 (docs/ui_spec.md section 10, "Screen reader mode") and task-189 (F2 to F4,
docs/ui_spec.md section 20.3); Up / Down on the 303 step editor in task-190 (section 20.5);
Left / Right and Space on the drum grids in task-191 (section 20.6).

### 3.5 Announcements

A polite live Label node, without bounds and without the focus, carries what the user should
hear without moving the focus; each message is a new such node (one at a time, docs/ui_spec.md
section 20.3):

- the status line's messages ("Saved ...", "Exported ...");
- a song opened (its name and mod), a mod chosen;
- the keyboard mode switched (Ctrl+F, Ctrl+G), the row focus moved;
- what a play key did when the focus is elsewhere: a pattern selected with the pattern keys,
  the tempo changed with the plus and minus keys, Record turned on or off.

Values changed on the focused control are spoken by NVDA from the value change itself and
are not announced again.

The error box is an alert dialog that takes the focus (on OK), so NVDA reads it at once: done
in task-193 (A11Y-8, docs/ui_spec.md 20.4); whether NVDA reads its message is a hand check
(NVDA checklist item 22).

Done in task-189 (docs/ui_spec.md section 20.3): each message is a new live node, because
`accesskit_windows` raises LiveRegionChanged for an added named node (`adapter.rs:256`) but
for an updated one only when its name changed (`adapter.rs:313`), so a repeated message would
otherwise be silent. The read-out keys F2, F3 and F4 announce through it too.

### 3.6 Files, mods and windows

- The file browser: in screen reader mode, Open, Save As, Export and Mods > Choose file use the
  system's dialog through `rfd::FileDialog` (`rfd` 0.17.2 is already a dependency, for the
  crash box). The Windows dialog is fully accessible. "Use as songs folder" and "Use as mods
  folder" become Options menu items with a folder dialog. Without the mode, the in-app browser
  stays as it is.
- The Mods menu holds 311 entries (for the 320 files of the collection) in a scroll area inside
  a pop-up menu, which is slow to walk with arrow keys. In screen reader mode, Mods > Choose
  opens a dialog with a filter field and a list (name, file, whether it has a demo song), with
  Enter to choose and a key for Mod Info.
- The unsaved-changes prompt, the error box and the Info, About, Keyboard Shortcuts and Mod
  Info windows: a Dialog (or AlertDialog) role, the focus moved into the window when it opens
  (to the default button of a prompt), and Escape as now. Done in task-193 (docs/ui_spec.md
  20.4).

### 3.7 Order of the work

The user asked for everything; the order follows what each step needs.

1. Foundation: the `a11y` module, the screen reader mode setting, a name and role for every
   interactive part of `layout.tsv`, sections and the Tab order, sliders, toggles and spin
   buttons, and the four key collisions resolved. After this the whole panel can be operated
   and heard, except for the pattern contents.
2. Patterns: the pattern selector, the 303 step editor, the drum grids and the read-out keys.
3. Files and windows: system file dialogs, the mod chooser (done, task-195: a Tree of
   TreeItem nodes with the selected item as `active_descendant`, as the drum grid's cells;
   docs/ui_spec.md 20.7), dialog focus and roles, the announcement node, the status line.
4. Fine-tuning with NVDA: what is said and in what order, live-region timing, braille output,
   the keyboard help, and a "Screen reader mode" section in docs/ui_spec.md.

### 3.8 Testing

- Headless tests of the accessibility tree with `cargo test`, run in CI on Windows, macOS and
  Linux: every interactive control has a name and a role; the Tab order inside each section;
  slider values after Increment, Decrement and SetValue requests; the grids' cells; nothing
  announced during playback unless asked; with the mode off, the existing keyboard tests
  unchanged.
- A Windows smoke script that reads the real window through UI Automation, as the
  measurement in 1.1 did.
- A manual NVDA checklist, run by a screen reader user: an agent can check the tree and the
  events but cannot hear NVDA.

### 3.9 Relation to upstream

The work stays in its own module, with small hooks in `app.rs`, `keys.rs`, `menu.rs` and
`settings.rs`, and the mode off means no change. That keeps rebasing on upstream simple and
leaves the door open to offering it upstream. Upstream's planned web version (milestone m-6)
would not get it: eframe 0.37's web runner drops the AccessKit update ("not currently
implemented", `eframe-0.37.0/src/web/app_runner.rs:403`). This fork has no web version and
will not have one (the user, 2026-11-05, TASK-198: a local editor only); m-6 is archived here.

## 4. Decisions

All were decided by the user on 2026-11-05. The plan lists them with the decisions made
later ([accessibility_plan.md](accessibility_plan.md) section 1.1); the options weighed for
each, and the reasons, are in its decision task (A11Y-D1 to A11Y-D6 = TASK-177 to TASK-182).

1. How screen reader mode turns on: by itself when the system reports a screen reader, plus
   an Options menu item (automatic, on, off), remembered.
2. Space and Enter: Space presses or toggles the focused control; F5 is Play and Stop; Enter
   keeps the original's meaning (the 303's Step, else Play).
3. Where a machine's mixer channel is: in the machine's section.
4. File dialogs in screen reader mode: the system's own.
5. How values are spoken: in physical units, as precise as the engine's code allows, each
   marked as exact, derived or estimated in a new `docs/control_units.md` (plan task
   A11Y-14); the stored 0 to 127 number on request.
6. The key map: section 3.4, transport on the function keys, the 303 tap on Ctrl+Tab.

## 5. The third-party mods and songs

The mod collection and the song collection are upstream's own repositories,
github.com/pingw33n/rb-mods and github.com/pingw33n/rb-songs, not part of openrb-338. They are
kept outside this repository (the user's decision, 2026-11-05: about 1.1 GB of third-party
files whose licenses are not stated), next to it:

- `C:\GIT\music\rb-collections\rb-mods\mods`: 320 `.rbm` files, 1.1 GB, at rb-mods fe08cb8
  (2026-10-31). They are recrypted (README, "Making mods work in OpenRB-338"), the form orb
  loads, so no `RB20FUL.DAT` is needed. The file names equal the 320 of
  `tests/refs/thirdparty_mods.json`, none missing and none extra.
- `C:\GIT\music\rb-collections\rb-songs\songs`: 1,289 files, of them 1,270 `.rbs` (1,268
  distinct songs, 531 of them on 43 third-party mods), at rb-songs 4af1075 (2026-10-31).

The project's own check (task-058, task-129; docs/rbm.md section 11) reads them in place:

```text
ORB_THIRDPARTY_MODS_DIR=C:/GIT/music/rb-collections/rb-mods/mods \
ORB_THIRDPARTY_SONGS_DIR=C:/GIT/music/rb-collections/rb-songs/songs \
    cargo test --release -p orb-cli --test thirdparty -- --nocapture --test-threads 1
```

In the UI, Mods > Choose file... on that folder, then "Use as mods folder", makes the Mods menu
list them.

### 5.1 Results (2026-11-05, Windows 11, main at cbca449)

- `mods` test: passed. Every one of the 320 recrypted mods parses, and its embedded files and
  46 sample slots hash as the manifest says.
- `songs` test: all 1,268 distinct songs (842 RBS 4.2, 426 ReBirth 1.x) rendered by orb hash
  equal to the original ReBirth.dll's renders in `tests/refs/thirdparty_songs.json`, including
  the 531 songs on third-party mods. The first run still failed on one check: 4 song files were
  reported as "not in the manifest". All 4 have an accented letter in their path (Ç, é, ü). The
  manifest holds these 4 paths decomposed (NFD: `Lokfu` + U+0308 `hrer`), while rb-songs' git
  stores them composed (NFC: `Lokf` + U+00FC `hrer`), and Windows and Linux check them out
  that way. The test compared the bytes. It now compares the NFC forms (`same_path` in
  `crates/orb-cli/tests/thirdparty.rs`, with the dev-dependency `unicode-normalization`).
  This is a defect of the test's data, not of orb or of the mods.
- The UI's own path, checked with a temporary headless test (not committed): with the folder as
  the mods folder, the Mods menu lists 311 entries for the 320 files (mods with equal names
  merge into one entry with variants, as task-151 designed). `App::use_kit` loads every one of
  the 320 files with no error. All 259 mods that carry a demo song open it with
  `App::open_demo_now` with no error, and the player gives nonzero audio within 8 seconds
  of Play. That audio can include the mod's startup sound, which is on by default, so this
  shows that the mod's samples play, not that the demo itself sounds right; the songs test
  above is the check of the sound. One mod
  name differs from its menu entry: `omega777.rbm` calls itself "omega" and sits as a variant
  under "Omega", because the names are compared ignoring ASCII case (`mods::names_match`).

## 6. Reference for developers

### 6.1 How egui 0.37 builds the accessibility tree

What follows was read in the egui 0.37.0 and AccessKit 0.25.2 sources this workspace uses
(`~/.cargo/registry/src/*/egui-0.37.0/src`), and checked with the test of 1.4.

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
- **Why the panel's rectangles are in the Tab order.** `ui.interact(rect, id,
  Sense::click_and_drag())` (`App::hit`) registers a widget that can take the focus:
  `Sense::click_and_drag` includes `FOCUSABLE` (`sense.rs:82`). egui then puts it in its Tab
  order and gives it a node with no role ("Unknown"; UI Automation: "Custom") and no name.
- **Actions from the screen reader.** A screen reader's request (Click, Focus, Increment,
  Decrement, SetValue, ...) arrives as `Event::AccessKitActionRequest` in the frame's input.
  egui handles Focus and Click itself for its own widgets (`context.rs:1579`); other actions
  are read by the widget with `ui.input(|i| i.accesskit_action_requests(id, action))`
  (`input_state/mod.rs:882`). A node advertises what it supports with `add_action`.
- **Keyboard focus.** egui moves its focus on Tab and Shift+Tab, and with the arrow keys
  (`memory/mod.rs:588` to 600: Up, Down, Left, Right without modifiers); Escape clears it. The
  panel's own keys (`App::keyboard`) read the same key events independently, which is the
  source of the collisions of section 1.3.
- **Announcements.** A node with `set_live(Live::Polite)` (or `Assertive`) is a live region;
  when its text changes, `accesskit_windows` raises UI Automation's LiveRegionChanged event
  (`adapter.rs:261` and 322), which NVDA speaks.
- **Roles that fit the panel.** AccessKit 0.25 has, among others, `Slider`, `SpinButton`,
  `CheckBox`, `RadioButton`, `RadioGroup`, `Button`, `Group`, `Region`, `Toolbar`, `Grid`,
  `Row`, `GridCell`, `Dialog`, `AlertDialog` (`accesskit-0.25.2/src/lib.rs`).
- **Deprecations in 0.37.** `egui::WidgetType` is now `egui::Role` (a re-export of
  `accesskit::Role`; some variants renamed, e.g. `TextEdit` is `TextInput`), and `Id::new` is
  replaced by `Id::unique` or `IdSalt::new`.
- **The web.** eframe 0.37's web runner ignores the AccessKit update
  (`eframe-0.37.0/src/web/app_runner.rs:403`, "not currently implemented").

### 6.2 A headless test of the tree

The measurements of 1.1 and 1.4 came from two throwaway tests. Task-183 (A11Y-0) turned them
into a shared helper, `crates/orb-ui/tests/common/a11y.rs`, used from a test file with
`#[path = "common/a11y.rs"] mod a11y;`: `AppTree` runs the `App` headless and keeps the last
tree (`tree()`, `key`, `send`), `Frames` runs any UI closure (a small test widget), `Tree`
finds nodes (`by_id`, `by_layout(kind, id)`, `by_name`, `by_role`, `focused`), and `request` /
`set_value_request` make the events a screen reader sends. `tests/a11y_actions.rs` shows an
Increment, Decrement and SetValue arriving in a widget; `tests/a11y_baseline.rs` records the
numbers of 1.1 and is updated by each later task. The core of it:

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
its main window, reads its UI Automation tree and prints the window's name, the number of
elements, the elements by type and by whether they have a name, how many can take the focus
and the names; `-Dump` lists every element; `-Arguments` passes other arguments (a song). It
uses a throwaway settings file (`ORB_UI_CONFIG`), so the user's `ui.toml` is untouched, and
it closes the program it started. Windows only; no screen reader needs to run.

```text
cargo build --release -p orb-ui
pwsh tools/uia_tree.ps1
```

Two things it handles that are easy to get wrong: the process has a hidden window of the same
class and title as the main window (the script takes the visible one), and the first UI
Automation request gets a placeholder element while AccessKit starts, so the script asks
again with a fresh element. The main window shows only after its first frame, about 10
seconds after a first start on the test machine.

Output on main at cbca449, 2026-11-05:

```text
window: 'OpenRB-338 - Untitled'
elements: 256
by type, named:
   250  Custom, False
     4  Button, True
     1  Menu, False
     1  Pane, False
can take the focus: 254, of them with a name: 4
names: File | Mods | Options | Help
```

Output after task-186 (A11Y-2: names, roles and values; 2026-11-05):

```text
window: 'OpenRB-338 - Untitled'
elements: 262
by type, named:
    86  Slider, True
    82  Button, True
    57  RadioButton, True
    23  CheckBox, True
    12  Text, True
     1  Menu, False
     1  Pane, False
can take the focus: 248, of them with a name: 248
names: Shuffle amount | Tempo up | Tempo down | Mode | Play | Stop | ... | Cutoff | ...
  | Channel 4 compressor | Master level | Master compressor | PCF on | ... | Threshold
  | File | Mods | Options | Help | Tempo 140 BPM | Bar off | Loop start off | ...
```

The 82 buttons are the 68 trigger, repeat and wind buttons (46 + 18 + 4), the 4 menu titles
and the 10 two-position switches (AccessKit's Switch role is a UI Automation Button whose value
is the position's word). The 86 sliders are the 77 knobs and 9 faders, the 57 radio buttons the
48 pattern and bank buttons and the 9 PCF and Comp switches, the 23 check boxes the toggles.
The 12 Text elements are the displays, which take no focus. The six click areas of the drum machines
(step strips, instrument selection) are no longer focusable and so no longer in the tree. The
names and roles are listed in docs/ui_spec.md section 20.

Output after task-188 (A11Y-4: sections; 2026-11-06), with `-ScreenReader on` (the parameter
writes `screen_reader = "on"` or `"off"` into the throwaway settings; the default, `auto`, is
on while NVDA runs, so the default run on the development machine gave this output too). With
`-ScreenReader off` the same run gave 198 elements: these without the 9 Groups, 194 that take
the focus, as before.

```text
window: 'OpenRB-338 - Untitled'
elements: 207
by type, named:
    86  Slider, True
    62  Button, True
    23  CheckBox, True
    14  Spinner, True
     9  Group, True
     9  RadioButton, True
     2  Text, True
     1  Menu, False
     1  Pane, False
can take the focus: 194, of them with a name: 194
names: Transport | Shuffle amount | Tempo | Mode | Play | ... | Bass Line 1 | Pattern section on
  | Pattern | ...
```

After task-190 (A11Y-6: the 303 step editors; 2026-11-06), `-ScreenReader on`: 207 elements,
as before, with 88 Slider (the two 303s' edit step Text elements are now the focusable
"Step editor" sliders, docs/ui_spec.md 20.5) and no Text; 196 take the focus, all named. With
the mode off the step editors stay the Text labels "Edit step N": 198 elements, 194 that take
the focus, as after task-188.

The nine Groups are the sections (docs/ui_spec.md 20.2), each before its controls in the tree.
The tree lists the controls in the drawing order; the Tab order of the mode is the sections'
(20.2).

Output after task-191 (A11Y-7: the drum grids; 2026-11-06), `-ScreenReader on`:

```text
window: 'OpenRB-338 - Untitled'
elements: 593
by type, named:
   384  DataItem, True
    86  Slider, True
    62  Button, True
    23  CheckBox, True
    14  Spinner, True
     9  Group, True
     9  RadioButton, True
     2  DataGrid, True
     2  Text, True
     1  Menu, False
     1  Pane, False
can take the focus: 196, of them with a name: 196
```

The two DataGrids are the drum grids ("Steps", focusable) and the 384 DataItems their cells (2
x 12 instruments x 16 steps, named "Bass drum, step 1, off", not focusable themselves: the
grid's active descendant is reported as the focus; docs/ui_spec.md 20.6). With the mode off
there are no grids and the tree is as before.

Output after TASK-195 and TASK-210 (main at 6cc85ab, 2026-11-07; the last tasks that change
the tree), `-ScreenReader on` and, with NVDA running, `auto`: the same 593 elements and types
as above with 88 Sliders (the step editors are sliders), and 198 that take the focus, all
named (the 194 controls, the two step editors, the two grids). `-ScreenReader off`: 198
elements (86 Sliders, 62 Buttons, 23 CheckBoxes, 14 Spinners, 9 RadioButtons, 2 Texts, the menu
and the pane), 194 that take the focus, all named. The mod chooser is a window, not in the
tree until it is open.

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
