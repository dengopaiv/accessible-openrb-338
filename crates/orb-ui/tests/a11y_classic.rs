//! Screen reader mode with the classic skin (task-238): the checks of the a11y tests that read
//! the tree (the names, the focus order, the section groups, the drive keys, the drum grids and
//! the 303 step editor) run with the modern and with the classic skin, whose layout is its own
//! (upstream's task-267); and a crossfade between the skins neither moves the focus nor says
//! anything.

#[path = "common/a11y.rs"]
mod a11y;

use std::collections::BTreeSet;

use a11y::AppTree;
use eframe::egui::accesskit::{Action, Node, Role};
use eframe::egui::{Id, Key};
use orb_ui::a11y::names;
use orb_ui::a11y::{drive, grid, nav, step303};
use orb_ui::menu::Cmd;
use orb_ui::model::{Event as Out, SELECTOR_808};
use orb_ui::screen_reader::ScreenReaderSetting;
use orb_ui::settings::{Settings, Theme};
use orb_ui::skin::Ctl;

const THEMES: [Theme; 2] = [Theme::Modern, Theme::Classic];

/// An app in screen reader mode, no menu bar (panel units are points), showing `theme` once its
/// crossfade is over. Its settings are a temporary file's, not the user's.
fn app(theme: Theme) -> (AppTree, tempfile::TempDir) {
    app_mode(theme, true)
}

/// The same with screen reader mode on or off.
fn app_mode(theme: Theme, screen_reader: bool) -> (AppTree, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let mut h = AppTree::new();
    h.app.menu = false;
    h.app.settings = Settings::load_from(&dir.path().join("ui.toml"));
    h.app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    h.frame(vec![]);
    if theme != Theme::Modern {
        switch(&mut h, theme);
    }
    h.app.panel.events.clear();
    (h, dir)
}

/// View > `theme`, then frames until the crossfade is over.
fn switch(h: &mut AppTree, theme: Theme) {
    h.app.command(Cmd::SetTheme(theme));
    for _ in 0..200 {
        h.frame(vec![]);
        if h.app.fading().is_none() {
            break;
        }
    }
    assert_eq!(h.app.fading(), None, "the crossfade ends");
    assert_eq!(h.app.theme(), theme);
}

/// The controls of the skin the app shows.
fn controls(h: &AppTree) -> Vec<Ctl> {
    h.app.skin().layout().controls.clone()
}

fn find<'a>(ctls: &'a [Ctl], kind: &str, id: &str) -> &'a Ctl {
    ctls.iter()
        .find(|c| c.kind == kind && c.id == id)
        .unwrap_or_else(|| panic!("{kind} {id}"))
}

fn focus(h: &mut AppTree, id: Id) {
    h.send(id, Action::Focus, None);
    assert_eq!(h.tree().focused_id(), id.accesskit_id());
}

fn name_of(h: &AppTree, id: Id) -> String {
    let n = h.tree().by_id(id).expect("the node");
    a11y::name(n)
        .or_else(|| a11y::value(n))
        .unwrap_or_default()
        .to_string()
}

/// The node's bounds as (x0, y0, x1, y1).
fn bounds(n: &Node) -> (f64, f64, f64, f64) {
    let r = n.bounds().expect("bounds");
    (r.x0, r.y0, r.x1, r.y1)
}

/// Whether the node's bounds are the rectangle (x, y, w, h).
fn near(a: (f64, f64, f64, f64), b: (f32, f32, f32, f32)) -> bool {
    let (x, y) = (f64::from(b.0), f64::from(b.1));
    let (w, h) = (f64::from(b.2), f64::from(b.3));
    [a.0 - x, a.1 - y, a.2 - (x + w), a.3 - (y + h)]
        .iter()
        .all(|d| d.abs() < 1.5)
}

/// The names, roles and the focus of every interactive control; the same controls in both
/// skins.
#[test]
fn every_control_of_either_skin_has_a_node_and_the_skins_agree() {
    let mut sets = vec![];
    for theme in THEMES {
        let (h, _d) = app(theme);
        let t = h.tree();
        let mut set = BTreeSet::new();
        for c in controls(&h) {
            if !names::is_interactive(&c.kind) || drive::joined(&c) {
                continue;
            }
            set.insert((c.kind.clone(), c.id.clone()));
            let what = format!("{theme:?}: {} ({})", c.id, c.kind);
            if !nav::focusable(&c) {
                // the classic skin's 808 instrument knob: hidden from a screen reader
                assert_eq!(c.id, SELECTOR_808, "{what}");
                let n = t.by_layout(&c.kind, &c.id).expect("the knob's node");
                assert!(n.is_hidden(), "{what}");
                continue;
            }
            let n = t
                .by_layout(&c.kind, &c.id)
                .unwrap_or_else(|| panic!("{what} has no node"));
            assert!(!a11y::name(n).unwrap_or_default().is_empty(), "{what}");
            assert!(a11y::supports(n, Action::Focus), "{what}");
        }
        // one node per pattern selector and 303 step editor
        for r in 0..4 {
            assert!(t.by_id(drive::selector_id(r)).is_some(), "{theme:?} {r}");
        }
        for k in 0..2 {
            assert!(t.by_id(step303::editor_id(k)).is_some(), "{theme:?} {k}");
        }
        assert!(set.len() > 100, "{theme:?}: {}", set.len());
        sets.push(set);
    }
    // the classic layout has exactly one control more: the 808's instrument knob
    let knob = ("knob".to_string(), SELECTOR_808.to_string());
    assert!(sets[1].remove(&knob), "the classic layout has the knob");
    assert!(!sets[0].contains(&knob));
    assert_eq!(
        sets[0], sets[1],
        "no missing or extra control in the classic layout"
    );
    // with screen reader mode off the node stays as it was: not hidden
    let (h, _d) = app_mode(Theme::Classic, false);
    let n = h
        .tree()
        .by_layout("knob", SELECTOR_808)
        .expect("the knob's node");
    assert!(!n.is_hidden());
}

/// The sections are named groups and Tab visits the focus order of the shown skin's layout.
#[test]
fn the_focus_order_and_the_groups_follow_either_skin() {
    let mut orders = vec![];
    for theme in THEMES {
        let (mut h, _d) = app(theme);
        let order = nav::order(&controls(&h));
        assert_eq!(order.len(), 194, "{theme:?}");
        for (i, stop) in order.iter().enumerate() {
            h.key(Key::Tab);
            assert_eq!(
                h.tree().focused_id(),
                stop.id.accesskit_id(),
                "{theme:?} Tab {i}"
            );
            let (_, g) = h.tree().parent(stop.id.accesskit_id()).expect("a group");
            assert_eq!(
                a11y::name(g),
                Some(nav::SECTIONS[stop.section]),
                "{theme:?} {i}"
            );
        }
        // F6 visits each of the 9 sections
        let mut seen = BTreeSet::new();
        for _ in 0..9 {
            h.key(Key::F6);
            let f = h.tree().focused_id();
            seen.insert(
                order
                    .iter()
                    .find(|s| s.id.accesskit_id() == f)
                    .unwrap()
                    .section,
            );
        }
        assert_eq!(seen.len(), 9, "{theme:?}: F6 visits every section");
        orders.push(
            order
                .iter()
                .map(|s| (s.section, name_of(&h, s.id)))
                .collect::<Vec<_>>(),
        );
    }
    // the order inside a section is each skin's layout order (the classic panel's controls
    // are in another order in its file), the stops of a section are the same
    for o in &mut orders {
        o.sort();
    }
    assert_eq!(orders[0], orders[1], "the same stops in each section");
}

/// The pattern selector's rectangle is the skin's own, and its keys and actions work.
#[test]
fn the_pattern_selector_follows_the_skins_layout() {
    for theme in THEMES {
        let (mut h, _d) = app(theme);
        let ctls = controls(&h);
        for r in 0..4 {
            let section = format!("pattern{}", r + 1);
            // the modern panel's rectangle comes from its layout; the classic one's is literal
            // (classic.tsv: buttons 17 wide from x 17 to 85, y 69 + 115 per row to 138)
            let want = if theme == Theme::Classic {
                (17.0, 69.0 + 115.0 * r as f32, 68.0, 69.0)
            } else {
                drive::selector_rect(&ctls, &section).unwrap()
            };
            let n = h.tree().by_id(drive::selector_id(r)).unwrap();
            assert!(
                near(bounds(n), want),
                "{theme:?} row {r}: {:?} {want:?}",
                bounds(n)
            );
        }
        // Increment is the mouse's click on the next pattern button
        let start = h.app.panel.rows[1].pattern;
        h.send(drive::selector_id(1), Action::Increment, None);
        assert_eq!(
            std::mem::take(&mut h.app.panel.events),
            vec![Out::Pattern {
                row: 1,
                pattern: start + 1
            }],
            "{theme:?}"
        );
        // keys on the focused selector and on a spin button
        focus(&mut h, drive::selector_id(2));
        h.key(Key::ArrowUp);
        assert_eq!(h.app.panel.rows[2].pattern, 1, "{theme:?}");
        let tempo = h.app.panel.display("transport.tempo");
        let id = orb_ui::a11y::hit_id(find(&ctls, "display", "transport.tempo"));
        focus(&mut h, id);
        h.key(Key::ArrowUp);
        assert_ne!(h.app.panel.display("transport.tempo"), tempo, "{theme:?}");
        let n = h.tree().by_id(id).unwrap();
        assert_eq!(a11y::role(n), Role::SpinButton, "{theme:?}");
    }
}

/// The drum grids sit over the skin's step cells and take their keys.
#[test]
fn the_drum_grids_follow_the_skins_layout() {
    for theme in THEMES {
        let (mut h, _d) = app(theme);
        let ctls = controls(&h);
        for is808 in [true, false] {
            let section = if is808 { "tr808" } else { "tr909" };
            let gid = grid::grid_id(is808);
            let prefix = format!("{section}.step");
            let steps: Vec<&Ctl> = ctls
                .iter()
                .filter(|c| c.kind == "steps" && c.id.starts_with(&prefix))
                .collect();
            assert_eq!(steps.len(), 16, "{theme:?} {section}");
            let x0 = steps.iter().map(|c| c.x).fold(f32::MAX, f32::min);
            let y0 = steps.iter().map(|c| c.y).fold(f32::MAX, f32::min);
            let x1 = steps.iter().map(|c| c.x + c.w).fold(0.0, f32::max);
            let y1 = steps.iter().map(|c| c.y + c.h).fold(0.0, f32::max);
            let want = (x0, y0, x1 - x0, y1 - y0);
            let t = h.tree();
            let g = t.by_id(gid).expect("the grid");
            assert_eq!(a11y::role(g), Role::Grid);
            assert_eq!(g.children().len(), 12 * 16, "{theme:?} {section}");
            assert!(
                near(bounds(g), want),
                "{theme:?} {section}: {:?} {want:?}",
                bounds(g)
            );
            let (p, _) = t.parent(gid.accesskit_id()).unwrap();
            let group = if is808 { "808" } else { "909" };
            assert_eq!(t.node(p).and_then(a11y::name), Some(group));
        }
        // Space sets the step under the cursor
        focus(&mut h, grid::grid_id(true));
        h.key(Key::Space);
        let events = std::mem::take(&mut h.app.panel.events);
        assert!(
            matches!(
                events.as_slice(),
                [Out::Step808 {
                    step: 0,
                    on: true,
                    ..
                }]
            ),
            "{theme:?}: {events:?}"
        );
    }
}

/// The 303 step editor: in its section, reached by Tab, with the keys.
#[test]
fn the_303_step_editor_follows_the_skins_layout() {
    for theme in THEMES {
        let (mut h, _d) = app(theme);
        let ctls = controls(&h);
        for k in 0..2 {
            let id = step303::editor_id(k);
            let n = h.tree().by_id(id).unwrap();
            assert_eq!(a11y::role(n), Role::Slider);
            assert_eq!(a11y::name(n), Some(step303::EDITOR_NAME));
            assert_eq!(a11y::value(n), Some("Step 1 of 16: rest, C"), "{theme:?}");
            let (_, g) = h.tree().parent(id.accesskit_id()).unwrap();
            assert_eq!(a11y::name(g), Some(format!("303-{}", k + 1).as_str()));
            let c = ctls
                .iter()
                .find(|c| c.kind == "display" && step303::editor_of_layout(&c.id) == Some(k))
                .unwrap();
            assert!(near(bounds(n), (c.x, c.y, c.w, c.h)), "{theme:?} {k}");
        }
        focus(
            &mut h,
            orb_ui::a11y::hit_id(find(&ctls, "knob", "tb303_1.accent")),
        );
        h.key(Key::Tab);
        assert_eq!(
            h.tree().focused_id(),
            step303::editor_id(0).accesskit_id(),
            "{theme:?}"
        );
        h.key(Key::ArrowDown);
        let v = a11y::value(h.tree().by_id(step303::editor_id(0)).unwrap()).unwrap();
        assert_eq!(v, "Step 2 of 16: rest, C", "{theme:?}");
    }
}

/// A crossfade between the skins keeps the focus on the same control and says nothing: the
/// View menu's radio items have no announcement of their own (`Cmd::SetTheme` is their whole
/// action), so the announcer stays as it was.
#[test]
fn a_crossfade_between_the_skins_keeps_the_focus_and_is_silent() {
    for (from, to) in [
        (Theme::Modern, Theme::Classic),
        (Theme::Classic, Theme::Modern),
    ] {
        let (mut h, _d) = app(from);
        let ctls = controls(&h);
        for (kind, cid) in [("knob", "tb303_1.cutoff"), ("toggle", "transport.loop")] {
            let id = orb_ui::a11y::hit_id(find(&ctls, kind, cid));
            focus(&mut h, id);
            let serial = h.app.announcer.serial();
            let text = h.app.announcer.text().map(str::to_string);
            h.app.command(Cmd::SetTheme(to));
            // the focus and the announcer stay through every frame of the fade
            let mut frames = 0;
            while h.app.fading().is_some() || frames == 0 {
                h.frame(vec![]);
                frames += 1;
                let what = format!("{from:?}->{to:?} {cid} frame {frames}");
                assert_eq!(h.tree().focused_id(), id.accesskit_id(), "{what}");
                assert_eq!(h.app.announcer.serial(), serial, "{what}");
                assert!(frames < 200, "{what}");
            }
            assert!(frames > 3, "a fade of several frames");
            assert_eq!(h.app.announcer.text().map(str::to_string), text);
            assert_eq!(h.app.theme(), to);
            assert!(h.app.panel.events.is_empty(), "no engine event");
            // and back
            switch(&mut h, from);
            assert_eq!(
                h.tree().focused_id(),
                id.accesskit_id(),
                "{from:?}->{to:?}->back"
            );
            assert_eq!(h.app.announcer.serial(), serial);
        }
    }
}
