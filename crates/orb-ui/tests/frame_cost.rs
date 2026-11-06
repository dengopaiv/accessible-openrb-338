//! What one frame of the panel costs (task-153): the shapes `App::show` emits, the meshes and
//! vertices egui tessellates them into, the textures they bind, and the CPU time of the two, in
//! a headless context (no GPU). `cargo test --release -p orb-ui --test frame_cost -- --nocapture`
//! prints the numbers (a debug run draws 10 frames and checks only the counts' upper bounds).
//!
//! Each size is measured twice: as the window draws without a screen reader, and with AccessKit
//! on (task-186: egui builds the accessibility tree only after `Context::enable_accesskit`,
//! which the platform calls when a screen reader or another UI Automation client looks at the
//! window). The difference of the two `show` times is the cost of building the tree.

use std::collections::HashSet;
use std::time::Instant;

use eframe::egui::{self, Pos2, RawInput, Rect, Vec2, epaint};
use orb_ui::app::App;
use orb_ui::layout::{PANEL_H, PANEL_W};
use orb_ui::menu::MENU_H;
use orb_ui::model::Panel;
use orb_ui::screen_reader::ScreenReaderSetting;

/// The cost of a frame, drawn in a window of `size` points at `ppp` pixels per point.
struct Cost {
    shapes: usize,
    meshes: usize,
    vertices: usize,
    textures: usize,
    /// The nodes of the accessibility tree (0 with AccessKit off).
    nodes: usize,
    show_ms: f64,
    tess_ms: f64,
}

fn count_shapes(s: &epaint::Shape) -> usize {
    match s {
        epaint::Shape::Vec(v) => v.iter().map(count_shapes).sum(),
        _ => 1,
    }
}

fn cost(size: Vec2, ppp: f32, accesskit: bool, screen_reader: bool) -> Cost {
    let ctx = egui::Context::default();
    egui_extras::install_image_loaders(&ctx);
    ctx.set_pixels_per_point(ppp);
    if accesskit {
        ctx.enable_accesskit();
    }
    let mut app = App::new(Panel::demo(), None);
    app.menu = true;
    app.settings.screen_reader = if screen_reader {
        ScreenReaderSetting::On
    } else {
        ScreenReaderSetting::Off
    };
    const WARM: usize = 8;
    // the timing needs many frames; a debug run only checks the counts
    const RUNS: usize = if cfg!(debug_assertions) { 2 } else { 200 };
    let mut c = Cost {
        shapes: 0,
        meshes: 0,
        vertices: 0,
        textures: 0,
        nodes: 0,
        show_ms: 0.0,
        tess_ms: 0.0,
    };
    for i in 0..WARM + RUNS {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            time: Some(i as f64 / 60.0),
            max_texture_side: Some(8192),
            ..Default::default()
        };
        let t0 = Instant::now();
        let mut out = ctx.run_ui(input, |ui| app.show(ui));
        let t1 = Instant::now();
        out.textures_delta.clear();
        let nodes = out
            .platform_output
            .accesskit_update
            .as_ref()
            .map_or(0, |u| u.nodes.len());
        assert_eq!(nodes > 0, accesskit, "a tree only with AccessKit on");
        let shapes: usize = out.shapes.iter().map(|s| count_shapes(&s.shape)).sum();
        let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
        let t2 = Instant::now();
        if i < WARM {
            continue;
        }
        c.show_ms += (t1 - t0).as_secs_f64() * 1000.0 / RUNS as f64;
        c.tess_ms += (t2 - t1).as_secs_f64() * 1000.0 / RUNS as f64;
        c.shapes = shapes;
        c.nodes = nodes;
        let mut tex = HashSet::new();
        c.meshes = 0;
        c.vertices = 0;
        for p in &prims {
            if let epaint::Primitive::Mesh(m) = &p.primitive {
                c.meshes += 1;
                c.vertices += m.vertices.len();
                tex.insert(m.texture_id);
            }
        }
        c.textures = tex.len();
    }
    c
}

fn report(what: &str, size: Vec2, ppp: f32, accesskit: bool) -> Cost {
    report_mode(what, size, ppp, accesskit, false)
}

fn report_mode(what: &str, size: Vec2, ppp: f32, accesskit: bool, screen_reader: bool) -> Cost {
    let c = cost(size, ppp, accesskit, screen_reader);
    println!(
        "{what}{}{}: {} shapes, {} meshes, {} vertices, {} textures, {} nodes; \
         show {:.3} ms, tessellate {:.3} ms",
        if accesskit { " with AccessKit" } else { "" },
        if screen_reader {
            " in screen reader mode"
        } else {
            ""
        },
        c.shapes,
        c.meshes,
        c.vertices,
        c.textures,
        c.nodes,
        c.show_ms,
        c.tess_ms
    );
    c
}

#[test]
fn a_frame_of_the_panel() {
    let native_size = Vec2::new(PANEL_W, PANEL_H + MENU_H);
    let big_size = Vec2::new(1800.0, 1100.0);
    let native = report("native 1x", native_size, 1.0, false);
    let big = report("1800x1100 at 2x", big_size, 2.0, false);
    let native_ak = report("native 1x", native_size, 1.0, true);
    let big_ak = report("1800x1100 at 2x", big_size, 2.0, true);
    // screen reader mode adds the section groups and the drum grids (task-188, task-191)
    let native_sr = report_mode("native 1x", native_size, 1.0, true, true);
    for (what, off, on) in [
        ("native 1x", &native, &native_ak),
        ("1800x1100 at 2x", &big, &big_ak),
        ("native 1x, screen reader mode", &native, &native_sr),
    ] {
        println!(
            "{what}: building the accessibility tree ({} nodes) costs {:.3} ms of show",
            on.nodes,
            on.show_ms - off.show_ms
        );
    }
    // the demo panel drew 559 shapes, 524 and 534 meshes, 14112 and 16412 vertices
    // (2026-10-30): a change that makes a frame much bigger fails here. AccessKit draws
    // nothing, so the same bounds hold with it on.
    for (c, max_vertices) in [
        (&native, 17_000),
        (&big, 20_000),
        (&native_ak, 17_000),
        (&big_ak, 20_000),
        (&native_sr, 17_000),
    ] {
        assert!(c.shapes > 0 && c.meshes > 0 && c.vertices > 0);
        assert!(c.shapes <= 680, "{} shapes", c.shapes);
        assert!(c.meshes <= 650, "{} meshes", c.meshes);
        assert!(c.vertices <= max_vertices, "{} vertices", c.vertices);
    }
}
