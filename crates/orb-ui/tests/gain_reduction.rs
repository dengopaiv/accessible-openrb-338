//! task-252 (decision task-251): the compressor's gain-reduction LEDs follow the engine's gain
//! byte as the original's bar from the left (docs/ui_spec.md sections 2 and 16): five LEDs for
//! the idle compressor on the master, none with the master's COMP switch off.

use std::sync::Arc;

use orb_engine::tables::Tables;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::link::Link;
use orb_ui::model::{Panel, gr_lit};

/// Runs `n` engine calls of the audio callback.
fn calls(audio: &mut Audio, n: usize) {
    let mut out = vec![0i16; 2 * CALL_FRAMES];
    for _ in 0..n {
        audio.render_i16(&mut out);
    }
}

/// The lit LEDs of the main panel, left to right, by their part names.
fn lit(panel: &Panel) -> Vec<bool> {
    let controls = orb_ui::layout::controls();
    (1..=9)
        .map(|i| {
            let c = controls
                .iter()
                .find(|c| c.id == format!("comp.gr{i}"))
                .unwrap();
            !panel.part_name(c, false, 0.0).starts_with("gr_led_off_")
        })
        .collect()
}

#[test]
fn the_master_comp_switch_turns_the_bar_off() {
    let samples = Some(rbm::standard::builtin_samples());
    let (mut link, mut audio) = Link::new(Arc::new(Tables::new()), samples.clone());
    let mut panel = Panel::new();
    link.load_new(samples, &mut panel);
    // the new song routes the compressor to the master; make sure it is on
    if !panel.toggle_on("master.comp") {
        panel.press("master.comp", 0.0);
    }
    link.sync(&mut panel);
    calls(&mut audio, 64);
    link.sync(&mut panel);
    assert!(panel.toggle_on("master.comp"));
    // silence: the compressor's gain rises toward unity (byte 18), LEDs 1 to 5 (16..=25)
    assert!((16..=18).contains(&panel.gr_byte), "byte {}", panel.gr_byte);
    assert_eq!(gr_lit(panel.gr_byte), 5);
    let mut want = vec![true; 5];
    want.extend([false; 4]);
    assert_eq!(lit(&panel), want);

    // the master's COMP off: the engine clears the byte, no LED is lit
    panel.press("master.comp", 0.0);
    link.sync(&mut panel);
    calls(&mut audio, 64);
    link.sync(&mut panel);
    assert!(!panel.toggle_on("master.comp"));
    assert_eq!(panel.gr_byte, 0);
    assert_eq!(lit(&panel), vec![false; 9]);
}
