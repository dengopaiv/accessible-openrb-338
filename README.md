<p align="center">
  <img src="assets/readme/logo.gif" alt="OpenRB-338" width="378">
</p>

<div align="center">

# OpenRB-338

</div>

<p align="center">
  <a href="https://github.com/pingw33n/openrb-338/releases"><b>Download</b></a> ·
  <a href="assets/readme/screenshot.png"><b>Screenshot</b></a> ·
  <a href="https://github.com/pingw33n/rb-songs"><b>Song collection</b></a> ·
  <a href="https://github.com/pingw33n/rb-mods"><b>Mod collection</b></a>
</p>

OpenRB-338 is a reimplementation of [ReBirth RB-338](https://en.wikipedia.org/wiki/ReBirth_RB-338),
the 1997 software synth of two TB-303 bass lines, a TR-808 and a TR-909, for today's macOS, Linux
and Windows.

## Features

- **Sounds exactly like the original.** The audio engine was rebuilt from ReBirth 2.01 to the
  bit. A song's exported loop equals the original's "Export Loop as Audio File" byte for byte. This
  is tested on 1,268 songs of the song collection, 531 of them on 43 third-party mods, and on a
  175-render reference corpus.
- **The whole machine on screen:** the two 303s, the 808 and the 909, the mixer, the delay,
  distortion, PCF and compressor, with pattern and song mode, the loop, shuffle and recording.
- **A scalable vector-graphics panel** that stays sharp at any window size and pixel density.
- **Songs:** opens ReBirth 2.x songs (`.rbs`, RBS 4.0 to 4.2) and ReBirth 1.x songs, and saves
  RBS 4.2 files that the original opens. File > Info shows and edits the song's title, text, web
  page and copyright.
- **Mods:** plays the drum samples of ReBirth mods (`.rbm`), chosen per song, with the mod's
  startup sound. The standard kit is built in, so no file of the original is needed.
- **Audio export** of the loop to WAV, FLAC or Ogg Vorbis.
- **Plays from the computer keyboard:** program the synths and select patterns without a mouse
  (Help > Keyboard Shortcuts).
- **`orb`, a command-line tool,** to work with songs and mods from the terminal:
  - `orb render`: render a song's loop to WAV, FLAC or Ogg, far faster than real time (a
    five-minute song takes about a second and a half);
  - `orb play`: play a song without the UI;
  - `orb info`: print a song's facts (format, title, mod, tempo, loop), also as JSON;
  - `orb mod`: recrypt mods, and unpack a mod to a folder or pack one from it.
- **Packages** for macOS (Apple silicon and Intel), Windows (x64 and ARM64) and Linux (x86_64 and
  ARM64).

## Not implemented

- MIDI, VST and ReWire. OpenRB-338 is a standalone application only, with no MIDI input or sync.
- Mod themes: a mod changes the drum sounds, not the look of the panel.
- The original's Edit menu functions on patterns (copy, paste, clear, shift, randomize and the
  like), and undo.
- The original's preferences and its Windows menu.
- Export to AIFF (use WAV, FLAC or Ogg).

## Accessibility (this fork)

This repository, accessible-openrb-338, is a fork of
[OpenRB-338](https://github.com/pingw33n/openrb-338) whose aim is to make OpenRB-338 usable
with a screen reader, NVDA on Windows first.

**Where it stands (2026-11-07): built and checked with NVDA.** Screen reader mode is
implemented and tested through the accessibility tree (every control has a name, a role and a
value; Tab stops at 198 places, all named), and a blind user went through the whole checklist
with NVDA on Windows and found it working. Other screen readers and systems are less tested.
With the
mode off, OpenRB-338 behaves exactly as upstream, and the sound engine is not touched, so the
output stays identical to the original's. The `orb` command line tool (render, play, info,
mods) needs no mode.

### Using OpenRB-338 with a screen reader

**Turning the mode on.** Options > Screen Reader Mode has three choices: Automatic (the
default: on while Windows reports a running screen reader, such as NVDA), On and Off. On
other systems Automatic stays off; choose On. Options > Speak Raw Values adds the 0 to 127
number after a knob's physical value ("1.24 kilohertz, 64"). Options > Step Editor Follows
Playback and Options > Pitch Keys Make Notes are explained under programming a pattern. F1 opens Help > Keyboard Shortcuts, which
lists the keys of the mode that are in force and reads line by line (Tab walks the lines).

**The sections.** The panel is nine sections: Transport, Bass Line 1, Bass Line 2, TR-808,
TR-909 (each machine with its pattern selector and its mixer channel), Master and
compressor, PCF, Delay and Distortion. F6 and Shift+F6 go to the first control of the next
and the previous section; Tab and Shift+Tab move control by control and wrap around. The menu
bar is reached with F10 (or Alt alone). Entering a machine's section gives that machine the
"row focus", which is where the pitch and drum keys of ReBirth go.

**The keys.** Everything else keeps its ReBirth meaning, so the panel's keyboard still plays
and programs the synths (Ctrl+F) or selects patterns (Ctrl+G).

| Key | What it does |
|---|---|
| F1 | Help > Keyboard Shortcuts |
| F2 | Reads the pattern of the section you are in (the 303's steps, or the drum machine's steps per instrument) |
| F3 | Reads the transport: in song mode playing or stopped, bar, tempo, mode, loop and recording; in pattern mode, where bar, loop and Record do nothing, only playing or stopped, tempo and "pattern mode" |
| F4 | Reads the master levels and the compressor's gain reduction |
| F5 | Play, or Stop if playing |
| F6 / Shift+F6 | Next / previous section |
| F9 (Insert too) | Record |
| F10, or Alt alone | The menu bar |
| F11 | Loop on / off |
| F12 | Song / pattern mode |
| Tab / Shift+Tab | Next / previous control |
| Space | Presses the focused button, toggle or switch |
| Enter | In a 303's section the 303's Step (Program Synth mode); in any other section Play |
| Up / Down | On a knob or fader: to the next value that is spoken differently (by 1 with Options > Speak Raw Values on); on a spin button (tempo, bar, lengths): one click; on a pattern selector: the next / previous pattern |
| Page Up / Page Down | Knob or fader by 8 or more, to a value spoken differently, nothing when none lies that way (by 8 with Speak Raw Values on); spin button by 10 clicks; pattern selector by one bank |
| Home / End | Knob or fader to 0 / 127; pattern selector to A1 / D8; on a short spin button (pattern length, delay steps, PCF pattern) its ends; nothing on tempo, bar and loop |
| Left / Right | On a drum grid: the step before / after, within the pattern's length. Nothing elsewhere, and nothing on a 303's step editor (Up / Down move its step) |
| Ctrl+Up / Ctrl+Down | Moves the row focus to the previous / next row without moving where you are |
| Ctrl+Tab / Ctrl+Shift+Tab | Program Synth mode, a 303 in focus: the playing step becomes a note / a rest |
| Ctrl+M | Choose Mod (below) |
| Escape | Closes a window, a dialog or a menu |

Knobs and faders are spoken in physical units, such as "350 milliseconds" or "minus 6.5
decibels". Messages ("Saved ...", "Exported ...", "Mod ...") are spoken when they happen.
Nothing is spoken by itself while the music plays.

**Programming a 303 pattern.** Press F6 until "Bass Line 1" (or 2) and choose the pattern
on its first control, the pattern selector (Up and Down). Tab to "Step editor"; it says the
step and what it holds, for example "Step 1 of 16: rest, C". On each step, a pitch key sets
the note (`C` is C, `F` C sharp, `V` D, and so on up to `/` for high C) and leaves a rest a
rest, as the original does, unless Options > Pitch Keys Make Notes is on; `-` or `=` turns the
rest into a note and back, `P` is accent, `[` slide, `'` octave up and `;` octave down. Down
(or Enter, in Program Synth mode) goes to the next step, Up or Backspace to the previous one, Home to step 1 and End
to the last step; Left and Right do nothing. Each key is answered with the step's new state,
and F2 reads the whole pattern back. F5 plays it. The step editor stays on its own pattern
while a song plays and says nothing when the song moves on; Options > Step Editor Follows
Playback makes it go along with the song at your next key instead, still silently.

**Programming a drum pattern.** F6 to "TR-808" or "TR-909" and choose the pattern; Tab to
"Steps". It says the cell under the cursor, "Bass drum, step 1, off". Up and Down choose the
instrument (Accent first), Left and Right the step, Home and End the first and the last step.
Space sets the step. On the TR-909 it goes round off, note, accent as a mouse click does;
Ctrl+Space sets a plain note, Shift+Space an accent and Ctrl+Alt+Space a flam (Alt+Space is
an accent too, but Windows then also opens the window's system menu: Escape closes it). The
cell says its new state. F2 reads the pattern back, each instrument with its steps.

**Choosing a mod.** Ctrl+M (or Mods > Choose Mod...) opens a dialog with a filter field and
the list of mods. Type to narrow the list by name; Down moves from the filter to the list;
Tab goes between the two. In the list Up, Down, Home, End, Page Up and Page Down move;
Right opens a mod that has variants and Left closes it. Enter chooses the selected mod (for a
mod with variants the preferred one, or the variant selected), Ctrl+I shows its Mod Info
without loading it, and Escape closes the dialog.

**Files.** File > Open, Save As and Export use the system's own file dialogs in this mode.

**Known limits.**
- It has not been tried with NVDA by the person it is made for yet; expect rough places.
- Detection of a running screen reader in Automatic is Windows only. Narrator, JAWS,
  VoiceOver and Orca are not tested.
- The drum grids are not a real table to the screen reader: each cell carries its
  instrument and step in its own text, and the arrow keys are the program's.
- The meters are read as lit segments (F4), not decibels.
- The original's pattern functions (copy, paste, clear a bar, undo) are not implemented.

- [docs/accessibility.md](docs/accessibility.md): what a screen reader gets today (measured),
  the options, the proposal, and notes for developers.
- [docs/accessibility_plan.md](docs/accessibility_plan.md): the plan, as decisions and tasks,
  with the NVDA checklist that ends it.

## Quick start

1. **Download** the package for your system and CPU from
   [Releases](https://github.com/pingw33n/openrb-338/releases):

   | System | File |
   |---|---|
   | macOS, Apple silicon | `OpenRB-338-<version>-aarch64-apple-darwin.dmg` |
   | macOS, Intel | `OpenRB-338-<version>-x86_64-apple-darwin.dmg` |
   | Windows, x64 | `OpenRB-338-<version>-x64-setup.exe` |
   | Windows, ARM64 | `OpenRB-338-<version>-arm64-setup.exe` |
   | Linux, x86_64 | `OpenRB-338-<version>-x86_64.AppImage` |
   | Linux, ARM64 | `OpenRB-338-<version>-aarch64.AppImage` |

   Plain archives (`.tar.gz`, or `.zip` on Windows) with the two binaries are there too.

2. **Install it.**
   - macOS: open the `.dmg` and drag OpenRB-338 to Applications.
   - Windows: run the installer. It can add `orb` to the PATH.
   - Linux: make the AppImage executable (`chmod +x OpenRB-338-*.AppImage`).

3. **Optionally, get songs and mods:** download the
   [song collection](https://github.com/pingw33n/rb-songs) and the
   [mod collection](https://github.com/pingw33n/rb-mods), and unpack them anywhere.

4. **Run OpenRB-338.** The binaries are not signed, so the first start needs your permission:
   - **macOS:** the first launch is refused. Open System Settings > Privacy & Security, click
     "Open Anyway" next to OpenRB-338, and confirm. On macOS 14 and older, right-click the app >
     Open > Open works too. Or, in Terminal: `xattr -dr com.apple.quarantine
     /Applications/OpenRB-338.app`.
   - **Windows:** SmartScreen says "Windows protected your PC". Click "More info", then
     "Run anyway".
   - **Linux:** nothing more. If the AppImage does not start because FUSE is missing, install
     `libfuse2` or run it with `--appimage-extract-and-run`.

5. **Point it at your songs and mods.**
   - **Songs:** File > Open... opens the file browser. Go to the folder of your songs and click
     "Use as songs folder", so the browser starts there next time. Then open a song (`.rbs`).
   - **Mods:** Mods > Choose file... opens the browser on mods. Go to your mods folder and click
     "Use as mods folder". The Mods menu then lists every mod in it, and a song finds its mod
     there by name. Mods must be recrypted to load (see below); the mods of the mod collection
     already are.

6. **Play:** press Play, or Space.

## Making mods work in OpenRB-338

The mods of the [mod collection](https://github.com/pingw33n/rb-mods) already work: open them
as they are.

Other mods need one step first. A ReBirth mod's samples are encrypted with a key that ReBirth
reads from `RB20FUL.DAT`, a file on the ReBirth CD. Instead of making you keep that file around,
OpenRB-338 carries the small part of the key that mods use, and needs mods re-encrypted with that
part. `orb mod recrypt` does this:

```sh
# one mod
orb mod recrypt --rb20ful RB20FUL.DAT MyMod.rbm -o MyMod-recrypted.rbm

# every mod of a folder, into another folder
orb mod recrypt --rb20ful RB20FUL.DAT ~/Mods -o ~/Mods-recrypted

# the key read straight from the ReBirth CD image
orb mod recrypt --iso "ReBirth.iso" ~/Mods -o ~/Mods-recrypted
```

Recrypting changes nothing else in the mod: its samples, panel pictures and text stay as they
were. The recrypted file is still a complete mod, and the original ReBirth opens it as before.
`RB20FUL.DAT` must be the one of the 2001 reissue CD (its checksum is checked). A mod that is
already recrypted is only checked and copied.

## Building

See [docs/building.md](docs/building.md).

## License

OpenRB-338 is free software under the [GNU General Public License v3.0](LICENSE) (GPL-3.0-only).
Copyright (C) 2026 the OpenRB-338 authors.

ReBirth RB-338 is a trademark of Propellerhead Software (now Reason Studios). OpenRB-338 is not
affiliated with or endorsed by them. The built-in standard kit (`assets/Standard.rbm`) holds
ReBirth's own drum samples and panel pictures. See [assets/README.md](assets/README.md).
