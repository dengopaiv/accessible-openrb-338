# The Edit menu's Initialize commands (task-306)

Test inputs of Edit > Initialize Loop from Pattern Mode and Initialize Song from Pattern Mode
(`docs/song_load.md` section 14, `docs/ui_spec.md` section 12).

- `init_loop.rbs`, `init_zero.rbs`: written by `build.py` (`tools/rbs.py build` over the Silent
  Default Song template; `/usr/bin/python3 tests/songs/edit_init/build.py` writes both again).
  One song in song mode with the loop on, at 130 BPM, whose device chunks (the pattern-mode
  values: the 303-1 on B2 with cutoff 33 and the square wave, the 303-2 off on A8, the 808 on B1,
  the 909 on B4, other mixer levels, pans and sends, delay and distortion) differ from its tracks, which carry automation before, inside, at the edges of and
  after the loop. `init_loop.rbs` loops bars 3..5 (ticks 1536..3840), `init_zero.rbs` bars 1..2
  (ticks 0..1536).
- `<song>.after_init_loop.rbs`, `<song>.after_init_song.rbs`: what ReBirth 2.01 saved on the
  test machine on 2026-11-25 after opening `<song>.rbs`, choosing the command from the Edit menu
  (for Initialize Song, then Initialize in its alert) and File > Save As. Not edited.

`crates/orb-engine/tests/edit_init.rs` checks orb's tracks after each command against the saved
files' TRAK chunks; `crates/orb-ui/tests/edit_init.rs` runs the commands from orb's Edit menu
and compares orb's whole save with these files.

## The playback check (task-306, open in task-308)

- `export.sh SONG loop|song OUT LOCAL_DIR`: one fresh ReBirth session on the test machine
  (rcs_client.py, all the permits of `redc-win-run` taken first; the script's header) that opens
  SONG, runs the command, exports the loop as OUT.wav and fetches it; it prints the open and
  export times. `fit.sh SAVED.rbs WAV [FROM TO STEP]`: the first idle count N (default 0..30000
  in steps of 16) at which `orb compare --idle-blocks N` finds orb's render of the original's
  save equal to the export (needs `cargo build --release -p orb-cli`).
- The WAVs are not committed (made again by `export.sh`). Those of task-306, 2026-11-25, by
  SHA-256 (all 16-bit stereo, 977028 bytes for `init_loop`, 651364 for `init_zero`):

| Export | Session | SHA-256 | N found |
|---|---|---|---|
| Initialize Loop on `init_loop.rbs` | first (several songs) | `4c900d18faec2fbfe69f750849a57865d48cc13a38e08a565736dd1e787399d5` | 32752 |
| Initialize Song on `init_loop.rbs` | first | `49726aa726ff381e47af8911c90d465f784a731e049147f798854fcd8efa222d` | none in 0..60000 |
| Initialize Loop on `init_zero.rbs` | first | `fe18178a181ede61cc700503a82fde42a4a1a18fc79d3eef74e84b07ece3a46d` | none in 0..60000 |
| Initialize Song on `init_zero.rbs` | first | `efa0b9d29854a063eaa32c2bb321cf57ccd9b2cc5853466263fb38f961aa1dbf` | none in 0..60000 |
| Initialize Song on `init_loop.rbs` | fresh | `74b16a9248c8a54bc648d105d1bd7696d005b2fbd77554b17f940471be81a2a9` | none in 0..30000 |
| Initialize Loop on `init_zero.rbs` | fresh | `45a49359ad8fef1432fdbb982267e71d665b4eeee1d13a487b16f4f8a435a726` | none in 0..30000 |
| Initialize Song on `init_zero.rbs` | fresh | `125306fd2dc73bc1d7097b1076c9a8cbe3f5086a8052beb2bf4ccb88070038bc` | none in 0..30000 |
| Initialize Loop on `init_loop.rbs` (`export.sh` as committed) | fresh | `2a0622be86b712c2191ae94e65e1ad9fe4df235b6e8dfc554aa278e888572997` | 6016 (open to export 9 s) |

All searches were in steps of 16 against the matching `*.after_*.rbs` of this folder.
