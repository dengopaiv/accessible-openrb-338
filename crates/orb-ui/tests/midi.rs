//! MIDI remote control on the panel (task-182): a byte stream through the Standard Mapping
//! (`orb_player::midi`), its commands through the link to a real player, and the panel shows
//! the values as it shows a mouse change. Headless; the test drives the audio callback itself.

use std::sync::Arc;

use orb_engine::tables::Tables;
use orb_player::midi::{StandardMapping, Timed};
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::link::Link;
use orb_ui::model::Panel;

struct Harness {
    link: Link,
    audio: Audio,
    panel: Panel,
    midi: StandardMapping,
}

impl Harness {
    fn new() -> Self {
        let (link, audio) = Link::new(
            Arc::new(Tables::new()),
            Some(rbm::standard::builtin_samples()),
        );
        let mut h = Harness {
            link,
            audio,
            panel: Panel::new(),
            midi: StandardMapping::new(),
        };
        h.calls(2);
        h
    }

    /// Runs `n` engine calls, then a UI turn that takes the snapshot into the panel and the
    /// mapping.
    fn calls(&mut self, n: usize) {
        let mut out = vec![0i16; 2 * CALL_FRAMES];
        for _ in 0..n {
            self.audio.render_i16(&mut out);
        }
        self.link.sync(&mut self.panel);
        self.midi.follow(self.link.player().snapshot());
    }

    /// Feeds `bytes` to the mapping and sends its commands.
    fn midi(&mut self, bytes: &[u8]) {
        let mut out = vec![];
        self.midi.feed(0, bytes, &mut out);
        let commands = out.iter().filter_map(|t: &Timed<_>| t.event.command());
        self.link.send(commands.collect::<Vec<_>>());
        self.calls(2);
    }
}

#[test]
fn midi_controls_show_on_the_panel() {
    let mut h = Harness::new();
    // a 303 knob, the 808 shared-slot switch, a 909 knob, a mixer fader, the delay steps
    h.midi(&[0xB0, 25, 100, 46, 127, 83, 33, 14, 90, 100, 127]);
    let p = &h.panel;
    assert_eq!(p.value("tb303_1.cutoff"), Some(100));
    assert_eq!(p.value("tr808.slot3_switch"), Some(1));
    assert_eq!(p.value("tr909.hh_level"), Some(33));
    assert_eq!(p.value("mixer2.level"), Some(90));
    assert_eq!(p.value("delay.steps"), Some(32));

    // the knob moves, as for a mouse change: the position follows the value
    h.midi(&[0xB0, 25, 10]);
    assert_eq!(h.panel.value("tb303_1.cutoff"), Some(10));

    // a toggle note: the delay's on switch, twice
    let on = h.panel.value("delay.on").unwrap();
    h.midi(&[0x90, 75, 100]);
    assert_eq!(h.panel.value("delay.on"), Some(1 - on));
    h.midi(&[0x90, 75, 100]);
    assert_eq!(h.panel.value("delay.on"), Some(on));
}

#[test]
fn midi_pattern_selection_shows_on_the_panel() {
    let mut h = Harness::new();
    // 808: bank C (note 41), pattern 4 (note 46) = 19
    h.midi(&[0x90, 41, 100, 0x90, 46, 100]);
    assert_eq!(h.panel.rows[2].pattern, 19);
    assert_eq!(h.panel.rows[2].bank, 2);
    // the 909's on/off switch
    let on = h.panel.rows[3].enabled;
    h.midi(&[0x90, 51, 100]);
    assert_eq!(h.panel.rows[3].enabled, !on);
}
