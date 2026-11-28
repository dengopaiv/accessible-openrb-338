# Changelog of accessible-openrb-338

The changes of each release of this fork, newest first. A release is one portable zip for
Windows x64 on the [releases page](https://github.com/dengopaiv/accessible-openrb-338/releases);
the zip carries this file as CHANGELOG.txt. The sound engine is OpenRB-338's: a song renders
the same as in OpenRB-338 and the original ReBirth in every release below.

## Next release (not released yet)

This section holds the changes made since the last release.

- **The menu bar**, in screen reader mode: Left and Right go from menu to menu as in Windows. On a menu's name they go to the previous or next name, and from Help round to File; in an open menu they open the previous or next menu with the focus on its first item. The focus no longer leaves the bar for the Transport section after Help. In the Mods menu's list of files that share a name, Right opens it, Left closes it. With screen reader mode off the menus are as before.
- **Options > Audio Output** shows the device that is in use after the program falls back from a lost one: it said the lost device until you picked again. It says nothing new; the program's own note is all that is announced.
- **Classic look**, in screen reader mode: the panel reads and works as the modern one does (names, Tab order, sections, pattern selectors, drum grids, 303 step editor). The 808's round instrument knob of the classic panel is skipped by the screen reader: the drum grid's Up and Down choose the instrument. Switching View > Classic or Modern keeps the focus where it is and says nothing.
- **Screen reader names**: NVDA now says 808, 909, 303-1 and 303-2 for the sections and controls, and the Keyboard Shortcuts help says the same, as the rest of the app does. They were TR-808, TR-909 and Bass Line 1 and 2.
- **Edit > Initialize Song from Pattern Mode**, in screen reader mode: the question takes the
  focus on Initialize and is read with its text; Tab reaches Cancel, and the focus goes back
  to where it was after the answer.
- **Options > Audio Output**, in screen reader mode: the window is made for screen readers.
  The focus goes into it and comes back when it closes. Driver, Device, Sample rate and Buffer
  size are named and say what is chosen; the "In use" numbers are lines you can reach with Tab
  and read. A change of driver, device, rate or buffer is announced once; the numbers that
  refresh twice a second are never announced. Escape closes an open list first, then the
  window. The 2026.1127.2013 release listed this window as not yet made for screen readers.
- **The 303-1 and 303-2 Pattern Editors**, in screen reader mode: the window takes the focus
  when it opens (Edit menu or Ctrl+E) and gives it back when it closes (Escape). Pattern,
  Length, Shuffle, Transpose, Shift, Copy, Paste and Clear are named controls you reach with
  Tab. The notes are a list of the 16 steps with the same keys as the panel's step editor:
  Up and Down move, the 303's keys program the step and it is read back, F2 reads the pattern.
  Nothing is spoken while the song plays.
- **The 808 and 909 Pattern Editors**, in screen reader mode: the window takes the focus when it
  opens (Edit menu or Ctrl+E) and gives it back when it closes (Escape). Pattern, the 909's
  Entry, Length, Shuffle, Shift, Copy, Paste and Clear are named controls you reach with Tab.
  The steps are a grid with the same keys and words as the panel's drum grid: the arrows move,
  Space sets the cell and it is read back, F2 reads the pattern. Nothing is spoken while the
  song plays.

## 2026.1127.2013 (2026-11-27)

This release holds the work of OpenRB-338 taken into the fork on 2026-11-18 and 2026-11-26.

### New from OpenRB-338

- **Options > Audio Output.** Choose the driver, the device, the sample rate and the buffer
  size, and see what is in use. On Windows, ASIO drivers are offered too. This window is not
  yet made for screen readers.
- **Pattern editors.** Edit > 303-1 Pattern Editor, 303-2 Pattern Editor, 808 Pattern Editor
  and 909 Pattern Editor open a window for one pattern, edited with the mouse; Ctrl+E opens the
  focused row's. These windows are not yet made for screen readers; in screen reader mode the
  panel's step editor and drum grids work as before.
- **The classic panel.** View > Classic draws the original ReBirth panel from the mod's
  pictures; View > Modern is the panel as before.
- **The Edit menu**, between File and Mods. It acts on the selected pattern of the focused row:
  - Cut, Copy and Paste Pattern (Ctrl+X, Ctrl+C, Ctrl+V) move a pattern to another slot; a
    303 pattern pastes into either 303. Copy puts the pattern on the Windows clipboard as text.
  - Duplicate Pattern (Ctrl+D) copies it to the row's next empty slot and selects it.
  - Undo (Ctrl+Z) takes back the last of these, one step.
  - Clear Pattern, and Initialize Song or Loop from Pattern Mode as in the original.

  In screen reader mode the menu is read like the others and each command says what it did.
- **Ctrl+I** opens File > Info. In screen reader mode's Choose Mod dialog, Ctrl+I still shows
  Mod Info of the selected mod, as before.
- A new song starts with four sample patterns in A1.
- The meters and the compressor's gain-reduction lights follow the original's scale. In screen
  reader mode, F4 now says the gain reduction as one of nine lights.
- Help > About has a button that copies the version.
- A double click on a knob sets it to its default value.
- The modern panel has a new look, OpenRB-338's redesign.

### Changed

- **Settings and portable mode.** The settings file is now called OpenRB-338.toml (your
  ui.toml is copied into it once and left as it was). The portable zip now has the file
  OpenRB-338.portable instead of portable.txt, keeps the settings in OpenRB-338.toml beside the
  program and the logs in the folder logs. A folder unpacked from an older release, with
  portable.txt and the folder data, stays portable and keeps its settings: the first start
  copies data\ui.toml into OpenRB-338.toml. The songs and mods folders inside the program's
  folder are still remembered relative to it, so a USB stick works under any drive letter.
- **Settings are no longer lost with two windows open.** Saving the settings now changes only
  what this window changed and keeps what another one saved.
- **No plugin.** The plugin that OpenRB-338 offers for music programs is not part of this
  fork, and the zip has no plugin file.

## 2026.1111.1330 (2026-11-11)

### New

- **The numpad keys of the original (Windows).** The numpad now works apart from the main
  keyboard, as in ReBirth, in both keyboard modes and whatever panel control has the focus
  (in screen reader mode numpad star, like Insert, does nothing while a window such as the
  key list is open):
  - Numpad plus and minus: tempo up and down (held, they repeat).
  - Numpad Enter: Play. Numpad 0: Stop.
  - Numpad 4 and 5: Rewind and Fast forward. Numpad 7 and 8: bar down and bar up.
  - Numpad 1 and 2: to the loop's start and end.
  - Numpad slash: Loop on or off. Numpad star: Record, as Insert.
  - Numpad 3, 6, 9 and the numpad's point do nothing, as in the original.

  The main keyboard's keys do what they did before. Keep Num Lock on: with it off, Windows
  turns the numpad into arrows, Insert, End and Delete, and NVDA's desktop layout takes the
  numpad for its own commands (numpad plus is its Say all with review). Help > Keyboard
  Shortcuts (F1) lists these keys in the group "The numpad".
- **The beat cookbook.** [docs/beats.md](docs/beats.md) shows how to write a song as a short
  Python script with `tools/rbs.py` and hear it with `orb`, from a single drum beat to all four
  machines in song mode, with the Windows form of every command. [examples/](examples/) holds
  a finished song, `empuje.rbs`, and the script that builds it. Written by gaolivaresm for
  OpenRB-338 (its pull request 6). It needs the repository and Python 3; it is not in the zip.

### Changed

- **Save writes the song the way the original does** (from OpenRB-338, its pull request 4 by
  ghstcoder47). File > Save now writes the patterns and song tracks as the sound engine holds
  them, for every song, new or opened, ReBirth 2 or 1. So saving a song you did not change no
  longer gives the same file byte for byte: song events past bar 999, which the original also
  drops when it loads a song, are not written back, and the tracks are in the engine's order.
  The song plays as before. File > Info's "Last event" follows the same tracks.

### Left out

- The same pull request 6 also adds `orb-mcp`, a server through which an AI client writes
  songs. This fork does not take it: the fork exists so that a person can make music with
  ReBirth's machines, and it keeps to tools for that person.

### Known limits

- The numpad keys pass their automated tests but have not yet been tried by hand with NVDA on
  a physical numpad.

## 2026.1109.1831 (2026-11-09)

The first release of the fork: OpenRB-338 2026.1103.0917 with screen reader mode added.

### New

- **Screen reader mode.** With NVDA running, the app turns the mode on by itself (Options >
  Screen Reader Mode: Automatic, On or Off). Every control has a name, a role and a spoken
  value; knobs and faders speak their physical units (hertz, decibels, milliseconds, percent,
  semitones), and Up and Down on a knob jump to the next value that is spoken differently (by
  1 with Options > Speak Raw Values on).
- **Moving around from the keyboard.** F1 lists the keys. F5 plays and stops. F6 and Shift+F6
  move between the sections, Tab and Shift+Tab between the controls. F10 or Alt opens the menu
  bar, and leaving it puts the focus back where it was.
- **The 303 step editor and the drum grids** can be programmed and read back from the keyboard.
  Shift+Space sets an accent on a drum grid. Options > Pitch Keys Make Notes lets the 303's
  pitch keys write notes.
- **Announcements and read-out keys.** Status messages are spoken, such as the opened song and
  its mod, the chosen mod, the keyboard mode and the transport keys; nothing is spoken by itself
  during playback. F2 reads the pattern of the focused section, F3 the transport, F4 the levels.
- **Dialogs** have roles, names and focus. The file dialogs are Windows' own in screen reader
  mode. The mod chooser has a filter, and the Mods menu reads its entries as checked items.
- **A portable zip.** While `portable.txt` lies next to OpenRB-338.exe, the settings and logs
  stay in the folder `data` beside it, and the songs and mods folders inside the program's
  folder are remembered relative to it, so a copy on a USB stick keeps working under another
  drive letter.

### Changed, in both modes

- The panel's keys wait while a menu is in use.
- On Windows the program prefers DirectX 12 for its graphics, so it starts on more PCs, and it
  writes a run log of each start (in `data\logs` beside the program), so a start that fails
  leaves something to read; docs/building.md describes it.
- The Export dialog keeps the format you pick.
