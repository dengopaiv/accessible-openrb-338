# The physical units of the knobs and faders

Task-192 (A11Y-14 of docs/accessibility_plan.md), decision A11Y-D5 (task-181): in screen reader
mode every knob and fader speaks its value in a physical unit, "as precise as possible as we
infer them from the code". The original's panel shows no units; the engine reproduces the
original bit for bit, so the code that turns each 0..127 value into a DSP parameter is the best
source there is. The code is `crates/orb-ui/src/a11y/units.rs`: every mapping is computed in
f64 from the engine's own tables and public functions (`orb_engine::tables`, `seq::shuffled`,
`pcf::smooth_cutoff`, `comp::params`, `Tb303::note_on` and its audio slot), and the few scalar
constants the engine keeps private are restated there with their addresses and checked against
the engine's behavior by its tests. The engine is not changed.

How exact each mapping is:

- **[C]** closed form from the engine's own formula (the number is what the engine computes,
  or an exact function of it);
- **[D]** derived numerically from the engine's tables, or by running the engine (the method is
  stated);
- **[G]** estimated (the basis is stated). No mapping needed it.

The tests (`crates/orb-ui/src/a11y/units/tests.rs`) check that the first table lists exactly
the 77 knobs and 9 faders of `assets/ui/layout.tsv` with the marks the code gives, and that
the code speaks the texts of the second table. `crates/orb-ui/tests/units_measure.rs` checks
the 303 cutoff and the 808 bass drum decay against renders of the engine (section "Checked
against renders").

## Definitions

- **Frequency of a 2-pole filter (303, PCF).** The tables (`tables::tb303_filter`,
  `tables::pcf_lowpass`; docs/initial_investigation.md 6.2 and 8.2) store
  `c0 = 2*exp(-pi*w/d)*cos(2*pi*w)` and `c1 = exp(-2*pi*w/d)` for the section
  `y = c0*y1 - c1*y2 + g*x`, so its poles are `r*e^(+-i*theta)` with `r = sqrt(c1)` and
  `theta = 2*pi*w = acos(c0/(2*sqrt(c1)))`. The spoken frequency is the pole frequency
  `theta/(2*pi)*44100` of the table entry the engine looks up ([D]: from the stored f32
  coefficients). At the lowest frequencies `cos(theta)` is so close to 1 that the f32
  rounding of `c0` and `c1` shows: against the closed form `w*44100` of the PCF table the
  pole frequency is off by at most 1.9 % (index 3, about 9 Hz) and less above (the test
  `the_pole_frequency_holds_at_the_low_end`); the 303's table, whose lowest spoken index is
  470, stays far from that. With high resonance it is where the filter rings and peaks; with low
  resonance (`d` near 0.5, a damped pair) it is the corner of the slope.
- **Frequency of a one-pole filter (808 tone, 909 attack).** `y = c*y + g*x`: the -3 dB point
  of `|H(w)|/|H(0)|`, `cos(w) = (1 + c^2 - 2*(1 - c)^2)/(2c)`. "open" when it has none below
  22.05 kHz (`c` below 0.1716: the tables' last entries 0.001).
- **Decay times.** The time for a gain to fall 60 dB (to 1/1000) under the engine's
  multiplier: `ln(0.001)/ln(m)` steps of the multiplier's period (a pair of samples for the
  drum voices, 4 samples for the PCF envelope and the Vintage 303, one 64-sample block for the
  2.0 303), plus any hold. "no decay" when the multiplier is 1 (the voice ends with its
  sample).
- **Levels.** In dB relative to the knob's top (0 dB at 127), or to unity where the engine's
  gain is absolute (the mixer's `T[]` curve, the master's `2*T[]`, the accents, the
  feedback). Every drum level's gain is `L^2` times constants (docs/initial_investigation.md
  7.3), so its dB is `40*log10(L/top)` with the effective level `L` of the knob (the 808's open
  hi-hat `lvl >> 1`, closed hi-hat `(2*lvl)/3`). "silent" at 0.
- **Context.** A few units depend on another control: the 303 cutoff and accent on the synth's
  resonance and the sound model (2.0 or Vintage), the PCF frequency on its Q fader, the
  compressor's ratio on its threshold, the flam on the tempo. The panel gives them
  (`names::units_context`); the second table uses the engine's constructor values: resonance
  90, the 2.0 sound, PCF Q 64, threshold 64, 140 BPM.
- **Words.** Three significant digits for Hz, ms and s, 0.1 dB, whole percent and cents,
  spelled for speech: "1.24 kilohertz", "350 milliseconds", "minus 6.5 decibels", "plus 3
  semitones 25 cents", "4.2 to 1".

## The mappings

| Controls | Spoken as | Formula | Source (engine; the original's address) | Mark |
|---|---|---|---|---|
| `transport.shuffle` | percent of a sixteenth the off-beat sixteenths come late | the first tick `t` of 48..95 with `shuffled(t, v) >= 48`, less 48, of 48 ticks: `trunc(v/6)/48` (0 to 43.75 %) | `seq::shuffled`; 0x1001F3D0 | [C] |
| `tb303_1.tune`, `tb303_2.tune` | semitones | `TB303_TUNE[v >> 2]` (-12..+12) | `tables::data::TB303_TUNE`, `Tb303::note_on`; `word_10033F90`, note-on 0x10023CD4 | [C] |
| `tb303_1.cutoff`, `tb303_2.cutoff` | Hz | the pole frequency of `FILT` at `c = v*0.0022834645669291337 + 0.46` (the smoother's steady state) with the envelope at its neutral point 0.4 (where the envelope mod adds nothing) and no accent: 2.0 sound `idx = trunc(f32(c)*1023)` between the rows `trunc(reso*0.0078732288*7)` and the next, interpolated as `sub_10023800`; Vintage `idx = trunc(c*1023)` in the row `reso >> 4` | `Tb303::smooth_cutoff`, `interpolate`, `process_vintage`; `dbl_10153B80`, `dbl_10153B88`, `flt_10153C3C`, `sub_10023800`, 0x10023F10, table 0x1012B364 | [D] |
| `tb303_1.reso`, `tb303_2.reso`, `pcf.q`, `pcf.amt`, `dist.amount` | percent of the range | `v/127` | no unit: the 303's resonance and the PCF's Q select and interpolate table rows of `d` (the pole pair's damping, which also depends on the cutoff index); the PCF amount scales the step velocity (`E = (vel*amount) >> 5` index steps of 1/48 octave), so its sweep depends on the pattern; the distortion amount sets the input gain `(T[amount] + 0.0625)*2048` and, with the shape, the output gain of a 5x5 grid | [C] |
| `tb303_1.envmod`, `tb303_2.envmod` | octaves the envelope sweeps the cutoff | `em*log2(1602)` with `em = v*0.0029921259842519685 + 0.08`: the envelope moves `c` from `-0.4*em` (decayed) to `+0.6*em` (a note's start), and `FILT`'s frequency is `1602^c` times a constant (before the cutoff clamp) | `Tb303::process_20`, `tables::tb303_filter`; `dbl_10153B90`, `dbl_10153B98`, 0x100234BC | [C] |
| `tb303_1.decay`, `tb303_2.decay` | time for the filter envelope to fall 60 dB after an unaccented note | the engine's note-on gives `env_decay` per 64-sample block (2.0) or per 4 samples (Vintage); accented notes always decay with 0.99969 per sample (505 ms) | `Tb303::note_on`; 0x10023D72 | [C] |
| `tb303_1.accent`, `tb303_2.accent` | dB of the peak gain of an accented note | `20*log10(1 + max acc)`, the accent envelope's peak measured by running the engine's voice (note-on with the accent, then the audio slot block by block) at the synth's resonance and sound model | `Tb303::note_on`, `process_20`, `process_vintage`; 0x10023CF2, 0x100249BB | [D] |
| `tr808.ac_level` | dB an accented step is louder | `20*log10(1 + v*0.03125)`: the AC column holds 1 | `Tr808::step`; `flt_10153F48`, 0x10027821 | [C] |
| `tr909.ac_level` | dB an accented step is louder | `20*log10(1 + 2*v*0.03125)`: the AC column holds 2 (docs/song_load.md 4.8) | `Tr909::step`; `flt_10154368`, 0x1002ACAF | [C] |
| `tr808.bd_level`, `tr808.sd_level`, `tr808.lt_level`, `tr808.mt_level`, `tr808.ht_level`, `tr808.rs_level`, `tr808.cp_level`, `tr808.cb_level`, `tr808.cy_level`, `tr909.bd_level`, `tr909.sd_level`, `tr909.lt_level`, `tr909.mt_level`, `tr909.ht_level`, `tr909.rs_level`, `tr909.cp_level`, `tr909.hh_level`, `tr909.cc_level`, `tr909.rc_level` | dB relative to the knob's top | `40*log10(v/127)`: every gain is `L^2` times constants | `Tr808::level`, `snare`, `tom`; `Tr909::simple`, `pitched`, `bass`, `snare`; `sub_1002A0B4`, `sub_1002A1E4`, `sub_1002A004`, `sub_1002D8B4`, `sub_1002DA24`, `sub_1002DB24`, `sub_1002DC54` | [C] |
| `tr808.oh_level` | dB relative to the knob's top | `40*log10((v >> 1)/63)` | `Tr808::trigger`; 0x10027DDB | [C] |
| `tr808.ch_level` | dB relative to the knob's top | `40*log10(((2*v)/3)/84)` | `Tr808::trigger`; 0x10027E90 | [C] |
| `tr808.bd_tone`, `tr808.cy_tone` | Hz, the low-pass's -3 dB point, or "open" | `TONE8[v >> 4]` one-pole (the cymbal's gain is not normalized, so its tone also sets its level) | `tables::data::TR808_TONE`, `Tr808::bass_or_cymbal`; 0x100340E4, 0x1002A171 | [C] |
| `tr808.bd_decay`, `tr808.cy_decay` | time to fall 60 dB, or "no decay" | `DEC16[v >> 3]` per pair | `tables::data::TR808_DECAY`; 0x10034104, 0x1002A18A | [C] |
| `tr808.oh_decay` | time to fall 60 dB | `4*v` blocks of 64 samples held, then 0.998 per pair (156 ms); a closed hi-hat cuts the hold | `Tr808::trigger`; 0x10027DDB, 0x10027E4F | [C] |
| `tr808.sd_tone` | which of the 5 snare samples | `(v*5) >> 7` | `tables::data::TR808_SD_SAMPLE`; 0x1002A268 | [C] |
| `tr808.sd_snappy` | dB of the noise sample's gain | `40*log10(v/127)`: `g2 = (v/127)^2` | `Tr808::snare`; `sub_1002A1E4` | [C] |
| `tr808.lt_tuning`, `tr808.mt_tuning`, `tr808.ht_tuning` | semitones and cents | `1200*log2(T808[v]/256)`, `T808[i] = trunc(256*2^((i-64)/192))`: -4..+4 semitones | `tables::tr808_tom_step`; 0x1014CBD8 | [C] |
| `tr909.bd_tune` | which of the 4 bass drum samples | `v >> 5` | `Tr909::bass`; 0x1002DB3E | [C] |
| `tr909.bd_attack` | Hz, the low-pass's -3 dB point, or "open" | `C8[v >> 4]` one-pole | `tables::data::TR909_BD_ATTACK`; 0x10034264, 0x1002DBC8 | [C] |
| `tr909.bd_decay`, `tr909.lt_decay`, `tr909.mt_decay`, `tr909.ht_decay`, `tr909.oh_decay` | time to fall 60 dB, or "no decay" | `DT[v >> 3]` per pair, `DT[j] = 1 - (15 - j)*2.5e-5` | `tables::tr909_decay`; 0x1014E64C | [C] |
| `tr909.ch_decay` | time to fall 60 dB | `f32(DT[v >> 3]^8)` per pair (an open hi-hat it cuts gets 0.996) | `Tr909::simple`; 0x1002D935 | [C] |
| `tr909.sd_tune` | which of the 5 snare samples | `(v*5) >> 7` | `tables::data::TR909_SD_SAMPLE`; 0x1002DCCD | [C] |
| `tr909.sd_tone` | time for the noise to fall 60 dB | `v + 30` blocks of 64 samples held, then 0.998 per pair | `Tr909::snare`; 0x1002DC6B, 0x1002DD1F | [C] |
| `tr909.sd_snappy` | dB of the noise sample's gain relative to the top | `20*log10(v/127)`: `g2 = v*0.01` | `Tr909::snare`; `flt_10154648` | [C] |
| `tr909.lt_tune`, `tr909.mt_tune`, `tr909.ht_tune`, `tr909.cc_tune`, `tr909.rc_tune` | semitones and cents | `1200*log2(T909[v]/256)`, `T909[i] = trunc(256*2^((i-64)/144))`: about -5.3..+5.2 semitones | `tables::tr909_tom_step`; 0x1014E54C | [C] |
| `tr909.flam` | time from a step to its flam | `(trunc(v*tempo/700000) + 2)` ticks of `60/(BPM*192)` s: the retrigger comes at the first block whose tick passes `tick + v*tempo/700000 + 1`; to within one 64-sample block (1.45 ms) | `Tr909::step`; 0x1002ABBC | [C] |
| `mixer1.pan`, `mixer2.pan`, `mixer3.pan`, `mixer4.pan`, `delay.pan` | "center", or the percent of the knob's travel to the left (64 values) or right (63) | `d = v - 64`: "left `-d/64`", "right `d/63`". The law (`PAN[a][b]`): the far side drops `(d*d) >> 5` steps of `T[]` | `tables::pan_table`; 0x10034550 | [C] |
| `mixer1.delay`, `mixer2.delay`, `mixer3.delay`, `mixer4.delay` | dB of the send, or "off" | `20*log10(T[v])`, `T[i] = (23/24)^(127-i)` as built (2.0 sound; the Vintage sound weighs the send by the pan, `(L + R)/2`) | `tables::level_curve`; mixer update 0x10012C14 | [C] |
| `mixer1.level`, `mixer2.level`, `mixer3.level`, `mixer4.level` | dB, or "silent" | `20*log10(T[v])` (at the center; the pan law cuts one side) | `tables::level_curve`; 0x10012C14 | [C] |
| `master.fader` | dB, or "silent" | `20*log10(2*T[v])` | `tables::level_curve`; `flt_10151F40`, 0x10012C14 | [C] |
| `pcf.freq` | Hz | the pole frequency of the PCF table at the cutoff base, the steady state of the cutoff smoother towards `4*v` (run with `pcf::smooth_cutoff`, f32 stores as the block), index `pcf::env_index(base)`, in the row `(q*8) >> 7` of the Q fader; the envelope adds `E` index steps of 1/48 octave on a step | `pcf::smooth_cutoff`, `env_index`, `tables::pcf_lowpass`; 0x100167B2, 0x10016968, table 0x1010C270 | [D] |
| `pcf.dec` | time for the envelope's trigger level to fall 60 dB, or "no decay" | `PCF_DECAY[v >> 3]` per group of 4 samples | `tables::data::PCF_DECAY`; 0x100332D0 | [C] |
| `delay.feedback` | dB per repeat, or "no repeats" | `20*log10(f32(v/127))` (127: 0 dB, endless) | `delay.rs` update; 0x1000F754 | [C] |
| `dist.shape` | percent of the range; "hard clip" at 0 | `v/127`; shape 0 is the clip mode `clamp(g*x)` | `dist.rs`; 0x10010B01 | [C] |
| `comp.ratio` | "N to 1", the slope of the static curve just above the knee | `1 + X/(off*floor)` with `(X, off, floor) = comp::params(v, threshold)`: the gain is `X/env + off`, so above the knee the output level is `X + off*env` | `comp::params`; update 0x1000EB44 | [C] |
| `comp.threshold` | dB of full scale at the knee | `20*log10(floor/2)`, `floor = comp::params(_, v).2`: the detector `|x1 + x3|` is twice the level of a signal well below 11 kHz | `comp::params`; 0x1000EB44, 0x1000ED14 | [C] |

## Sample values

The spoken text at 0, 32, 64, 96 and 127, one row per mapping, with the context of the
constructor values (resonance 90, 2.0 sound, PCF Q 64, threshold 64, 140 BPM). Regenerate with
`cargo test -p orb-ui --lib units::tests::print_sample_table -- --ignored --nocapture`.

| Control | 0 | 32 | 64 | 96 | 127 |
|---|---|---|---|---|---|
| `transport.shuffle` | 0 percent of a sixteenth | 10.4 percent of a sixteenth | 20.8 percent of a sixteenth | 33.3 percent of a sixteenth | 43.8 percent of a sixteenth |
| `tb303_1.tune` | minus 12 semitones | minus 5 semitones | 0 semitones | plus 6 semitones | plus 12 semitones |
| `tb303_1.cutoff` | 289 hertz | 496 hertz | 852 hertz | 1.45 kilohertz | 2.46 kilohertz |
| `tb303_1.reso` | 0 percent | 25 percent | 50 percent | 76 percent | 100 percent |
| `tb303_1.envmod` | 0.852 octaves | 1.87 octaves | 2.89 octaves | 3.91 octaves | 4.9 octaves |
| `tb303_1.decay` | 505 milliseconds | 881 milliseconds | 1.87 seconds | 5.62 seconds | 15.6 seconds |
| `tb303_1.accent` | 0 decibels | plus 4.2 decibels | plus 7 decibels | plus 9.1 decibels | plus 10.7 decibels |
| `tr808.ac_level` | 0 decibels | plus 6 decibels | plus 9.5 decibels | plus 12 decibels | plus 13.9 decibels |
| `tr808.bd_level` | silent | minus 23.9 decibels | minus 11.9 decibels | minus 4.9 decibels | 0 decibels |
| `tr808.bd_tone` | 42.9 hertz | 693 hertz | 2.79 kilohertz | 14.8 kilohertz | open |
| `tr808.bd_decay` | 313 milliseconds | 801 milliseconds | 1.44 seconds | 4.48 seconds | no decay |
| `tr808.sd_tone` | sample 1 of 5 | sample 2 of 5 | sample 3 of 5 | sample 4 of 5 | sample 5 of 5 |
| `tr808.sd_snappy` | silent | minus 23.9 decibels | minus 11.9 decibels | minus 4.9 decibels | 0 decibels |
| `tr808.lt_tuning` | minus 4 semitones 2 cents | minus 2 semitones 1 cent | 0 semitones | plus 1 semitone 98 cents | plus 3 semitones 92 cents |
| `tr808.oh_level` | silent | minus 23.8 decibels | minus 11.8 decibels | minus 4.7 decibels | 0 decibels |
| `tr808.oh_decay` | 156 milliseconds | 342 milliseconds | 528 milliseconds | 714 milliseconds | 894 milliseconds |
| `tr808.ch_level` | silent | minus 24.1 decibels | minus 12 decibels | minus 4.7 decibels | 0 decibels |
| `tr909.ac_level` | 0 decibels | plus 9.5 decibels | plus 14 decibels | plus 16.9 decibels | plus 19 decibels |
| `tr909.bd_tune` | sample 1 of 4 | sample 2 of 4 | sample 3 of 4 | sample 4 of 4 | sample 4 of 4 |
| `tr909.bd_attack` | 586 hertz | 1.57 kilohertz | 3.42 kilohertz | 9.73 kilohertz | open |
| `tr909.bd_decay` | 835 milliseconds | 1.14 seconds | 1.79 seconds | 4.18 seconds | no decay |
| `tr909.sd_tone` | 200 milliseconds | 246 milliseconds | 293 milliseconds | 339 milliseconds | 384 milliseconds |
| `tr909.sd_snappy` | silent | minus 12 decibels | minus 6 decibels | minus 2.4 decibels | 0 decibels |
| `tr909.lt_tune` | minus 5 semitones 34 cents | minus 2 semitones 70 cents | 0 semitones | plus 2 semitones 63 cents | plus 5 semitones 22 cents |
| `tr909.ch_decay` | 104 milliseconds | 142 milliseconds | 224 milliseconds | 522 milliseconds | no decay |
| `tr909.flam` | 4.46 milliseconds | 17.9 milliseconds | 31.3 milliseconds | 46.9 milliseconds | 60.3 milliseconds |
| `mixer1.pan` | left 100 | left 50 | center | right 51 | right 100 |
| `mixer1.delay` | off | minus 35.1 decibels | minus 23.3 decibels | minus 11.5 decibels | 0 decibels |
| `mixer1.level` | silent | minus 35.1 decibels | minus 23.3 decibels | minus 11.5 decibels | 0 decibels |
| `master.fader` | silent | minus 29.1 decibels | minus 17.3 decibels | minus 5.4 decibels | plus 6 decibels |
| `pcf.freq` | 9.04 hertz | 57.1 hertz | 363 hertz | 2.3 kilohertz | 13.8 kilohertz |
| `pcf.dec` | 62.3 milliseconds | 209 milliseconds | 1.25 seconds | 6.26 seconds | no decay |
| `delay.feedback` | no repeats | minus 12 decibels per repeat | minus 6 decibels per repeat | minus 2.4 decibels per repeat | 0 decibels per repeat |
| `dist.shape` | 0 percent, hard clip | 25 percent | 50 percent | 76 percent | 100 percent |
| `comp.ratio` | 1 to 1 | 1.3 to 1 | 1.9 to 1 | 3.7 to 1 | 89 to 1 |
| `comp.threshold` | minus 44.3 decibels | minus 10.1 decibels | minus 4.1 decibels | minus 0.6 decibels | plus 1.8 decibels |

## Checked against renders

`crates/orb-ui/tests/units_measure.rs` (run it in release):

- **The 303 cutoff, tolerance 2%.** A `Tb303` (2.0 sound, saw, tune 0 and octave down: C at
  16.35 Hz, envmod 0, accent 0) plays one note; the envelope is pinned at the neutral point 0.4
  before every block (`env_decay = 1`), so the filter sits at the knob's cutoff. After 2000
  blocks, 65536 samples are taken; under a Hann window the magnitude at each harmonic
  `k*f0` is divided by the saw's `1/k`, by the voice's one-pole pre-filter (its `c3` from the
  engine's coefficients) and by its DC blocker (0.985853374), and a parabola through the
  logarithms of the largest harmonic and its neighbors gives the resonance peak. Result
  (2026-11-06): cutoff 0 / 40 / 64 / 100 / 127 at resonance 127 spoken 326.7 / 643.6 / 963.9 /
  1766.8 / 2783.1 Hz, peak 325.1 / 642.4 / 963.3 / 1766.2 / 2782.5 Hz (0.49%, 0.19%, 0.06%,
  0.03%, 0.02%); cutoff 64 at resonance 112: 918.7 Hz spoken, 917.0 Hz peak (0.18%). The
  peak of a resonant pair lies a little below its pole frequency (`cos(w_peak) =
  (1 + r^2)/(2r) * cos(theta)`), most at the lowest cutoffs, where `d` is smallest.
- **The 808 bass drum decay, tolerance 1.5 blocks (2.2 ms) on the 60 dB time.** A `Tr808`
  with a constant bass drum sample (16000 for 12 s) plays one hit at level 127 with the tone
  open (`TONE8[7]`); the time the output takes to fall 40 dB, times 1.5, is the 60 dB time of
  an exponential fall. Result (2026-11-06): decay 0 / 40 / 64 / 100 spoken 0.3131 / 0.9011 /
  1.4422 / 4.4768 s, measured the same to 0.1 ms. Decay 127 (`DEC16[15] = 1`) is "no decay".

## Facts found on the way

- **A drum voice stops before it has fallen 60 dB.** A SIMPLE, FILTER or DUAL voice ends when
  its gain is below 3e-8 (`flt_1015364C`, `drums::SILENT`; the DUAL voice tests `g1` at
  0x1001E7E2); the gain carries the int16 scale `2^-15` (0x1001E2CD). A full-level 808 bass
  drum starts at `g = ((127*k)*127)*0.005*(1 - c)/32768 = 1.94e-5` (`k = f32(1/127)`
  `flt_10153F4C`, `0.005` the level scale `flt_10153F50`, `LEVEL_SCALE` in `tr808.rs`, the
  open tone `c = 0.001`), so it stops about 56 dB
  down: its 60 dB decay time is the multiplier's, not a time the voice reaches. Lower levels
  stop sooner (a level of 64 starts 11.9 dB lower and stops about 44 dB down). The spoken decay
  times are the multipliers' times; the sample's length can end a voice earlier too.
- **The 303's decay knob shapes the filter envelope only.** The envelope `+0xCA8` moves the
  cutoff (`(e - 0.4)*em`); the amp has its own gate constants (0x10023E3D, 0x10023CB3). An
  accented note ignores the knob: its envelope decays with 0.99969 per sample (0x10023DB0),
  the knob's value 0 (0.99999 - 0.0003): 505 ms to fall 60 dB.
- **The PCF's resting index is `4*freq - 1`.** The cutoff smoother (`pcf::smooth_cutoff`,
  0x100167B2, f32 store `+0x80`) settles one f32 step below `4*freq` for every frequency 1..127,
  so with the envelope at rest (E = 0) the table index `trunc(env)` (0x10016968) is one below
  the `4*freq` docs/initial_investigation.md 8.2 implies (noted there). The spoken frequency
  uses the settled value (the test `some_numbers` checks every value).
- **The resonance raises the 303's and the PCF's pole frequency.** The tables' `w` is
  `pow(k, i)*base*(0.6 + 0.4*row/7)` (0x100234A0, 0x100172C4), so at a fixed cutoff the top
  resonance row is 5/3 of the bottom one's frequency (8.8 semitones); the spoken cutoff
  follows the resonance for that reason.
