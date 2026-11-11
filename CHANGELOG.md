# Changelog of accessible-openrb-338

The changes of each release of this fork, newest first. A release is one portable zip for
Windows x64 on the [releases page](https://github.com/dengopaiv/accessible-openrb-338/releases);
the zip carries this file as CHANGELOG.txt. The sound engine is OpenRB-338's: a song renders
the same as in OpenRB-338 and the original ReBirth in every release below.

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
