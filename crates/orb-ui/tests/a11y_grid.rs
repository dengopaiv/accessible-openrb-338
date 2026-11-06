//! The drum grids of screen reader mode (task-191, A11Y-7, docs/ui_spec.md 20.6): the Grid node
//! of each drum machine, its keys, its cells' texts, F2's drum part, and the grid's own
//! pattern while playback moves the row (decision task-209).
//!
//! The steps set through a grid are compared with the same steps clicked by mouse on a second
//! app (the panel at 1:1, synthetic pointer events, as `tests/interaction.rs`): the engine
//! events (`Panel::events`) must be equal, on both machines and with each 909 modifier.

#[path = "common/a11y.rs"]
mod a11y;

use std::sync::Arc;

use a11y::{AppTree, Frames};
use eframe::egui::accesskit::{Action, Role};
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use orb_engine::song::Device;
use orb_engine::tables::Tables;
use orb_player::output::Health;
use orb_player::{Audio, CALL_FRAMES};
use orb_ui::a11y::{grid, nav};
use orb_ui::app::App;
use orb_ui::layout::{self, PANEL_H, PANEL_W};
use orb_ui::link::{Link, Msg};
use orb_ui::model::{Event as Out, Panel};
use orb_ui::screen_reader::ScreenReaderSetting;

/// The app with screen reader mode on or off and no menu bar (panel pixels are points).
fn app(screen_reader: bool) -> AppTree {
    let mut h = AppTree::new();
    h.app.menu = false;
    h.app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.frame(vec![]);
    h.app.panel.events.clear();
    h
}

fn take(h: &mut AppTree) -> Vec<Out> {
    std::mem::take(&mut h.app.panel.events)
}

/// Ctrl as Windows reports it (Ctrl is also the command key there).
const CTRL: Modifiers = Modifiers {
    alt: false,
    ctrl: true,
    shift: false,
    mac_cmd: false,
    command: true,
};
const ALT: Modifiers = Modifiers::ALT;
const CTRL_ALT: Modifiers = Modifiers { alt: true, ..CTRL };

/// Presses and releases a key with modifiers in two frames.
fn key(h: &mut AppTree, key: Key, modifiers: Modifiers) {
    let ev = |pressed| Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers,
    };
    h.frame(vec![Event::ModifiersChanged(modifiers), ev(true)]);
    h.frame(vec![ev(false), Event::ModifiersChanged(Modifiers::NONE)]);
}

fn keys(h: &mut AppTree, k: Key, n: usize) {
    for _ in 0..n {
        key(h, k, Modifiers::NONE);
    }
}

/// A mouse click at `p` with `mods` held.
fn click_at(h: &mut AppTree, p: Pos2, mods: Modifiers) {
    let button = |pressed| Event::PointerButton {
        pos: p,
        button: PointerButton::Primary,
        pressed,
        modifiers: mods,
    };
    h.frame(vec![Event::PointerMoved(p)]);
    h.frame(vec![Event::ModifiersChanged(mods), button(true)]);
    h.frame(vec![
        button(false),
        Event::ModifiersChanged(Modifiers::NONE),
    ]);
}

/// The center of a step button (0-based) of the 808 or the 909.
fn step_pos(is808: bool, s: usize) -> Pos2 {
    let id = format!("{}.step{}", if is808 { "tr808" } else { "tr909" }, s + 1);
    let c = layout::controls().into_iter().find(|c| c.id == id).unwrap();
    Pos2::new(c.x + c.w / 2.0, c.y + c.h / 2.0)
}

/// A point of an instrument's legend: the 808's legend row (22 px from x 10 of the section,
/// y 71..84), the 909's (AC 4..25, BD..HT 42 px from 26, RS..RC 21 px from 236; y 4..17).
fn legend_pos(is808: bool, i: usize) -> Pos2 {
    if is808 {
        let (x, y) = layout::TR808_ORIGIN;
        Pos2::new(x + 10.0 + 22.0 * i as f32 + 11.0, y + 77.0)
    } else {
        let (x, y) = layout::TR909_ORIGIN;
        let dx = match i {
            0 => 14.0,
            1..=5 => 26.0 + 42.0 * (i - 1) as f32 + 21.0,
            _ => 236.0 + 21.0 * (i - 6) as f32 + 10.0,
        };
        Pos2::new(x + dx, y + 10.0)
    }
}

fn focus(h: &mut AppTree, id: eframe::egui::Id) {
    h.send(id, Action::Focus, None);
}

/// One edit: the instrument, the step (0-based) and the modifiers held.
type Edit = (usize, usize, Modifiers);

/// Makes the edits with the mouse on one app and through the grid on another; returns both
/// event lists.
fn both(is808: bool, edits: &[Edit], flam: bool) -> (Vec<Out>, Vec<Out>) {
    let mut mouse = app(false);
    let mut sr = app(true);
    if flam {
        for h in [&mut mouse, &mut sr] {
            h.app.panel.press("tr909.flam_button", 0.0);
        }
    }
    focus(&mut sr, grid::grid_id(is808));
    for &(i, s, mods) in edits {
        click_at(&mut mouse, legend_pos(is808, i), Modifiers::NONE);
        click_at(&mut mouse, step_pos(is808, s), mods);

        // the grid: Up / Down to the instrument, Home and Right to the step, Space
        let at = grid::selected(&sr.app.panel, is808);
        if i < at {
            keys(&mut sr, Key::ArrowUp, at - i);
        } else {
            keys(&mut sr, Key::ArrowDown, i - at);
        }
        key(&mut sr, Key::Home, Modifiers::NONE);
        keys(&mut sr, Key::ArrowRight, s);
        key(&mut sr, Key::Space, mods);
        assert_eq!(grid::selected(&sr.app.panel, is808), i);
        assert_eq!(
            grid::selected(&mouse.app.panel, is808),
            i,
            "the legend click"
        );
    }
    (take(&mut mouse), take(&mut sr))
}

#[test]
fn the_808_grid_sets_steps_as_clicks_do() {
    let edits: Vec<Edit> = [
        (1, 0),
        (1, 4),
        (1, 8),
        (1, 12),
        (1, 4), // off again
        (0, 2),
        (3, 5),
        (11, 15),
    ]
    .iter()
    .map(|&(i, s)| (i, s, Modifiers::NONE))
    // the 808's mouse ignores the modifiers: so does the grid
    .chain([(2, 7, CTRL), (2, 7, ALT), (9, 1, CTRL_ALT)])
    .collect();
    let (mouse, sr) = both(true, &edits, false);
    assert_eq!(mouse.len(), edits.len(), "{mouse:?}");
    assert_eq!(sr, mouse);
}

#[test]
fn the_909_grid_sets_steps_as_clicks_do_with_each_modifier() {
    let n = Modifiers::NONE;
    let edits: Vec<Edit> = vec![
        // the click cycle: off, note, accent, off
        (1, 0, n),
        (1, 0, n),
        (1, 0, n),
        (1, 2, CTRL),
        (1, 2, CTRL),
        (1, 3, ALT),
        (1, 4, CTRL_ALT),
        (1, 4, CTRL_ALT),
        (1, 5, CTRL_ALT),
        // the accent row: on / off
        (0, 6, n),
        (0, 6, ALT),
        // a hi-hat clears the other on its step
        (8, 9, n),
        (9, 9, n),
        (11, 15, n),
    ];
    let (mouse, sr) = both(false, &edits, false);
    assert!(mouse.len() >= edits.len(), "{mouse:?}");
    assert!(
        mouse.contains(&Out::Step909 {
            instrument: 8,
            pattern: 0,
            step: 9,
            value: 0
        }),
        "the closed hi-hat cleared: {mouse:?}"
    );
    assert_eq!(sr, mouse);
}

#[test]
fn the_909_flam_button_acts_on_the_grid_as_on_the_mouse() {
    let n = Modifiers::NONE;
    let edits: Vec<Edit> = vec![(1, 0, n), (1, 0, n), (2, 3, CTRL), (2, 4, ALT)];
    let (mouse, sr) = both(false, &edits, true);
    assert_eq!(
        mouse.first(),
        Some(&Out::Step909 {
            instrument: 1,
            pattern: 0,
            step: 0,
            value: 3
        })
    );
    assert_eq!(sr, mouse);
}

#[test]
fn the_grid_node_its_cells_and_the_keys() {
    let mut h = app(true);
    let gid = grid::grid_id(true);
    assert_eq!(
        nav::section_of(&nav::order(&layout::controls()), gid),
        Some(3)
    );
    focus(&mut h, gid);
    let t = h.tree();
    assert_eq!(t.focused_id(), gid.accesskit_id());
    let g = t.by_id(gid).expect("the grid's node");
    assert_eq!(a11y::role(g), Role::Grid);
    assert_eq!(a11y::name(g), Some("Steps"));
    assert_eq!((g.row_count(), g.column_count()), (Some(12), Some(16)));
    // 12 rows by 16 steps, the grid's children
    assert_eq!(g.children().len(), 12 * 16);
    let (parent, _) = t.parent(gid.accesskit_id()).unwrap();
    assert_eq!(t.node(parent).and_then(a11y::name), Some("TR-808"));
    // the active cell: the selected instrument (Bass drum at start), step 1
    let active = |h: &AppTree| {
        let g = h.tree().by_id(gid).unwrap();
        let c = h.tree().node(g.active_descendant().unwrap()).unwrap();
        a11y::name(c).unwrap().to_string()
    };
    assert_eq!(active(&h), "Bass drum, step 1, off");
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    key(&mut h, Key::Space, Modifiers::NONE);
    assert_eq!(active(&h), "Bass drum, step 3, on");
    key(&mut h, Key::ArrowUp, Modifiers::NONE);
    assert_eq!(active(&h), "Accent, step 3, off");
    key(&mut h, Key::ArrowUp, Modifiers::NONE);
    assert_eq!(h.app.panel.inst808, 0, "Up stops at Accent");
    keys(&mut h, Key::ArrowDown, 3);
    assert_eq!(active(&h), "Low tom, step 3, off");
    // the slot is named by the voice its switch plays now
    h.app.panel.set_switch("tr808.slot3_switch", 0);
    h.frame(vec![]);
    assert_eq!(active(&h), "Low conga, step 3, off");
    key(&mut h, Key::End, Modifiers::NONE);
    assert_eq!(active(&h), "Low conga, step 16, off");
    // the keys moved nothing else: no focus move, no engine event but the one step (and the
    // switch the test set)
    assert_eq!(h.tree().focused_id(), gid.accesskit_id());
    assert_eq!(
        take(&mut h),
        [
            Out::Step808 {
                instrument: 1,
                pattern: 0,
                step: 2,
                on: true
            },
            Out::Set {
                id: "tr808.slot3_switch".into(),
                value: 0
            }
        ]
    );

    // the 909's cells say accent and flam
    let g9 = grid::grid_id(false);
    focus(&mut h, g9);
    key(&mut h, Key::Space, Modifiers::NONE);
    key(&mut h, Key::Space, Modifiers::NONE);
    let name9 = |h: &AppTree| {
        let g = h.tree().by_id(g9).unwrap();
        let c = h.tree().node(g.active_descendant().unwrap()).unwrap();
        a11y::name(c).unwrap().to_string()
    };
    assert_eq!(name9(&h), "Bass drum, step 1, accent");
    key(&mut h, Key::ArrowRight, Modifiers::NONE);
    key(&mut h, Key::Space, CTRL_ALT);
    assert_eq!(name9(&h), "Bass drum, step 2, flam");
}

#[test]
fn steps_past_the_pattern_length_are_not_offered() {
    let mut h = app(true);
    let gid = grid::grid_id(false);
    focus(&mut h, gid);
    key(&mut h, Key::End, Modifiers::NONE);
    // the 909's pattern length down to 6 (its spin button)
    for _ in 0..10 {
        h.app.panel.arrow("pattern4.steps_down");
    }
    assert_eq!(grid::length(&h.app.panel, false), 6);
    h.frame(vec![]);
    let g = h.tree().by_id(gid).unwrap();
    assert_eq!(g.column_count(), Some(6));
    assert_eq!(g.children().len(), 12 * 6);
    // the cursor came back within the length, and Right stops at its end
    let active = |h: &AppTree| {
        let g = h.tree().by_id(gid).unwrap();
        let c = h.tree().node(g.active_descendant().unwrap()).unwrap();
        a11y::name(c).unwrap().to_string()
    };
    assert_eq!(active(&h), "Bass drum, step 6, off");
    keys(&mut h, Key::ArrowRight, 3);
    assert_eq!(active(&h), "Bass drum, step 6, off");
    assert!(h.tree().by_id(grid::cell_id(false, 1, 6)).is_none());
    // F2 does not read a step past the length
    h.app.panel.steps909[1][2] = 1;
    h.app.panel.steps909[1][10] = 1;
    key(&mut h, Key::F2, Modifiers::NONE);
    assert_eq!(
        h.app.announcer.text(),
        Some("TR-909, pattern A1, 6 steps, shuffle off. Bass drum: 3")
    );
}

#[test]
fn f2_reads_a_known_drum_pattern() {
    let mut h = app(true);
    let p = &mut h.app.panel;
    p.steps808 = [[false; 16]; 12];
    for s in [0, 4, 8, 12] {
        p.steps808[1][s] = true;
    }
    for s in [2, 6, 10, 14] {
        p.steps808[11][s] = true;
    }
    p.steps808[0][8] = true;
    focus(&mut h, grid::grid_id(true));
    key(&mut h, Key::F2, Modifiers::NONE);
    assert_eq!(
        h.app.announcer.text(),
        Some(
            "TR-808, pattern A1, 16 steps, shuffle off. Accent: 9. Bass drum: 1, 5, 9, 13. \
             Closed hi-hat: 3, 7, 11, 15"
        )
    );
    // from another control of the section too (the 808's level fader), and on the 909
    focus(&mut h, a11y::hit_id("fader", "mixer3.level"));
    key(&mut h, Key::F2, Modifiers::NONE);
    assert!(
        h.app
            .announcer
            .text()
            .unwrap()
            .ends_with("Closed hi-hat: 3, 7, 11, 15")
    );
    let p = &mut h.app.panel;
    p.steps909 = [[0; 16]; 12];
    p.steps909[1][0] = 1;
    p.steps909[1][4] = 2;
    p.steps909[2][4] = 3;
    p.steps909[0][4] = 2;
    focus(&mut h, grid::grid_id(false));
    key(&mut h, Key::F2, Modifiers::NONE);
    assert_eq!(
        h.app.announcer.text(),
        Some(
            "TR-909, pattern A1, 16 steps, shuffle off. Accent: 5. Bass drum: 1, 5 accent. \
             Snare drum: 5 flam"
        )
    );
}

#[test]
fn the_selection_frame_follows_the_grids_row() {
    let mut h = app(true);
    focus(&mut h, grid::grid_id(true));
    keys(&mut h, Key::ArrowDown, 4);
    assert_eq!(h.app.panel.inst808, 5);
    focus(&mut h, grid::grid_id(false));
    keys(&mut h, Key::ArrowDown, 8);
    keys(&mut h, Key::ArrowUp, 1);
    assert_eq!(h.app.panel.inst909, 8);
    // the frame is drawn where `App::instrument_selection` puts it for the selected
    // instrument: the same selection the legend click makes (tests/interaction.rs)
    let mut mouse = app(false);
    click_at(&mut mouse, legend_pos(true, 5), Modifiers::NONE);
    click_at(&mut mouse, legend_pos(false, 8), Modifiers::NONE);
    assert_eq!(
        (mouse.app.panel.inst808, mouse.app.panel.inst909),
        (h.app.panel.inst808, h.app.panel.inst909)
    );
}

#[test]
fn with_the_mode_off_there_is_no_grid() {
    let mut h = app(false);
    h.frame(vec![]);
    assert!(h.tree().by_id(grid::grid_id(true)).is_none());
    assert!(h.tree().by_role(Role::Grid).next().is_none());
    // the arrows and Space are the original's (Space plays)
    key(&mut h, Key::Space, Modifiers::NONE);
    assert!(h.app.panel.transport.playing);
    // no pin: the strips show and edit the row's pattern, as the original
    h.app.panel.adopt_pattern(2, 3);
    h.frame(vec![]);
    assert_eq!(h.app.panel.drum_pin, [None; 2]);
    assert_eq!(h.app.panel.edit_pattern(2), 3);
}

/// An app on a player (the new song, the built-in kit) with its audio callback, in screen
/// reader mode, without the menu bar, the link logging what it sends, settled (as
/// tests/a11y_announce.rs).
fn player() -> (AppTree, Audio) {
    let samples = Some(rbm::standard::builtin_samples());
    let (mut link, audio) = Link::new(Arc::new(Tables::new()), samples.clone());
    link.log = Some(vec![]);
    let mut app = App::with_link(Panel::new(), link, None);
    app.samples = samples;
    app.audio_health = Some(Health::started());
    app.menu = false;
    app.settings.screen_reader = ScreenReaderSetting::On;
    app.new_now();
    let mut h = AppTree {
        frames: Frames::new(Vec2::new(PANEL_W, PANEL_H)),
        app,
    };
    for _ in 0..4 {
        h.frame(vec![]);
    }
    (h, audio)
}

/// Runs `n` engine calls of the audio callback.
fn calls(audio: &mut Audio, n: usize) {
    let mut out = vec![0i16; 2 * CALL_FRAMES];
    for _ in 0..n {
        audio.render_i16(&mut out);
    }
}

/// The TR-808 steps the link sent to the engine since the last call, as `[pattern, step,
/// instrument, value]` (the device's sub 62, `Link::drum_step`).
fn sent808(h: &mut AppTree) -> Vec<[u8; 4]> {
    let log = h.app.link.as_mut().unwrap().log.as_mut().unwrap();
    std::mem::take(log)
        .into_iter()
        .filter_map(|m| match m {
            Msg::Device { dev, sub: 62, data } if dev == Device::Tr808.id() && data.len() == 4 => {
                Some([data[0], data[1], data[2], data[3]])
            }
            _ => None,
        })
        .collect()
}

/// The name of a grid's active cell.
fn active(h: &AppTree, gid: eframe::egui::Id) -> String {
    let t = h.tree();
    let g = t.by_id(gid).unwrap();
    let c = t.node(g.active_descendant().unwrap()).unwrap();
    a11y::name(c).unwrap().to_string()
}

/// Decision task-209 on the drum grids: while playback moves the 808 to another pattern, the
/// focused grid stays on its pattern and nothing is spoken, with Options > Step Editor Follows
/// Playback off and on; on, the grid moves at the next input; F2 says where it is; Space edits
/// the grid's pattern; a pattern chosen on the panel moves it; a tap goes to the playing one.
/// On a real link and engine, so the strips are filled with the pinned pattern's steps.
#[test]
fn playback_moving_the_pattern_is_silent_on_a_grid() {
    for follows in [false, true] {
        let (mut h, mut audio) = player();
        h.app.settings.step_editor_follows = follows;
        assert_eq!(
            h.app.panel.steps808, [[false; 16]; 12],
            "a new song's A1 is empty"
        );
        // A2 (pattern 1): a bass drum on step 3, sent to the engine directly
        h.app.panel.events.push(Out::Step808 {
            instrument: 1,
            pattern: 1,
            step: 2,
            on: true,
        });
        // the grid: a bass drum on step 1 of A1
        let gid = grid::grid_id(true);
        focus(&mut h, gid);
        key(&mut h, Key::Space, Modifiers::NONE);
        let first = "Bass drum, step 1, on";
        assert_eq!(active(&h, gid), first);
        assert_eq!(sent808(&mut h), [[1, 2, 1, 1], [0, 0, 1, 1]]);

        // the engine moves the 808 to A2 (as song mode does), not the panel's input
        let n = h.app.announcer.serial();
        h.app.panel.events.push(Out::Pattern { row: 2, pattern: 1 });
        for _ in 0..4 {
            calls(&mut audio, 2);
            h.frame(vec![]);
        }
        assert_eq!(h.app.panel.rows[2].pattern, 1, "the panel shows A2");
        assert_eq!(h.app.panel.drum_pin[0], Some(0), "follows {follows}");
        assert_eq!(active(&h, gid), first, "follows {follows}");
        assert!(h.app.panel.steps808[1][0], "the strip shows A1 still");
        assert_eq!(
            h.app.announcer.serial(),
            n,
            "nothing spoken, follows {follows}"
        );

        // F2 is input: on, the grid has moved to A2; off, F2 says where the grid is
        key(&mut h, Key::F2, Modifiers::NONE);
        let f2 = h.app.announcer.text().unwrap().to_string();
        let row = "TR-808, pattern A2, 16 steps, shuffle off. ";
        if follows {
            assert_eq!(f2, format!("{row}Bass drum: 3"));
            assert_eq!(active(&h, gid), "Bass drum, step 1, off");
        } else {
            assert_eq!(f2, format!("{row}Grid on pattern A1: Bass drum: 1"));
            assert_eq!(active(&h, gid), first);
        }
        // Space edits the grid's pattern: on, A2's step 1 goes on; off, A1's goes off
        key(&mut h, Key::Space, Modifiers::NONE);
        let (p, v) = (u8::from(follows), u8::from(follows));
        assert_eq!(sent808(&mut h), [[p, 0, 1, v]], "follows {follows}");
        if !follows {
            // a tap goes to the playing pattern (A2) and leaves the strip (A1) as it is
            h.app.panel.playing_step808 = Some(5);
            h.app.panel.tap_drum(true, 2);
            h.frame(vec![]);
            assert_eq!(sent808(&mut h), [[1, 5, 2, 1]]);
            assert!(!h.app.panel.steps808[2][5]);
            // the user chooses A2 on the panel: the grid goes there
            h.app.panel.press("pattern3.pattern2", 0.0);
            h.frame(vec![]);
            assert_eq!(h.app.panel.drum_pin[0], Some(1));
            keys(&mut h, Key::ArrowRight, 2);
            assert_eq!(active(&h, gid), "Bass drum, step 3, on");
        }
    }
}
