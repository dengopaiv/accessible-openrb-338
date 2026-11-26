//! A texture freed in a frame that drew it (task-292). egui-baseview 0.9.0's wgpu renderer
//! freed egui's `textures_delta.free` before `queue.submit`; egui-wgpu's `free_texture`
//! destroys the texture at once, so the submitted frame drew a destroyed texture, a wgpu
//! validation error that aborted the host (the mods folder changed in the plugin drops the mod
//! window's splash picture in a frame that drew it). The workspace takes egui-baseview from a
//! fork that frees after the submit (the workspace `Cargo.toml`, docs/plugin.md 10).
//!
//! egui-baseview's renderer needs a window and its surface, which a test cannot have, so the
//! check is in two parts: the frame of egui-baseview's renderer replayed on a headless wgpu
//! device in both orders (the old one fails validation, the fork's does not; skipped when the
//! machine has no wgpu adapter), and the order in the source of the egui-baseview that cargo
//! resolves for this workspace (fails on the crates.io 0.9.0, passes on the fork).

use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::process::Command;
use std::task::{Context, Poll, Waker};

use egui_wgpu::wgpu;

/// wgpu's native futures are ready when first polled or after a device poll; a busy poll is
/// enough here.
fn block_on<F: Future>(f: F) -> F::Output {
    let mut f = pin!(f);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
        std::thread::yield_now();
    }
}

/// One egui frame that draws a texture and frees it, rendered as egui-baseview's renderer
/// renders a frame (`src/renderer/wgpu/renderer.rs` `render`), to an offscreen texture instead
/// of the surface's. The textures are freed before the submit (`free_before_submit`, 0.9.0) or
/// after it (the fork). Returns the validation error of the frame, or `None` with no wgpu
/// adapter on this machine (`Some(None)`: the frame was valid).
fn frame_that_frees_a_drawn_texture(free_before_submit: bool) -> Option<Option<String>> {
    #[allow(unused_mut)]
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    // Windows: DirectX 12, then GL, without Vulkan, as the fork's app (its task-217): an Intel
    // Vulkan driver crashed this test's process (STATUS_ACCESS_VIOLATION) on the fork's machine
    #[cfg(windows)]
    {
        desc.backends = wgpu::Backends::DX12 | wgpu::Backends::GL;
    }
    let instance = wgpu::Instance::new(desc);
    let Ok(adapter) = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        eprintln!("skipped: no wgpu adapter on this machine");
        return None;
    };
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("requesting the wgpu device");
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut renderer = egui_wgpu::Renderer::new(&device, format, Default::default());

    // The frame: the texture is drawn, then its last handle is dropped (as `App::refresh_mods`
    // drops the splash picture), so the frame's output both draws it and frees it.
    let ctx = egui::Context::default();
    let mut handle = Some(ctx.load_texture(
        "drawn and freed",
        egui::ColorImage::filled([4, 4], egui::Color32::WHITE),
        egui::TextureOptions::default(),
    ));
    let raw = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(64.0, 64.0),
        )),
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw, |ui| {
        if let Some(h) = handle.take() {
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            let rect = egui::Rect::from_min_size(egui::pos2(8.0, 8.0), egui::vec2(32.0, 32.0));
            ui.painter().image(h.id(), rect, uv, egui::Color32::WHITE);
        }
    });
    assert!(
        !out.textures_delta.free.is_empty(),
        "the frame frees no texture"
    );
    let prims = ctx.tessellate(out.shapes, out.pixels_per_point);
    let freed = out.textures_delta.free.clone();
    assert!(
        prims.iter().any(|p| match &p.primitive {
            egui::epaint::Primitive::Mesh(m) => freed.contains(&m.texture_id),
            _ => false,
        }),
        "the frame does not draw the freed texture"
    );

    let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let screen = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [64, 64],
        pixels_per_point: 1.0,
    };
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    for (id, deltas) in &out.textures_delta.set {
        for delta in deltas {
            renderer.update_texture(&device, &queue, *id, delta);
        }
    }
    let user_cmd_bufs = renderer.update_buffers(&device, &queue, &mut encoder, &prims, &screen);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("frame"),
        size: wgpu::Extent3d {
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());
    {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui_render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer.render(&mut pass.forget_lifetime(), &prims, &screen);
    }
    let free = |renderer: &mut egui_wgpu::Renderer| {
        for id in &freed {
            renderer.free_texture(id);
        }
    };
    if free_before_submit {
        free(&mut renderer);
    }
    queue.submit(user_cmd_bufs.into_iter().chain([encoder.finish()]));
    if !free_before_submit {
        free(&mut renderer);
    }
    // Applied above (epaint asserts that a frame's deltas are handled).
    out.textures_delta.clear();
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    Some(block_on(scope.pop()).map(|e| e.to_string()))
}

#[test]
fn freeing_before_the_submit_fails_validation() {
    let Some(error) = frame_that_frees_a_drawn_texture(true) else {
        return;
    };
    let error = error.expect("the 0.9.0 order passed validation");
    assert!(error.contains("destroyed"), "unexpected error: {error}");
}

#[test]
fn freeing_after_the_submit_is_valid() {
    let Some(error) = frame_that_frees_a_drawn_texture(false) else {
        return;
    };
    assert_eq!(error, None);
}

/// The host triple, from `cargo -vV`.
fn host_triple() -> String {
    let out = Command::new(env!("CARGO"))
        .arg("-vV")
        .output()
        .expect("running cargo -vV");
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .find_map(|l| l.strip_prefix("host: "))
        .expect("cargo -vV names the host")
        .trim()
        .to_string()
}

/// The source of the egui-baseview this workspace resolves (`cargo metadata`, offline). The
/// metadata is filtered to the host's platform: the packages of other targets (the Windows-only
/// ASIO build's, task-193) are not downloaded on a machine that only builds for itself.
fn resolved_egui_baseview_renderer() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let out = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--offline",
            "--format-version",
            "1",
            "--filter-platform",
        ])
        .arg(host_triple())
        .arg("--manifest-path")
        .arg(&root)
        .output()
        .expect("running cargo metadata");
    assert!(
        out.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).expect("metadata JSON");
    let packages = meta["packages"].as_array().expect("packages");
    let found: Vec<&serde_json::Value> = packages
        .iter()
        .filter(|p| p["name"] == "egui-baseview")
        .collect();
    assert_eq!(
        found.len(),
        1,
        "egui-baseview resolves to {} packages",
        found.len()
    );
    let manifest = Path::new(found[0]["manifest_path"].as_str().expect("manifest_path"));
    manifest
        .parent()
        .unwrap()
        .join("src/renderer/wgpu/renderer.rs")
}

#[test]
fn the_resolved_egui_baseview_frees_textures_after_the_submit() {
    let path = resolved_egui_baseview_renderer();
    let src = std::fs::read_to_string(&path).expect("reading egui-baseview's renderer");
    let at = |what: &str| {
        let i: Vec<usize> = src.match_indices(what).map(|(i, _)| i).collect();
        assert_eq!(
            i.len(),
            1,
            "{what} occurs {} times in {}",
            i.len(),
            path.display()
        );
        i[0]
    };
    let submit = at(".submit(");
    let free = at("free_texture(");
    let present = at(".present(");
    assert!(
        submit < free && free < present,
        "{} frees the textures before the submit or presents before freeing them \
         (submit at {submit}, free_texture at {free}, present at {present})",
        path.display()
    );
}
