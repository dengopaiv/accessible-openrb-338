//! The skin layer (task-266, docs/ui_spec.md section 22): the one controller drives any skin's
//! layout. A test skin moves controls, places one twice, lacks others and has one the model
//! does not know, with its own knob, fader, strip and hit-area geometry; a second SVG skin is
//! made of a tiny dummy part set and table, in units of half the panel's, driven and drawn
//! through the same controller and renderer.

use std::sync::Arc;

use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use orb_ui::app::App;
use orb_ui::art::PartSet;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::model::Panel;
use orb_ui::skin::Skin;
use orb_ui::skin::svg::SvgSkin;

struct Harness {
    ctx: egui::Context,
    app: App,
    time: f64,
    pos: Pos2,
    shapes: usize,
}

impl Harness {
    fn new(skin: Arc<dyn Skin>) -> Self {
        let ctx = egui::Context::default();
        egui_extras::install_image_loaders(&ctx);
        let mut app = App::new(Panel::new(), None);
        app.set_skin(skin);
        let mut h = Self {
            ctx,
            app,
            time: 0.0,
            pos: Pos2::ZERO,
            shapes: 0,
        };
        h.frame(vec![]);
        h
    }

    fn frame(&mut self, events: Vec<Event>) {
        self.frame_mods(events, Modifiers::NONE);
    }

    fn frame_mods(&mut self, mut events: Vec<Event>, modifiers: Modifiers) {
        self.time += 0.016;
        events.insert(0, Event::ModifiersChanged(modifiers));
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(PANEL_W, PANEL_H))),
            time: Some(self.time),
            events,
            ..Default::default()
        };
        let app = &mut self.app;
        let mut out = self.ctx.run_ui(input, |ui| app.show(ui));
        self.shapes = out.shapes.len();
        out.textures_delta.clear();
    }

    fn move_to(&mut self, p: Pos2) {
        self.pos = p;
        self.frame(vec![Event::PointerMoved(p)]);
    }

    fn button(&mut self, pressed: bool) {
        let pos = self.pos;
        self.frame(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }

    fn click_at(&mut self, p: Pos2) {
        self.move_to(p);
        self.button(true);
        self.button(false);
    }

    fn key(&mut self, key: Key) {
        for pressed in [true, false] {
            self.frame(vec![Event::Key {
                key,
                physical_key: Some(key),
                pressed,
                repeat: false,
                modifiers: Modifiers::NONE,
            }]);
        }
    }
}

const HEADER: &str = "id\tsection\tkind\tpart\tx\ty\tw\th\tparams\n";

/// A skin on the embedded parts whose layout differs from the modern one: Play moved and
/// placed twice, Stop missing (Space still stops), a control the model does not know, the
/// cutoff knob with a span of 20, a mixer fader with a 45-high track, a smaller 808 strip and
/// legend, other row areas.
fn moved_skin() -> SvgSkin {
    let rows = [
        "panel\t-\tpanel\t-\t0\t0\t624\t510",
        "transport.bg\ttransport\tbg\tbg_transport\t16\t0\t592\t50",
        "logo\ttransport\tlogo\tlogo\t474\t1\t134\t48",
        "pattern1.row\tpattern1\trow\t-\t0\t300\t624\t50\trow=0",
        "pattern3.row\tpattern3\trow\t-\t0\t400\t624\t50\trow=2",
        "transport.play\ttransport\ttrigger\tbtn_play_rest\t10\t10\t32\t25",
        "transport.play\ttransport\ttrigger\tbtn_play_rest\t300\t60\t32\t25",
        "test.extra\ttest\ttrigger\tbtn_stop_rest\t450\t60\t32\t25",
        "transport.tempo\ttransport\tdisplay\tseg_0..9,seg_blank\t500\t100\t27\t16",
        "transport.tempo_up\ttransport\trepeat\tarrow_inc_rest\t540\t100\t11\t9",
        "tb303_1.cutoff\ttb303_1\tknob\tknob_303\t100\t100\t30\t30\tspan=20",
        "mixer1.level\tmixer1\tfader\tfader_track\t200\t100\t15\t45\thandle=11",
        "tr808.step1\ttr808\tsteps\tstep808_red_off\t50\t300\t10\t20",
        "tr808.strip\ttr808\tstrip\t-\t50\t300\t160\t20\tcells=16",
        "tr808.legend\ttr808\tselect\t-\t50\t340\t120\t10\tcells=50:10:0 count=12",
        "tr808.mark_AC\ttr808\tmark\tsel808_AC\t50\t340\t10\t10\tslot=0 lit=- litsel=-",
    ];
    let table = format!("{HEADER}{}\n", rows.join("\n"));
    SvgSkin::new(&table, PartSet::embedded().clone()).expect("the test skin")
}

#[test]
fn the_controller_drives_a_skin_with_moved_extra_and_missing_controls() {
    let mut h = Harness::new(Arc::new(moved_skin()));
    assert!(h.shapes > 0, "the skin draws");
    assert_eq!(h.app.skin().layout().controls.len(), 8);
    // Play at its new place; the modern place of Stop (and Play) is nothing here
    h.click_at(Pos2::new(160.0, 33.0));
    assert!(!h.app.panel.transport.playing);
    h.click_at(Pos2::new(26.0, 22.0));
    assert!(h.app.panel.transport.playing);
    // Stop has no control in this skin, its key still works
    h.key(Key::Space);
    assert!(!h.app.panel.transport.playing);
    // the second Play
    h.click_at(Pos2::new(316.0, 72.0));
    assert!(h.app.panel.transport.playing);
    h.key(Key::Space);
    // a control the model does not know takes the click and changes nothing
    let before = h.app.panel.transport.tempo;
    h.click_at(Pos2::new(466.0, 72.0));
    assert!(!h.app.panel.transport.playing);
    assert_eq!(h.app.panel.transport.tempo, before);
    // the tempo arrow at its new place: one BPM per click
    h.click_at(Pos2::new(545.0, 104.0));
    assert_eq!(h.app.panel.transport.tempo, before + 1000);

    // the knob: a span of 20 skin units, so 10 up is half the range
    h.app.panel.set_knob("tb303_1.cutoff", 0);
    h.move_to(Pos2::new(115.0, 115.0));
    h.button(true);
    h.move_to(Pos2::new(115.0, 105.0));
    h.button(false);
    assert_eq!(h.app.panel.knob("tb303_1.cutoff"), 500);
    // the knob's modern place does nothing
    h.click_at(Pos2::new(210.0, 77.0));
    assert_eq!(h.app.panel.knob("tb303_1.cutoff"), 500);

    // the fader: a track of 45 and a handle of 11 (travel 34, k = 294): a press at the top
    // jumps to the top, at the bottom to the bottom
    h.app.panel.set_fader("mixer1.level", 0);
    h.click_at(Pos2::new(207.0, 100.5));
    assert_eq!(h.app.panel.fader("mixer1.level"), 999);
    h.click_at(Pos2::new(207.0, 144.5));
    assert_eq!(h.app.panel.fader("mixer1.level"), 0);
    // a press at 20 from the top: (45 - 20 - 5 + 2) * 294 / 10
    h.click_at(Pos2::new(207.0, 120.5));
    assert_eq!(h.app.panel.fader("mixer1.level"), 22 * 294 / 10);

    // the 808's strip: cells of 10 from x 50; a drag paints every step crossed
    let inst = h.app.panel.inst808;
    h.move_to(Pos2::new(85.0, 310.0));
    h.button(true);
    h.move_to(Pos2::new(115.0, 310.0));
    h.button(false);
    let steps = h.app.panel.steps808[inst];
    assert_eq!(&steps[2..7], &[false, true, true, true, true][..]);
    assert!(!steps[7]);
    // the legend: cells of 10 from x 50
    h.click_at(Pos2::new(75.0, 345.0));
    assert_eq!(h.app.panel.inst808, 2);

    // the rows' focus: this skin's areas
    h.click_at(Pos2::new(10.0, 420.0));
    assert_eq!(h.app.panel.focus, 2);
    h.click_at(Pos2::new(10.0, 320.0));
    assert_eq!(h.app.panel.focus, 0);
    // the modern row 2 area is nothing here
    h.click_at(Pos2::new(30.0, 200.0));
    assert_eq!(h.app.panel.focus, 0);
    assert!(h.shapes > 0);
}

/// A part of the dummy set: a `w x h` rectangle of `color` with a bleed of `b`.
fn svg(w: f32, h: f32, b: f32, color: &str) -> Vec<u8> {
    let (vw, vh) = (w + 2.0 * b, h + 2.0 * b);
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{vw}\" height=\"{vh}\" \
         viewBox=\"{} {} {vw} {vh}\"><rect width=\"{w}\" height=\"{h}\" fill=\"{color}\"/></svg>",
        -b, -b
    )
    .into_bytes()
}

/// The dummy part set: a background, a push button in three states, a knob, a fader, seven
/// segment digits and an LED, named as a theme names its parts.
fn dummy_parts() -> PartSet {
    let mut files = vec![
        ("dummy_bg".to_string(), svg(312.0, 255.0, 0.0, "#334")),
        ("dummy_btn_rest".into(), svg(20.0, 10.0, 1.0, "#888")),
        ("dummy_btn_pressed".into(), svg(20.0, 10.0, 1.0, "#666")),
        ("dummy_btn_lit".into(), svg(20.0, 10.0, 1.0, "#8f8")),
        ("dummy_knob_body".into(), svg(16.0, 16.0, 0.0, "#aaa")),
        ("dummy_knob_pointer".into(), svg(16.0, 16.0, 0.0, "#000")),
        ("dummy_fader_track".into(), svg(8.0, 30.0, 0.0, "#222")),
        ("dummy_fader_handle".into(), svg(8.0, 5.0, 0.0, "#eee")),
        ("dummy_led_on".into(), svg(4.0, 4.0, 1.0, "#f00")),
        ("dummy_led_off".into(), svg(4.0, 4.0, 1.0, "#400")),
        ("dummy_seg_blank".into(), svg(5.0, 8.0, 0.0, "#000")),
    ];
    for d in 0..10 {
        files.push((format!("dummy_seg_{d}"), svg(5.0, 8.0, 0.0, "#f80")));
    }
    PartSet::from_svgs("dummy", files).expect("the dummy part set")
}

/// The dummy table, in half the panel's units (312 x 255).
fn dummy_table() -> String {
    let rows = [
        "panel\t-\tpanel\t-\t0\t0\t312\t255",
        "all.bg\tall\tbg\tdummy_bg\t0\t0\t312\t255",
        "transport.play\ttransport\ttrigger\tdummy_btn_rest\t10\t10\t20\t10",
        "tb303_1.cutoff\ttb303_1\tknob\tdummy_knob\t40\t10\t16\t16\tspan=10",
        "mixer1.level\tmixer1\tfader\tdummy_fader_track\t70\t10\t8\t30\thandle=5",
        "transport.tempo\ttransport\tdisplay\tdummy_seg_0..9,dummy_seg_blank\t90\t10\t15\t8",
        "tb303_1.pitch_led\ttb303_1\tled\tdummy_led_off\t110\t10\t4\t4",
    ];
    format!("{HEADER}{}\n", rows.join("\n"))
}

#[test]
fn a_second_svg_skin_is_data_only_and_runs_through_the_same_controller_and_renderer() {
    let parts = Arc::new(dummy_parts());
    assert_eq!(
        parts.part("dummy_btn_rest").map(|p| (p.w, p.h, p.bleed)),
        Some((20.0, 10.0, 1.0))
    );
    let skin = Arc::new(SvgSkin::new(&dummy_table(), parts).expect("the dummy skin"));
    assert_eq!(skin.layout().size, Vec2::new(312.0, 255.0));
    let mut h = Harness::new(skin.clone());
    // the window is 624 x 510 points: 2 points per skin unit
    h.frame(vec![]);
    assert!(h.shapes > 0);
    assert!(orb_ui::app::PartTextures::count(&h.ctx) > 0);
    // Play at (10, 10) 20 x 10 units: points (20..60, 20..40)
    h.click_at(Pos2::new(40.0, 30.0));
    assert!(h.app.panel.transport.playing);
    let play = &skin.layout().controls[0];
    assert_eq!(
        skin.part_name(&h.app.panel, play, false, 0.0),
        "dummy_btn_lit"
    );
    // the display's digits and the LED are the set's
    let tempo = &skin.layout().controls[3];
    assert_eq!(tempo.id, "transport.tempo");
    let led = &skin.layout().controls[4];
    assert_eq!(
        skin.part_name(&h.app.panel, led, false, 0.0),
        "dummy_led_off"
    );
    // the knob: 5 units (10 points) up is half its span of 10
    h.app.panel.set_knob("tb303_1.cutoff", 0);
    h.move_to(Pos2::new(96.0, 36.0));
    h.button(true);
    h.move_to(Pos2::new(96.0, 26.0));
    h.button(false);
    assert_eq!(h.app.panel.knob("tb303_1.cutoff"), 500);
    // the fader: a press at its track's top jumps to the top
    h.app.panel.set_fader("mixer1.level", 0);
    h.click_at(Pos2::new(148.0, 21.0));
    assert_eq!(h.app.panel.fader("mixer1.level"), 999);
    h.frame(vec![]);
    // every part drawn is the dummy set's
    let missing = orb_ui::art::missing();
    assert!(
        missing.iter().all(|m| !m.starts_with("dummy_")),
        "{missing:?}"
    );
}
