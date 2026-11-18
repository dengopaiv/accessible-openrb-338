//! AC #3 of task-043: the player, given a corpus song and then the export's start after the same
//! idle count, outputs the export's samples.
//!
//! The export (`orb_engine::frame::export::export_song`, rbhost's run) loads the song into a new
//! engine, renders `idle_blocks` idle blocks, then stops, locates to the loop start and plays, and
//! renders the loop. The player gets the same song with [`Player::load`] before its first
//! callback; its callback (driven here directly, without a device) renders `idle_blocks / 16`
//! engine calls of 1024 frames (every corpus N is a multiple of 16), then the UI sends Stop,
//! Locate(loop start) and Play, which the next engine call applies before it renders, as the
//! export applies them between its 0-frame calls. From there the callback's output must equal
//! the export's frames sample for sample, whatever the callback sizes (they vary below), and so
//! the corpus reference (the WAV's sha256 in the manifest).
//!
//! The two songs whose export faults (the PCF's division by zero from block 6, the
//! `pcf_*_wave127` songs) are compared too: the export's chunk buffer never gets written, so it
//! stays zero and the reference is the all-zero WAV, and the player outputs silence for a
//! faulting call; the player's snapshot must report the faulted calls
//! ([`orb_player::Snapshot::faulted_calls`]).
//!
//! Every `diag`, `app` and `shipped` entry is checked (the 8 shipped songs of the built-in kit;
//! the corpus' modded songs need the user's mods), in the optimized test profile in 3 s.

mod common;

use std::sync::Arc;

use common::{manifest, run, samples, send, song, tables};
use orb_engine::frame::export::{self, ExportRange};
use orb_engine::frame::{Devices, Engine};
use orb_player::{Command, Player};
use sha2::{Digest, Sha256};

/// The callback sizes, in frames, cycled after the play command.
const SIZES: [usize; 8] = [1, 441, 1000, 64, 4096, 7, 2048, 513];

struct Outcome {
    name: String,
    faulted: bool,
    player_faulted_calls: u64,
    equal: bool,
    sha_ok: bool,
}

fn check(name: &str, entry: &serde_json::Value) -> Outcome {
    let s = song(entry);
    let idle = entry["idle_blocks"].as_u64().unwrap() as u32;
    assert_eq!(
        idle % 16,
        0,
        "{name}: idle_blocks {idle} is not a whole number of calls"
    );
    // the offline export
    let mut eng = Engine::new(tables(), Devices::implemented(Some(samples())));
    let want = export::export_song(&mut eng, &s, idle).unwrap();
    let faulted = eng.fault().is_some();
    // the player
    let (mut p, mut a) = Player::new(tables(), Some(samples()));
    assert!(p.load(&s, Some(samples())).is_ok());
    run(&mut a, (idle / 16) as usize);
    send(&mut p, Command::Stop);
    send(
        &mut p,
        Command::Locate(ExportRange::from(&s.globals).loop_start),
    );
    send(&mut p, Command::Play);
    let mut got = vec![0i16; want.len()];
    let mut at = 0;
    for &n in SIZES.iter().cycle() {
        if at == got.len() {
            break;
        }
        let end = (at + 2 * n).min(got.len());
        a.render_i16(&mut got[at..end]);
        at = end;
    }
    let sha: String = Sha256::digest(export::wav_bytes(&got))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    p.collect_garbage();
    Outcome {
        name: name.to_string(),
        faulted,
        player_faulted_calls: p.snapshot().faulted_calls,
        equal: got == want,
        sha_ok: sha == entry["wav_sha256"].as_str().unwrap(),
    }
}

#[test]
fn the_player_outputs_the_export_after_the_same_idle_count() {
    let m = manifest();
    let mut entries: Vec<(String, serde_json::Value)> = Vec::new();
    // the shipped songs: 41 minutes of audio, sha256 references only
    let sets = ["diag", "app", "shipped"];
    for set in sets {
        for (k, v) in m[set].as_object().unwrap() {
            entries.push((format!("{set}/{k}"), v.clone()));
        }
    }
    let entries = Arc::new(entries);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let outcomes: Vec<Outcome> = std::thread::scope(|sc| {
        let hs: Vec<_> = (0..threads)
            .map(|t| {
                let entries = entries.clone();
                sc.spawn(move || {
                    entries
                        .iter()
                        .skip(t)
                        .step_by(threads)
                        .map(|(n, e)| check(n, e))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let mut checked = 0;
    let mut faulted = Vec::new();
    for o in &outcomes {
        if o.faulted {
            assert!(o.name.contains("wave127"), "{}: the export faults", o.name);
            assert!(
                o.player_faulted_calls > 0,
                "{}: the player reports no fault",
                o.name
            );
            faulted.push(o.name.as_str());
        } else {
            assert_eq!(o.player_faulted_calls, 0, "{}", o.name);
        }
        assert!(
            o.equal,
            "{}: the player's output differs from the export",
            o.name
        );
        assert!(
            o.sha_ok,
            "{}: the player's output differs from the reference",
            o.name
        );
        checked += 1;
    }
    // 145 diag + 11 app + 8 shipped
    assert_eq!(checked, 164);
    assert_eq!(faulted.len(), 2);
    eprintln!(
        "{checked} songs equal to the export and the reference, of them faulting: {faulted:?}"
    );
}
