//! A screen reader's Increment request reaches the app (task-183, AC 2): a small test widget,
//! a painted rectangle made into a slider, reads it with `accesskit_action_requests` and the
//! new value shows in the next tree.

#[path = "common/a11y.rs"]
mod a11y;

use a11y::Frames;
use eframe::egui::accesskit::{Action, ActionData, Role};
use eframe::egui::{self, Id, Pos2, Rect, Sense, Vec2, WidgetInfo};

/// The widget: its value changes only through an accessibility request.
fn slider(ui: &mut egui::Ui, id: Id, value: &mut f64, seen: &mut Vec<Action>) {
    let r = ui.interact(
        Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(20.0)),
        id,
        Sense::click_and_drag(),
    );
    ui.input(|i| {
        for (action, delta) in [(Action::Increment, 1.0), (Action::Decrement, -1.0)] {
            for _ in i.accesskit_action_requests(id, action) {
                *value += delta;
                seen.push(action);
            }
        }
        for req in i.accesskit_action_requests(id, Action::SetValue) {
            if let Some(ActionData::NumericValue(v)) = req.data {
                *value = v;
                seen.push(Action::SetValue);
            }
        }
    });
    let shown = *value;
    r.widget_info(|| WidgetInfo::labeled(Role::Slider, true, "Cutoff"));
    ui.ctx().accesskit_node_builder(id, |b| {
        b.set_numeric_value(shown);
        b.set_min_numeric_value(0.0);
        b.set_max_numeric_value(127.0);
        b.set_numeric_value_step(1.0);
        b.set_value(format!("{shown:.0} of 127"));
        b.add_action(Action::Increment);
        b.add_action(Action::Decrement);
        b.add_action(Action::SetValue);
    });
}

#[test]
fn increment_decrement_and_set_value_arrive_in_the_app() {
    let id = Id::unique("test.cutoff");
    let mut f = Frames::new(Vec2::new(100.0, 100.0));
    let (mut value, mut seen) = (64.0_f64, vec![]);
    let run = |f: &mut Frames, ev, value: &mut f64, seen: &mut Vec<Action>| {
        f.run(ev, |ui| slider(ui, id, value, seen));
    };
    run(&mut f, vec![], &mut value, &mut seen);
    {
        let n = f
            .tree
            .as_ref()
            .unwrap()
            .by_id(id)
            .expect("the widget has a node");
        assert_eq!(a11y::role(n), Role::Slider);
        assert_eq!(a11y::name(n), Some("Cutoff"));
        assert!(a11y::supports(n, Action::Increment));
        assert_eq!(a11y::numeric_value(n), Some(64.0));
    }

    run(
        &mut f,
        vec![a11y::request(id, Action::Increment, None)],
        &mut value,
        &mut seen,
    );
    assert_eq!(value, 65.0);
    assert_eq!(seen, [Action::Increment]);
    run(&mut f, vec![], &mut value, &mut seen);
    let n = f.tree.as_ref().unwrap().by_id(id).unwrap();
    assert_eq!(a11y::numeric_value(n), Some(65.0));
    assert_eq!(a11y::value(n), Some("65 of 127"));

    run(
        &mut f,
        vec![a11y::request(id, Action::Decrement, None)],
        &mut value,
        &mut seen,
    );
    assert_eq!(value, 64.0);
    run(
        &mut f,
        vec![a11y::set_value_request(id, 100.0)],
        &mut value,
        &mut seen,
    );
    assert_eq!(value, 100.0);

    // a request for another node does not reach it
    let other = Id::unique("test.other");
    run(
        &mut f,
        vec![a11y::request(other, Action::Increment, None)],
        &mut value,
        &mut seen,
    );
    assert_eq!(value, 100.0);
}
