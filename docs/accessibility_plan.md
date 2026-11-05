# Screen reader accessibility: the plan

Status: plan, 2026-11-05, nothing implemented. The analysis behind it, with the measurements
and the options, is in [accessibility.md](accessibility.md); read that first. This document
turns its section 3 into a milestone, decisions and tasks.

## The tasks in the backlog

Created on 2026-11-05 with the Backlog.md CLI (1.55.3) from this document, so their text is
the text below. The backlog is now the source of truth for status and progress.

- Milestone: m-7, "m-7: Screen reader accessibility (NVDA first)".
- Decisions, all Done: A11Y-D1 = TASK-177, A11Y-D2 = TASK-178, A11Y-D3 = TASK-179,
  A11Y-D4 = TASK-180, A11Y-D5 = TASK-181, A11Y-D6 = TASK-182.
- Work: A11Y-0 = TASK-183, A11Y-11 = TASK-184, A11Y-1 = TASK-185, A11Y-2 = TASK-186,
  A11Y-3 = TASK-187, A11Y-4 = TASK-188, A11Y-5 = TASK-189, A11Y-6 = TASK-190,
  A11Y-7 = TASK-191, A11Y-14 = TASK-192, A11Y-8 = TASK-193, A11Y-9 = TASK-194,
  A11Y-10 = TASK-195, A11Y-12 = TASK-196, A11Y-13 = TASK-197.
- The already-done test fix: TASK-176 (milestone m-5, Done).

`backlog task list -m m-7 --plain` lists them. On Windows, a script that passes multi-line
text to the CLI must run `node .../backlog.md/cli.js` directly: through npm's `backlog.cmd`
shim, cmd.exe cuts an argument at its first newline and drops the rest of the command line
(TASK-176 was created that way first, then repaired with `task edit`; the CLI sets references
only at creation, so its references are in its description).

## How to use this document

The repository plans in Backlog.md (`backlog/`, AGENTS.md "Backlog rules"), and its files are
changed only through the Backlog.md CLI or MCP tools, never by hand. The CLI was not available
when this plan was written, so the plan was written here first, so that each entry could be
created in the backlog as it stands (which was done; previous section):

- the milestone becomes `m-7` (`backlog milestone add`; its title starts with its id);
- each `Decide:` entry becomes a task with `--type spike -l decision,ui -a @dengopaiv`;
- each work entry becomes a task with the labels, type, priority, assignee, dependencies,
  references and acceptance criteria it lists;
- the CLI assigns the task numbers. Here the entries are named `A11Y-D1`..`A11Y-D6` (the
  decisions) and `A11Y-0`..`A11Y-14` (the work), and the dependencies use these names; replace
  them with the task numbers when the tasks are created;
- the ordinals continue after m-6's (`--ordinal 200000`, then steps of 1000 in the order below).

Once the tasks exist, the backlog is the source of truth and this document only explains the
plan; update it when the plan changes shape, not for each task's progress.

Each task description below is meant to be enough for an agent who has not seen the
conversation that produced it, together with [accessibility.md](accessibility.md) and the
files it references.

## Milestone m-7: screen reader accessibility (NVDA first)

**Goal.** A blind musician can use OpenRB-338 with NVDA on Windows, everything the panel does:
play and stop, set every knob, fader and switch, choose patterns, program the 303s and the
drum machines and read their patterns back, record in song mode, open, save and export songs,
choose mods, and read every message and window. All of it from the keyboard, with speech that
says what each control is, what its value is and what a key just did, and stays quiet while
the music plays unless asked.

**Scope.** The desktop UI (`crates/orb-ui`), on Windows with NVDA first; the same accessibility
tree serves Narrator and JAWS (UI Automation), VoiceOver (macOS) and Orca (Linux) through
AccessKit, with less testing. Not in scope: the engine (no change at all: the rendered audio
stays byte for byte the original's), the `orb` command line tool (already usable with a screen
reader), MIDI, and any web version: the fork is a local editor only and has none (the user,
2026-11-05, TASK-198; upstream's m-6 is archived here).

**Ground rules for every task of the milestone.**

- With screen reader mode off, the app behaves exactly as upstream, key for key, except where
  the user decided otherwise: the panel's keys wait while a menu is in use (TASK-202). The
  existing tests of `crates/orb-ui/tests` (keyboard.rs in particular) pass unchanged.
- Every accessible action goes through the same `model::Panel` method as the mouse or the
  original's key (`press`, `arrow`, `set_knob`, `set_fader`, `set_switch`,
  `select_instrument`, `set_step808`, `set_step909`, `tap_303`, `tap_drum`), so the events
  the engine hears, recording in song mode included, are the same.
- The accessible layer lives in `crates/orb-ui/src/a11y/`, with small hooks elsewhere.
- Nothing is announced by itself during playback (position, meters, LEDs, running lights);
  what the user does is confirmed.
- Each task adds headless tests of the accessibility tree (the harness of A11Y-0) for what it
  builds, updates docs/ui_spec.md and Help > Keyboard Shortcuts (`crates/orb-ui/src/key_help.rs`)
  for what it changes, and checks the frame cost with `crates/orb-ui/tests/frame_cost.rs`
  (which A11Y-2 extends to measure with AccessKit on as well; today it draws without it).
- Lines of Rust at most 100 columns, `cargo clippy --all-targets -- -D warnings` clean.

**Done when.** All tasks below are Done, and A11Y-13 (the NVDA pass by the user) found nothing
open.

## The order

The user asked for everything (2026-11-05); the order follows what each task needs.

1. Decisions: A11Y-D1 to A11Y-D6 (the user's).
2. Foundation: A11Y-0 (test harness), A11Y-1 (mode setting), A11Y-2 (names and roles), A11Y-3
   (sliders, switches, spin buttons, pattern selector), A11Y-4 (sections, focus, key
   conflicts), A11Y-5 (announcements and read-out keys). After these, the whole panel can be
   operated and heard, except for the contents of the patterns.
3. Patterns and units: A11Y-6 (303 step editor), A11Y-7 (drum grids), A11Y-14 (physical
   units for the knobs and faders, from the engine).
4. Files and windows: A11Y-8 (dialogs and windows), A11Y-9 (system file dialogs), A11Y-10
   (mod chooser), A11Y-11 (menu bar from the keyboard).
5. Wrap-up: A11Y-12 (documentation), A11Y-13 (NVDA pass).

Dependencies, in short: A11Y-1 needs D1. A11Y-2 needs A11Y-0 and D5. A11Y-3 needs A11Y-2. A11Y-4
needs A11Y-1, A11Y-3, D2, D3 and D6. A11Y-5 needs A11Y-2 and D6. A11Y-6 and A11Y-7 need A11Y-4
and A11Y-5. A11Y-8 needs A11Y-5. A11Y-9 needs A11Y-1 and D4. A11Y-10 needs A11Y-8. A11Y-11 needs
A11Y-0. A11Y-14 (physical units) needs A11Y-2 and D5. A11Y-12 needs all of A11Y-1 to A11Y-11
and A11Y-14. A11Y-13 needs A11Y-12.

A11Y-0, A11Y-11 and the decisions can start at once and run side by side.

## Already done

### Third-party songs test: compare song paths in NFC (TASK-176, Done, commit 4d76af3)

- Type: bug. Labels: tests. Priority: medium. Milestone: m-5 (maintenance). Status: Done.
- References: `crates/orb-cli/tests/thirdparty.rs`, `crates/orb-cli/Cargo.toml`,
  `tools/rbhost/thirdparty_songs.py`, `tests/refs/thirdparty_songs.json`, docs/rbm.md
  section 11.5.
- What: on Windows (and Linux) the gated `songs` test reported 4 song files of rb-songs as not
  in the manifest, all with an accented letter in the path. The manifest holds those paths
  decomposed (NFD), rb-songs' git stores them composed (NFC). The test now compares the NFC
  forms (`same_path`, dev-dependency `unicode-normalization` 0.1), with a unit test; the
  manifest tool writes NFC paths from now on (`nfc`), and still opens files by the file
  system's path.
- Result: `cargo test --release -p orb-cli --test thirdparty` with
  `ORB_THIRDPARTY_MODS_DIR` and `ORB_THIRDPARTY_SONGS_DIR` set: 3 passed (mods: 320 mods; songs:
  1268 distinct songs equal to rbhost's renders, 531 on 43 third-party mods), 473 s, Windows 11,
  2026-11-05. Before: `songs` failed on the 4 paths, every render equal.

## Decisions

Each is a task titled as below, `--type spike -l decision,ui -a @dengopaiv`, milestone m-7,
priority high. Its acceptance criterion is always the same: the user has chosen, and the final
summary records the choice and the date.

**All six were decided by the user on 2026-11-05**, before the tasks were created. Each entry
below keeps its options and adds a "Decided" line; when the tasks are created in the backlog,
each decision task is created and closed in the same commit, its final summary being that
line (`--final-summary`, `-s Done`, its acceptance criterion checked). The work tasks below
already follow the decisions.

In short: screen reader mode turns on by itself, with an Options menu override (D1). Space
presses or toggles the focused control, F5 is Play/Stop, Enter keeps the original's meaning
(D2). A machine's mixer channel is in the machine's section (D3). The system's file dialogs
(D4). Values in physical units, as precise as the engine's code allows (D5). The function keys
F1 to F12 as in D6, the 303 tap on Ctrl+Tab and Ctrl+Shift+Tab (D6).

### A11Y-D1. Decide: how screen reader mode turns on

- References: `crates/orb-ui/src/settings.rs`, `crates/orb-ui/src/menu.rs` (Options menu),
  docs/accessibility.md sections 3.1 and 4.
- Why: the accessible layer changes a few keys (A11Y-4) and the file dialogs (A11Y-9), so it is
  a mode that must be on for screen reader users and off for everyone else, without the user
  having to find a setting they cannot hear.
- Options:
  - A. On by itself when the system reports a screen reader (Windows: the screen reader flag,
    `SystemParametersInfo(SPI_GETSCREENREADER)`, which screen readers are meant to set; that
    NVDA sets it is to be checked in A11Y-1, and if it does not, the detection falls back to
    looking for NVDA's process or its controller DLL's answer; macOS: VoiceOver's state; Linux:
    the AT-SPI "screen reader enabled" property), plus an Options menu item "Screen Reader Mode"
    with three states (automatic, on, off) kept in `ui.toml`.
  - B. Only by hand, from the Options menu, kept in `ui.toml`.
  - C. Always on: no mode at all.
- Recommendation: A. C would change the keys for every user and could not go upstream.
- **Decided (2026-11-05): A.** Automatic when the system reports a screen reader, with the
  Options menu item "Screen Reader Mode" (automatic, on, off), kept in `ui.toml`.

### A11Y-D2. Decide: Space and Enter on a focused control

- References: `crates/orb-ui/src/keys.rs` (Space is Play/Stop, `keys.rs:141`), docs/ui_spec.md
  section 10.
- Why: in Windows dialogs Space presses or toggles the focused control; in ReBirth, and in
  every music program, Space plays and stops.
- Facts found while deciding: no function key (F1 to F12) is used by orb-ui (no `Key::F..` in
  `crates/`) or by the original's key map (docs/ui_spec.md section 10 names none), so all twelve
  are free. Enter is the 303's Step key in Program Synth mode with a 303 focused, pressed after
  every programmed note, and Play everywhere else (`keys.rs:158` and 188).
- Options: A. Space stays Play/Stop everywhere; Enter presses or toggles the focused control.
  B. Space toggles the focused control as in dialogs; Play/Stop moves to another key. Asked
  again once the function keys were known to be free: Space and F5 both Play/Stop with Enter
  toggling; Space toggling with F5 Play/Stop; Space Play/Stop with F5 left free. And for Enter:
  keep the original's meaning; press focused buttons only; always press the focused control.
- Recommendation: A at first; then, for Enter, keep the original's meaning, so that
  programming a 303 works as in ReBirth.
- **Decided (2026-11-05): B, with F5 as Play/Stop; Enter keeps the original's meaning.** In
  screen reader mode Space presses or toggles the focused control (a button, a toggle, a
  switch, a step of a drum grid) and is no longer Play/Stop; F5 is Play/Stop. Enter does what
  it does in the original whatever has the focus: the 303's Step in Program Synth mode with a
  303 focused, else Play. With the mode off, Space is Play/Stop as before.

### A11Y-D3. Decide: where a machine's mixer channel is in the navigation

- References: `assets/ui/layout.tsv` (sections `mixer1`..`mixer4`), docs/accessibility.md 3.2.
- Options: A. In the machine's own section (Bass Line 1's section ends with channel 1's level,
  pan, delay send, PCF, compressor, distortion and mute). B. A Mixer section with the four
  channels one after the other.
- Recommendation: A: in ReBirth a row is one machine with its channel, and the row focus that
  directs the keys follows the section.
- **Decided (2026-11-05): A.** Each machine's section ends with its mixer channel.

### A11Y-D4. Decide: system file dialogs in screen reader mode

- References: `crates/orb-ui/src/browser.rs` (the in-app browser; its rows are painted,
  `browser.rs:881`), `rfd` 0.17.2 (already a dependency, `crates/orb-ui/Cargo.toml`).
- Options: A. In screen reader mode, Open, Save As, Export and Mods > Choose file use the
  system's dialog (`rfd::FileDialog`); the browser's "Use as songs folder" and "Use as mods
  folder" become Options menu items with a folder dialog. B. Make the in-app browser
  accessible (named list items, roles, focus) and keep it in both modes.
- Recommendation: A: the Windows dialog is already fully accessible and familiar; B is more
  work for a worse result. The in-app browser stays as it is with the mode off.
- **Decided (2026-11-05): A.** The system's dialogs in screen reader mode, the folder choices
  in the Options menu; the in-app browser unchanged with the mode off.

### A11Y-D5. Decide: how values are spoken

- References: `crates/orb-ui/src/behavior.rs` (`knob_send`, `fader_send`: 0..127),
  `crates/orb-ui/src/model.rs` (`display`).
- Options: A. The 0 to 127 value the original stores, with words for switches ("square"),
  pan ("center", "left 20") and the displays' own units (BPM, steps, bar). B. Physical units
  (Hz, ms, dB) for the synth and effect knobs. C. Percent.
- Recommendation: A: the original panel shows no units, so B's numbers would be estimates; A
  is exact and matches what a song file holds.
- **Decided (2026-11-05): B, made as precise as the code allows.** The user's words: "We can,
  though these are estimates, still have them as precise as possible as we infer them from the
  code." The engine is a bit-exact reimplementation, so the mapping from each 0..127 value to
  its DSP parameter is in `crates/orb-engine` (for example the 303 filter's coefficient table
  `FILT[reso >> 4][cutoff]`, `tables/mod.rs:43`; the decay multipliers per sample pair
  `TR808_DECAY`, `PCF_DECAY`, `tables/data.rs`). Each unit is derived from there and marked by
  how exact it is: computed in closed form from the engine's formula, derived numerically from
  its tables or by measuring its output, or estimated. A quantity with no physical unit
  (resonance, distortion shape) is spoken as a percent of its range. Switches, pan and the
  displays keep their words and units. The work is task A11Y-14; until it lands, the value
  text is the 0..127 number. The node's numeric value stays 0..127 (what Increment and
  SetValue act on).

### A11Y-D6. Decide: the key map of screen reader mode

- References: `crates/orb-ui/src/keys.rs`, `crates/orb-ui/src/shortcuts.rs`,
  docs/accessibility.md sections 1.3 and 3.4.
- Why: four keys collide with NVDA or with focus navigation (Tab, Up/Down, Insert, the
  keypad with Num Lock off), and the new functions need keys.
- Proposal to decide on:
  - F6 / Shift+F6: next / previous section. Tab / Shift+Tab: next / previous control.
  - Up, Down, Page Up, Page Down, Home, End: change the focused value. Left / Right: move in the
    step editor and the grids. Enter: press or toggle (with D2's answer).
  - The 303 tap keys move from Tab / Shift+Tab to backslash / Shift+backslash.
  - Record also gets Ctrl+R (Insert keeps working when it is not the NVDA key).
  - The row focus keys (Up/Down of the original) are replaced by the section moves.
  - F2: read the focused section's pattern. F3: read the transport. F4: read the levels.
  - Ctrl+Up / Ctrl+Down: the original's row focus, for users who want it without moving the
    navigation focus.
- Recommendation: the proposal as it stands; the user changes what does not suit their NVDA
  setup.
- **Decided (2026-11-05).** The function keys take the transport (with D2), in the layout the
  user chose, "transport on F9 to F12": the read-outs low, the transport high and away from F6,
  so a slip while moving between sections does not start the music. The 303 tap keys are
  Ctrl+Tab and Ctrl+Shift+Tab, close to the original's Tab. The key map of screen reader mode:
  - F1: Help > Keyboard Shortcuts.
  - F2: read the focused section's pattern. F3: read the transport. F4: read the levels.
  - F5: Play / Stop.
  - F6 / Shift+F6: next / previous section.
  - F7, F8: free.
  - F9: Record on / off. (Insert keeps working for users whose NVDA key it is not.)
  - F10: the menu bar (and Alt pressed and released alone, A11Y-11).
  - F11: Loop on / off.
  - F12: song / pattern mode.
  - Tab / Shift+Tab: next / previous control.
  - Space: press or toggle the focused control (D2).
  - Enter: the original's meaning, whatever has the focus (D2).
  - Up, Down, Page Up, Page Down, Home, End: change the focused value. Left / Right: move in
    the step editor and the grids.
  - Ctrl+Up / Ctrl+Down: the original's row focus, without moving the navigation focus.
  - Ctrl+Tab / Ctrl+Shift+Tab: tap a note / a rest into the playing step of the focused 303
    (the original's Tab / Shift+Tab). Note that `App::keyboard` today skips every key pressed
    with Ctrl (`app.rs:725`, the menu shortcuts), so this key needs its own handling; egui does
    not move its focus on Ctrl+Tab (`memory/mod.rs:595`, Tab only without modifiers).
  - Every other key of the original (the pitch, drum and pattern keys, minus, equals, the
    digits, the keypad) keeps its meaning, whatever has the focus.

## Work

### A11Y-0. Accessibility test harness and the UI Automation probe

- Assignee: @agent-worker. Type: task. Labels: tests, ui. Priority: high. Dependencies: none.
- References: `crates/orb-ui/tests/keyboard.rs` (the headless harness style),
  `tools/uia_tree.ps1`, docs/accessibility.md section 1.1, egui 0.37 `Context::enable_accesskit`,
  `FullOutput::platform_output.accesskit_update`, `Event::AccessKitActionRequest`.
- Why: every later task proves its result on the accessibility tree; they need one shared way to
  build it headless, find nodes and send the actions a screen reader sends.
- What: a test helper module `crates/orb-ui/tests/common/a11y.rs` (used through `#[path]` or
  `mod common` as the other tests do): run frames of an `App` with AccessKit enabled and keep
  the last `TreeUpdate`; find nodes by name, by role, by layout id (the node id is derived from
  the egui id); read name, role, value, numeric value, toggled state and supported actions;
  the focused node; send an `ActionRequest` (Focus, Click, Increment, Decrement, SetValue with
  a number) as an input event; press keys as `keyboard.rs` does. A baseline test
  `crates/orb-ui/tests/a11y_baseline.rs` that records today's state (261 nodes, 4 named) as the
  starting point and is updated, not deleted, as tasks land. `tools/uia_tree.ps1` (committed
  already) stays the Windows check of the real window.
- Acceptance criteria:
  1. The helper builds the tree of `App` headless in `cargo test` on Windows, macOS and Linux
     (CI), without a window or a GPU.
  2. A test sends an Increment request to a node and sees it arrive in the app
     (`InputState::accesskit_action_requests`), with a small test widget.
  3. The baseline test passes and documents what it measures.
  4. `tools/uia_tree.ps1`'s use is described in docs/accessibility.md.

### A11Y-1. Screen reader mode: the setting and the detection

- Assignee: @agent-worker. Type: feature. Labels: ui. Priority: high. Dependencies: A11Y-D1.
- References: `crates/orb-ui/src/settings.rs`, `crates/orb-ui/src/menu.rs` (the Options menu,
  `menu.rs:1214`), `crates/orb-ui/src/main.rs` (the Win32 FFI style of `attach_parent_console`),
  docs/ui_spec.md section 18 (menus).
- What (with D1's recommendation): a setting `screen_reader` in `ui.toml` with the values
  `auto` (default), `on`, `off`; an Options menu item "Screen Reader Mode" with the three
  choices; `App::screen_reader_mode()` true when on, or auto and the system reports a screen
  reader. Detection on Windows through `SystemParametersInfoW(SPI_GETSCREENREADER)` by FFI
  (no new crate), checked at start and when the window regains the focus; on macOS and Linux
  `false` in this task (a later task can add VoiceOver and AT-SPI detection; note it as a
  follow-up task if not done). Nothing else changes yet: later tasks read the mode.
- Acceptance criteria:
  1. The setting round-trips through `ui.toml` (a test in the style of
     `crates/orb-ui/tests/settings.rs`), and an old `ui.toml` without the key reads as `auto`.
  2. The menu item shows and changes the setting; with `off` no behavior differs from main.
  3. On Windows with NVDA running, `auto` reports the mode on; without a screen reader, off
     (checked by hand, recorded in the notes with the NVDA version).
  4. docs/ui_spec.md section 18 lists the item.

### A11Y-2. A name, a role and a value for every panel control

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: high.
  Dependencies: A11Y-0, A11Y-D5.
- References: `assets/ui/layout.tsv` (348 parts: id, section, kind), `crates/orb-ui/src/app.rs`
  (`App::control` 813, `App::hit` 892, `knob` 950, `fader` 991, `drum_strips` 1020,
  `instrument_selection` 1091), `crates/orb-ui/src/model.rs` (`value`, `knob`, `fader`,
  `switch_position`, `display`, `toggle_on`), docs/ui_spec.md (the names of the controls as the
  original's manual gives them), docs/accessibility.md sections 1.2 and 3.3.
- Why: today 250 of the 254 focusable elements have no name and no role
  (docs/accessibility.md 1.1).
- What: `crates/orb-ui/src/a11y/names.rs`: for each layout id its spoken name ("Cutoff",
  "Bass drum level", "Channel 3 mute") and its section's name ("Bass Line 1", "TR-808"), as a
  table checked against `layout.tsv` by a test; value text per kind (the 0..127 number for
  knobs and faders until A11Y-14 gives their units, words for switches and pan, the displays'
  units); roles: Slider (knob,
  fader), CheckBox or ToggleButton (toggle), RadioButton (radio), Button (trigger), the switch
  as a two-state control with its two words. Non-interactive parts (LEDs, meters, the step
  parts, the click areas, the mod picture) get no focus: the panel's `ui.interact` for them
  uses a sense without focus, or they get no node. The node data is written with
  `Context::accesskit_node_builder` or `Response::widget_info` in the place each control is
  drawn. Note that egui 0.37 deprecates `WidgetType` (now `egui::Role`) and `Id::new`.
- Acceptance criteria:
  1. A test walks `layout.tsv`: every interactive part (every kind except led, meter, steps,
     area, modview, display) has a node with a non-empty name, the role of its kind and, for a
     value control, a value; no two nodes of one section share a name.
  2. No LED or meter node can take the focus; the number of focusable nodes is recorded in
     the baseline test (A11Y-0) and is lower than today's 254.
  3. `tools/uia_tree.ps1` on the release build shows the names (recorded in the notes).
  4. `crates/orb-ui/tests/frame_cost.rs` also measures a frame with AccessKit on (it enables
     it on its context; today it does not) and prints the time of building the tree; the
     shape, mesh and vertex bounds it asserts still hold, and the times before and after are in
     the notes.
  5. With screen reader mode off, the keyboard and interaction tests pass unchanged.

### A11Y-3. Sliders, switches, spin buttons and the pattern selector that the reader can drive

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: high. Dependencies: A11Y-2.
- References: `crates/orb-ui/src/model.rs` (`set_knob`, `set_fader`, `set_switch`, `press`,
  `arrow`, `Row::lit_button`; a row has 32 patterns in 4 banks of 8),
  `crates/orb-ui/src/behavior.rs` (`knob_from_engine`, `knob_send`, `fader_send`), the repeat
  buttons and displays of `layout.tsv`, docs/accessibility.md section 3.3.
- What:
  - Knobs and faders: numeric value 0..127, min, max, step 1, actions Increment, Decrement and
    SetValue; a request sets the value through `set_knob(id, knob_from_engine(v))` (faders:
    the fader position whose `fader_send` is `v`).
  - Each display with its up and down repeat buttons becomes one SpinButton (tempo, bar, loop
    start, loop length, each row's pattern length, delay steps, PCF pattern) whose Increment
    and Decrement call `Panel::arrow` with the button's id, as one click does; the separate
    buttons and the display leave the focus order.
  - Each row's 8 pattern buttons and 4 bank buttons become one selector, "Pattern B3", whose
    Increment and Decrement move through the 32 patterns with the same `press` calls a click on
    the bank and the pattern button makes.
  - With the focus on such a control, Up/Down, Page Up/Page Down (8 for sliders, 1 bank for the
    pattern selector), Home and End change it (in screen reader mode; A11Y-4 does the routing
    of keys, this task the operations).
- Acceptance criteria:
  1. For each kind, a test sends the requests and compares the engine events (`Panel::events`)
     with those of the same change made by mouse in `crates/orb-ui/tests/interaction.rs`'s way:
     equal.
  2. A selection made with the pattern selector during song-mode recording is recorded like a
     click (a test with a link, as `crates/orb-ui/tests/wired.rs`).
  3. Values read back as words or numbers ("64" until A11Y-14, "center", "128 BPM", "Pattern
     B3"), through one function that A11Y-14 extends with the units.
  4. The tree has one node per spin button and per pattern selector, not the parts.

### A11Y-4. Sections, the focus order and the key conflicts

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: high.
  Dependencies: A11Y-1, A11Y-3, A11Y-D2, A11Y-D3, A11Y-D6.
- References: `crates/orb-ui/src/app.rs` (`App::show` 584, `App::keyboard` 704, `App::act` 747),
  `crates/orb-ui/src/keys.rs` (`map`; Tab 165, Up/Down 142, Insert 144),
  egui's focus (`Memory::move_focus`; egui moves the focus on Tab and the arrows itself),
  docs/ui_spec.md section 10, docs/accessibility.md sections 1.3, 3.2 and 3.4.
- What, in screen reader mode only:
  - The sections of docs/accessibility.md 3.2 as Group nodes with names, in that order; F6 and
    Shift+F6 move the focus to the first control of the next or previous section; Tab and
    Shift+Tab move within the panel in section order (wrapping at the ends of the panel).
  - The row focus follows the section the focus is in (the two 303s, the 808, the 909), and the
    change is announced through A11Y-5's announcer when that exists, else by the section name.
  - The key map of D6: Tab no longer taps the 303 (Ctrl+Tab and Ctrl+Shift+Tab do), Space
    presses or toggles the focused control and is no longer Play/Stop (D2), F5 is Play/Stop,
    F9 Record, F11 Loop, F12 song/pattern mode (each the same `Panel::press` as the panel's
    button), Enter keeps the original's meaning, Up/Down
    change the focused value instead of the row focus, Ctrl+Up/Down move the row focus.
  - egui's own arrow-key focus movement is suppressed on the panel (the arrows change values).
  - The original's play and program keys (letters, digits, minus, equals, Enter in Program
    Synth mode) keep working, whatever control has the focus.
  - Auto-repeat of Up/Down/Page Up/Page Down on a focused value control is honored (the
    original ignores repeats for its own keys; that stays).
- Acceptance criteria:
  1. Tests: F6 visits the sections in order and wraps; Tab visits the controls of the panel in
     section order; the focused node after each step has the expected name.
  2. Tests: while a 303 plays, Tab moves the focus and does not change the pattern; Ctrl+Tab
     taps the playing step as Tab did (Ctrl+Shift+Tab a rest); F5, F9, F11 and F12 give the
     same engine events as the panel's Play/Stop, Record, Loop and mode buttons; Space on a
     focused toggle toggles it and does not play; Enter with a 303 focused in Program Synth
     mode is the 303's Step.
  3. With the mode off, `crates/orb-ui/tests/keyboard.rs` passes unchanged, and a test shows
     Tab and Up/Down behaving as on main.
  4. docs/ui_spec.md section 10 has a "Screen reader mode" part with the key map, and Help >
     Keyboard Shortcuts lists the mode's keys when it is on.

### A11Y-5. Announcements and the read-out keys

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: high.
  Dependencies: A11Y-2, A11Y-D6.
- References: `crates/orb-ui/src/app.rs` (`status_bar` 307: painted text),
  `crates/orb-ui/src/menu.rs` (where `status` is set: save, export, refresh mods), AccessKit
  `Node::set_live(Live::Polite)`, `accesskit_windows-0.35.2/src/adapter.rs:261`
  (LiveRegionChanged), `crates/orb-ui/src/model.rs` (`Transport`, `meters`, `gr_leds`,
  `playing_step*`).
- What: `crates/orb-ui/src/a11y/announce.rs`: one node, polite live region, outside the focus
  order, whose text is the last message (repeated messages must be spoken again: add a
  changing invisible part or clear and set in two frames; test with NVDA which works). It
  speaks: the status line's messages; a song opened (its name and mod); a mod chosen; the
  keyboard mode changed (Ctrl+F, Ctrl+G); what an original key did when the focus is not on the
  control it changes (pattern selected with the pattern keys, tempo up/down, Record, Loop, mode,
  Play/Stop with F5 or Enter, Record with F9). The status line itself also gets a node with its
  text. Read-out
  keys (D6): F2 reads the focused section's pattern (A11Y-6 and A11Y-7 fill in the synths and
  drums; here the pattern row: "Pattern B3, 16 steps, shuffle off"), F3 the transport ("Playing,
  bar 5, 128 BPM, song mode, loop 1 to 8, recording off"), F4 the levels (master left and right,
  compressor gain reduction; in dB where A11Y-14 finds the engine's scale, else as numbers).
- Acceptance criteria:
  1. Tests: each listed event sets the announcer's text; nothing sets it during 10 seconds of
     playback without input (position, meters and LEDs moving).
  2. Tests: F2, F3 and F4 set the expected texts on a known song.
  3. With NVDA, the announcements are spoken (by hand; notes record the NVDA version and what
     was heard), including a message repeated twice.

### A11Y-6. The 303 step editor, spoken

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: medium.
  Dependencies: A11Y-4, A11Y-5.
- References: `crates/orb-ui/src/model.rs` (`Synth303`: `edit`, `pitch_mode`, `patterns`,
  `move_edit`, `toggle_pitch`, `step`; `Step303`: note, slide, accent, up, down, rest),
  `crates/orb-ui/src/keys.rs` (the 303 keys: pitch keys `C F V G B N J M K , L . /`, P accent,
  `[` slide, `;` down, `'` up, `-`/`=` note/pause, Enter step, Backspace back),
  docs/ui_spec.md section 10 (Program Synth) and the 303 section (pitch and time modes).
- What: in each 303 section, a "Step editor" node (focusable) whose value is the step being
  edited ("Step 5 of 16: C sharp, up, accent, slide", "Step 6: rest") and the editor's mode
  (pitch or time mode as the original has them). After each programming key, the step's new
  state is spoken (the node's value changes; if NVDA does not speak value changes of the
  focused node reliably, the announcer). Left and Right move the edit step where the original
  allows (`move_edit`), Home and End to the first and last step. F2 on a 303 section reads the
  whole pattern, one step after another, compactly ("1 C accent, 2 rest, 3 D sharp slide,
  ..."), in the pattern's length.
- Acceptance criteria:
  1. Tests: program a pattern with the keys alone, read it back with F2, and compare with the
     pattern written by the mouse in the same steps: equal events and equal text.
  2. The note names follow the original's keyboard (C to high C, with "sharp"), and up/down
     are spoken as octave up/down.
  3. A short how-to in docs/ui_spec.md: programming a 303 pattern with a screen reader.

### A11Y-7. The drum grids, navigable and readable

- Assignee: @agent-complex. Type: feature. Labels: ui. Priority: medium.
  Dependencies: A11Y-4, A11Y-5.
- References: `crates/orb-ui/src/model.rs` (`inst808`, `inst909`, `steps808`, `steps909`,
  `select_instrument`, `set_step808`, `set_step909`, `step909_new_value`),
  `crates/orb-ui/src/behavior.rs` (`tr909_step_value`: Ctrl, Alt, flam), `crates/orb-ui/src/app.rs`
  (`drum_strips` 1020, `instrument_selection` 1091), the 808 slot switches (`tr808.slot3_switch`
  .. `slot7_switch`: which voice a slot plays), docs/ui_spec.md sections on the 808 and 909.
- What: in each drum section, a Grid node with 12 rows (the instruments, AC first, named as
  the panel's legend: "Accent", "Bass drum", ...; the 808's switchable slots by the voice they
  play now) and 16 columns (the steps, up to the pattern's length). Up and Down choose the
  instrument (`select_instrument`, so the panel's selection follows), Left and Right the step,
  Space sets the step as a click does (D2; 808: on/off; 909: the click cycle, with Ctrl, Alt and
  Ctrl+Alt as the mouse's modifiers, and the flam button's effect). Each cell reads "Bass drum,
  step 5, on" ("accent", "flam" on the 909). F2 on a drum section reads each instrument that
  has steps, with its steps ("Bass drum: 1, 5, 9, 13. Closed hi-hat: 3, 7, 11, 15").
- Acceptance criteria:
  1. Tests: setting steps through the grid gives the same events as clicks on the same steps,
     on both machines and with each 909 modifier.
  2. Tests: F2 reads a known pattern correctly; steps past the pattern length are not offered.
  3. The selection frame on the panel follows the grid's row (visible for sighted helpers).

### A11Y-8. Dialogs and windows: roles, focus and announcements

- Assignee: @agent-worker. Type: feature. Labels: ui. Priority: medium. Dependencies: A11Y-5.
- References: `crates/orb-ui/src/menu.rs` (`prompt_ui` 997, the unsaved-changes prompt),
  `crates/orb-ui/src/error_box.rs`, `crates/orb-ui/src/info.rs` (File > Info),
  `crates/orb-ui/src/about.rs`, `crates/orb-ui/src/key_help.rs`, `crates/orb-ui/src/mod_info.rs`.
- What: each window gets a Dialog role and its title as name (the error box: AlertDialog);
  when it opens, the focus moves into it (the prompt: to Save; the error box: to OK; Info: to
  the first field; About, Keyboard Shortcuts, Mod Info: to the window's text or its close
  button), and when it closes the focus returns where it was. The error box's message is read
  when it opens. The texts of About, Keyboard Shortcuts and Mod Info can be read line by line
  (labels in a scrollable region, reachable with Tab or the arrows).
- Acceptance criteria:
  1. Tests: for each window, the focused node after it opens and after it closes.
  2. With NVDA, each window is announced with its title and content when it opens (by hand;
     notes).
  3. With the mode off, the windows behave as on main (their tests pass unchanged).

### A11Y-9. System file dialogs in screen reader mode

- Assignee: @agent-worker. Type: feature. Labels: ui. Priority: medium.
  Dependencies: A11Y-1, A11Y-D4.
- References: `crates/orb-ui/src/browser.rs` (`Mode`: OpenSong, SaveSong, Export, ChooseMod;
  "Use as songs folder", "Use as mods folder"), `crates/orb-ui/src/app.rs` (`open_browser` 262,
  `browse` 277), `crates/orb-ui/src/menu.rs` (`chosen`), `crates/orb-ui/src/settings.rs`
  (`songs_dir`, `mods_dir`, `last_*_dir`), `rfd` 0.17.2 (`FileDialog::pick_file`, `save_file`,
  `pick_folder`; default features off in `crates/orb-ui/Cargo.toml`: check that the Windows
  dialog needs no feature, and what Linux needs).
- What (D4's recommendation): in screen reader mode, `open_browser` opens the system dialog for
  the mode instead (filters: songs `.rbs`, mods `.rbm`, export WAV/FLAC/Ogg as the browser
  offers), starting in the folder the browser would start in, and passes the result to the same
  `chosen` path. Options menu items "Set Songs Folder..." and "Set Mods Folder..." with a folder
  dialog, doing what the browser's "Use as ... folder" buttons do. The dialog blocks the frame
  loop while open; the audio thread keeps playing (check, and note it).
- Acceptance criteria:
  1. In screen reader mode each of the four menu commands opens the system dialog, and a chosen
     file opens, saves, exports or loads exactly as through the browser (by hand on Windows;
     the shared `chosen` path is tested headless).
  2. The folder items change `ui.toml` as the browser's buttons do (test).
  3. With the mode off, the in-app browser opens as before.

### A11Y-10. A mod chooser with a filter

- Assignee: @agent-worker. Type: feature. Labels: ui, rbm. Priority: medium.
  Dependencies: A11Y-8.
- References: `crates/orb-ui/src/menu.rs` (the Mods menu 1137; `list_mods` 195; `ModEntry`,
  `ModVariant`; `preferred_variant`; `use_kit`; `choose_variant`; `has_demo`),
  `crates/orb-ui/src/mod_info.rs`, docs/rbm.md section 11.3 (the menu's order and duplicate
  names).
- Why: with the user's mods folder the Mods menu holds 311 entries (the 320 files of the rb-mods
  collection; equal names merge with variants), in a scroll area inside a pop-up menu. Walking
  it with the arrows is slow, and egui menus have no type-ahead.
- What: in screen reader mode, the Mods menu's first item is "Choose Mod..." (with a shortcut,
  to decide in the task and add to the key help), opening a dialog with a filter field and a
  list. The list holds the entries of the menu in the menu's order, each read as its name, then
  "variants: N" when it has variants, "has demo" when it has a demo song, and "current" for the
  playing kit. Typing filters by name (ASCII case ignored, as the menu sorts). Enter chooses
  (`use_kit`, or for a merged name the preferred variant, with the variants reachable as
  sub-items), Escape closes, a key opens Mod Info for the selected entry. The built-in
  "Standard ReBirth" is the first entry.
- Acceptance criteria:
  1. Tests with a folder of test mods (the existing test mods, or `ORB_THIRDPARTY_MODS_DIR` gated
     with a skip message): the list equals the menu's entries in order; the filter narrows it;
     Enter loads the same kit as the menu item.
  2. With the rb-mods collection, the dialog opens with all 311 entries in under 100 ms after
     the list is cached (measure; notes).
  3. With NVDA, the list and the filter are announced as such (by hand; notes).

### A11Y-11. The menu bar from the keyboard

- Assignee: @agent-worker. Type: task. Labels: ui. Priority: medium. Dependencies: A11Y-0.
- References: `crates/orb-ui/src/menu.rs` (`menu_bar` 1099, `egui::MenuBar`), egui 0.37's menu
  code (`containers/menu.rs`), docs/ui_spec.md section 18.
- What: find out how egui 0.37's menu bar is reached and walked from the keyboard (today the
  four menu titles are focusable buttons; whether Alt or F10 focuses the bar, whether the arrow
  keys open and walk a menu, whether Escape returns). Make F10 (and Alt pressed and released
  alone, on Windows) focus the File menu, the arrows walk the bar and the items, Enter choose,
  Escape close, in both modes if harmless, else in screen reader mode. Checkboxes and radio
  items in the Options menu read their state.
- Acceptance criteria:
  1. Tests: F10 focuses File; Right moves to Mods; Down opens it and focuses its first item;
     Escape closes; Enter on an item runs it.
  2. The findings about egui's own behavior are in the notes and in docs/ui_spec.md section 18.

### A11Y-12. Documentation of screen reader mode

- Assignee: @agent-worker. Type: docs. Labels: docs, ui. Priority: medium.
  Dependencies: A11Y-1 to A11Y-11, A11Y-14.
- References: README.md, docs/ui_spec.md, docs/accessibility.md, this plan,
  `crates/orb-ui/src/key_help.rs`.
- What: a README section "Using OpenRB-338 with a screen reader" (turning the mode on, the
  sections, the keys, programming a pattern, choosing a mod, known limits); docs/ui_spec.md's
  "Screen reader mode" section complete and consistent with the code; Help > Keyboard Shortcuts
  complete for the mode; docs/accessibility.md's status line set to what is implemented, with
  the measurements of section 1.1 taken again (`tools/uia_tree.ps1`) and recorded.
- Acceptance criteria:
  1. Every key of the mode is in the README, the ui_spec and the key help, with the same
     meaning (a test compares the key help's table with the key map in code, if the key help
     has one).
  2. The new measurement is in docs/accessibility.md next to the old one.

### A11Y-13. NVDA pass by the user

- Assignee: @dengopaiv. Type: task. Labels: ui, tests. Priority: high. Dependencies: A11Y-12.
- References: the checklist below, README's screen reader section.
- What: the user goes through the checklist with NVDA on Windows and the release build, and
  files each problem as a task (labels ui, milestone m-7). The milestone is done when this task
  is done with no task left open.
- Acceptance criteria:
  1. Each item of the checklist is marked as working or has a task filed for it.

### A11Y-14. Physical units for every knob and fader, derived from the engine

- Assignee: @agent-complex. Type: feature. Labels: ui, engine, docs. Priority: medium.
  Dependencies: A11Y-2, A11Y-D5.
- References: `crates/orb-engine/src` (`tb303.rs`, `tr808.rs`, `tr909.rs`, `drums.rs`,
  `pcf.rs`, `delay.rs`, `dist.rs`, `comp.rs`, `process.rs`, `seq.rs`, `tables/mod.rs`,
  `tables/data.rs`), `crates/orb-engine/src/song.rs` (the controller indices),
  docs/initial_investigation.md and docs/export_state.md (the original's formulas with their
  addresses), `crates/orb-ui/src/behavior.rs` (`knob_send`, `fader_send`), the user's decision
  D5.
- Why: the user decided (D5) that values are spoken in physical units, "as precise as possible
  as we infer them from the code". The engine reproduces the original bit for bit, so the code
  that turns each 0..127 value into a DSP parameter is the best source there is; the original
  panel shows no units.
- What: for each knob and fader (77 knobs, 9 faders; `layout.tsv`), find in the engine how its
  value becomes a parameter, and derive what a musician would want to hear:
  - frequencies in Hz or kHz (the 303's cutoff, from its filter coefficient table
    `FILT[reso >> 4][cutoff]` by the pole frequency of the 2-pole sections; the PCF's frequency;
    tunings as semitones and cents from the pitch tables);
  - times in ms or s (the decays: the time to fall 60 dB from the per-sample or per-pair gain
    multipliers, `TR808_DECAY`, `PCF_DECAY`, the 909's decay table; the 909 attack; the delay
    time from the steps and the tempo);
  - levels in dB (the faders' and level knobs' gains, the compressor's threshold, the accent
    levels), and ratios as "4 to 1";
  - pan as left/right percent ("left 30", "center");
  - shuffle as the delay of the off-beat sixteenths in percent of a sixteenth (or ms at the
    tempo);
  - quantities with no physical unit (resonance, distortion shape and amount, envelope
    modulation depth if it has no clean frequency range) as percent of their range.
  Each mapping is a function in `crates/orb-ui/src/a11y/units.rs` (computed in f64 from the same
  tables the engine uses, imported from orb-engine, not copied), and is marked in a new
  knowledge document `docs/control_units.md` with: the control, the formula, the code it comes
  from (engine function and the original's address as the engine's comments give it), and how
  exact it is: [C] closed form from the engine's own formula, [D] derived numerically from its
  tables or by measuring a render (state the measurement), [G] estimated (say from what). The
  spoken text rounds to a sensible precision (3 significant digits for Hz and ms, 0.1 dB) and
  says the unit in words NVDA reads well ("1.24 kilohertz", "350 milliseconds", "minus 6.5
  decibels"). A setting "Speak raw values" (off by default, `ui.toml`, Options menu in screen
  reader mode) adds the 0..127 number ("1.24 kilohertz, 64"). The engine is not changed.
- Acceptance criteria:
  1. `docs/control_units.md` lists every knob and fader with formula, source and its [C], [D]
     or [G] mark; a test checks that the list and `layout.tsv` agree.
  2. Tests: for each [C] and [D] mapping, sample values (0, 64, 127 and two others) give the
     documented numbers; for at least the 303 cutoff and one decay, the number is checked
     against a measurement of an engine render (a sine or impulse through the filter, a
     decaying voice) within the stated tolerance.
  3. The spoken value of every knob and fader uses the unit; the "Speak raw values" setting
     adds the number; with screen reader mode off nothing changes.
  4. The rendered audio is unchanged: `cargo test --release -p orb-engine --test corpus_golden`
     passes (no engine file is modified).

## The NVDA checklist (A11Y-13)

With NVDA (note its version) and the release build, screen reader mode on (by itself, A11Y-1):

1. Start the program: the window title is spoken, the focus is on the first control of the
   Transport section.
2. F6 through all sections and back with Shift+F6: each section's name is spoken.
3. Tab through the Transport section: each control's name, role and value are spoken.
4. Change the tempo with Up and Down: the new tempo is spoken; Page Up and Page Down.
5. F5 plays and stops; nothing is spoken by itself while it plays. Space on a focused toggle
   toggles it and does not play. F9 Record, F11 Loop and F12 mode are spoken when pressed.
6. F3 reads the transport while playing and while stopped.
7. In Bass Line 1: set the cutoff with the arrows and with NVDA's own slider commands; set the
   waveform; mute channel 1; choose pattern B3.
8. Program a 303 pattern with the keys alone; each step is spoken; F2 reads it back.
9. In the TR-808: walk the grid, set a bass drum on steps 1, 5, 9, 13, F2 reads it back; same
   on the 909 with an accent and a flam.
10. Record in song mode: a pattern change and a knob move are recorded; play them back.
11. File > Open in the system dialog; open a song from rb-songs; its name and mod are spoken.
12. Choose a mod with the mod chooser, with the filter; Mods > Info reads the mod's text;
    Mods > Demo Song.
13. File > Info: read and edit the song's title.
14. Save, Save As, and Export Loop as Audio File: the status messages are spoken.
15. Trigger an error (open a file that is not a song): the error box is spoken and OK closes it.
16. Close with unsaved changes: the prompt is spoken, the focus is on Save.
17. Help > Keyboard Shortcuts: the list can be read line by line.
18. Turn the mode off: the original keys behave as before (Tab taps the 303 while playing).
19. With NVDA closed (no screen reader running), Options > Screen Reader Mode set to Automatic
    leaves the mode off (the other half of A11Y-1's check; A11Y-1 verified only that Automatic
    turns the mode on with NVDA running).
20. In screen reader mode, Save As, Export Loop as Audio File, Mods > Choose file and
    Options > Set Songs Folder / Set Mods Folder open the system dialog, NVDA reads it, and a
    file saved or exported through it is written as through the browser (TASK-194's deferred
    hand check; item 11 covers Open). Type a name with a dot and no extension (`v1.2 mix`) over
    an existing `v1.2 mix.rbs`: the question "Replace v1.2 mix.rbs?" is spoken, No is the
    default.

## Risks and how the plan meets them

- **egui changes.** AccessKit's integration in egui moves between versions (0.37 already
  deprecated `WidgetType` and `Id::new`). The accessible layer stays in one module and talks to
  egui through a few calls; an egui upgrade (upstream's) is checked with A11Y-0's tests.
- **What NVDA does with a custom tree.** Live regions, value changes of a focused slider and
  grid navigation are spoken differently by NVDA, JAWS and Narrator. Each task that depends on
  NVDA's behavior has a by-hand check with NVDA in its criteria, and the checklist covers the
  whole.
- **Frame cost.** The tree is rebuilt each frame; the panel repaints at 30 frames per second
  while playing (task-153). Measured with the empty tree: no measurable change. Each task checks
  `frame_cost.rs`; if the cost grows, the nodes that do not change are cached.
- **Upstream.** With the mode off nothing changes, so rebasing on upstream stays easy and the
  work can be offered upstream as it is.
- **Units that are only estimates.** Some controls may have no clean physical quantity, or
  one that can only be estimated (D5 accepted that). A11Y-14 marks each unit [C], [D] or [G]
  in `docs/control_units.md`, checks the [C] and [D] ones against renders, and the "Speak raw
  values" setting gives the exact stored number next to it.
- **Space no longer plays in screen reader mode.** A habit from ReBirth (D2). F5 plays, Enter
  still plays when no 303 is being programmed, and Help > Keyboard Shortcuts and the
  announcement when the mode turns on say so.
- **Speech over music.** The rule "nothing by itself during playback" is a test (A11Y-5), not a
  hope.
