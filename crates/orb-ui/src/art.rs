//! The panel's SVG parts (`assets/ui`), embedded by `build.rs`, and the knob designs.

include!(concat!(env!("OUT_DIR"), "/parts_gen.rs"));

/// A part: its URI for egui's loaders, its SVG bytes and its bleed: how far, in panel pixels,
/// the art reaches beyond its control's rectangle on every side (shadows, halos, glows;
/// task-179). The SVG's canvas is the rectangle grown by the bleed.
#[derive(Clone, Copy)]
pub struct Part {
    pub name: &'static str,
    pub uri: &'static str,
    pub svg: &'static [u8],
    pub bleed: f32,
    /// The control's rectangle (panel pixels) the part is drawn for.
    pub w: f32,
    pub h: f32,
}

impl Part {
    fn from_row(row: &PartRow) -> Self {
        let &(name, uri, svg, bleed, w, h) = row;
        Self {
            name,
            uri,
            svg,
            bleed,
            w,
            h,
        }
    }
}

/// The part called `name` of the embedded set (`PARTS` is sorted by name).
pub fn part(name: &str) -> Option<Part> {
    let i = PARTS.binary_search_by(|p| p.0.cmp(name)).ok()?;
    Some(Part::from_row(&PARTS[i]))
}

/// A part set: the SVG parts one SVG skin draws with (docs/ui_spec.md section 22). The modern
/// skin's is the embedded `assets/ui` ([`PartSet::embedded`]); another set is made at run time
/// from SVG files ([`PartSet::from_svgs`]).
pub struct PartSet {
    /// Sorted by name.
    parts: Vec<Part>,
}

impl PartSet {
    /// The embedded parts of `assets/ui`.
    pub fn embedded() -> &'static std::sync::Arc<PartSet> {
        static SET: std::sync::OnceLock<std::sync::Arc<PartSet>> = std::sync::OnceLock::new();
        SET.get_or_init(|| {
            std::sync::Arc::new(PartSet {
                parts: PARTS.iter().map(Part::from_row).collect(),
            })
        })
    }

    /// A set of the SVG files `files` (part name, SVG bytes) named `set` (for the parts'
    /// URIs). Each part's rectangle and bleed are read from its `viewBox`
    /// (`-b -b w+2b h+2b`, assets/ui/README.md), as `build.rs` reads the embedded ones. The
    /// names and bytes live as long as the process (the part textures are keyed by them, as
    /// the embedded parts' are): a set is made once per skin.
    pub fn from_svgs(set: &str, files: Vec<(String, Vec<u8>)>) -> Result<PartSet, String> {
        let mut parts = Vec::with_capacity(files.len());
        for (name, svg) in files {
            let text = std::str::from_utf8(&svg).map_err(|_| format!("{name}: not UTF-8"))?;
            let vb: Vec<f32> = text
                .split("viewBox=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .ok_or_else(|| format!("{name}: no viewBox"))?
                .split([' ', ','])
                .filter(|v| !v.is_empty())
                .map(|v| v.parse::<f32>())
                .collect::<Result<_, _>>()
                .map_err(|_| format!("{name}: a malformed viewBox"))?;
            if vb.len() != 4 {
                return Err(format!("{name}: a viewBox of {} numbers", vb.len()));
            }
            let b = -vb[0];
            let uri = format!("bytes://orb-ui-{set}/{name}.svg");
            parts.push(Part {
                name: Box::leak(name.into_boxed_str()),
                uri: Box::leak(uri.into_boxed_str()),
                svg: Box::leak(svg.into_boxed_slice()),
                bleed: b,
                w: vb[2] - 2.0 * b,
                h: vb[3] - 2.0 * b,
            });
        }
        parts.sort_by(|a, b| a.name.cmp(b.name));
        Ok(PartSet { parts })
    }

    /// The part called `name`.
    pub fn part(&self, name: &str) -> Option<Part> {
        let i = self.parts.binary_search_by(|p| p.name.cmp(name)).ok()?;
        Some(self.parts[i])
    }

    /// Every part, by name.
    pub fn all(&self) -> impl Iterator<Item = Part> + '_ {
        self.parts.iter().copied()
    }
}

/// The part names asked for that no art has, in this process.
static MISSING: std::sync::Mutex<std::collections::BTreeSet<String>> =
    std::sync::Mutex::new(std::collections::BTreeSet::new());

/// Records that the part `name` is missing; true the first time it is.
pub fn report_missing(name: &str) -> bool {
    let mut m = MISSING.lock().unwrap_or_else(|e| e.into_inner());
    m.insert(name.to_owned())
}

/// The part names drawn as nothing because no art has them, in this process (task-156).
pub fn missing() -> Vec<String> {
    let m = MISSING.lock().unwrap_or_else(|e| e.into_inner());
    m.iter().cloned().collect()
}

/// Every part.
pub fn all() -> impl Iterator<Item = Part> {
    PARTS.iter().map(Part::from_row)
}

/// A knob design: the layers are `<name>_body` (static) and `<name>_pointer`, rotated about
/// `(cx, cy)` (assets/ui/README.md). The knobs have no value arc since task-179.
pub struct KnobDesign {
    pub name: &'static str,
    pub w: f32,
    pub h: f32,
    pub cx: f32,
    pub cy: f32,
}

pub const KNOBS: [KnobDesign; 5] = [
    KnobDesign {
        name: "knob_303",
        w: 30.0,
        h: 30.0,
        cx: 15.0,
        cy: 15.0,
    },
    KnobDesign {
        name: "knob_808_red",
        w: 13.0,
        h: 13.0,
        cx: 6.5,
        cy: 6.5,
    },
    KnobDesign {
        name: "knob_808_white",
        w: 13.0,
        h: 13.0,
        cx: 6.5,
        cy: 6.5,
    },
    KnobDesign {
        name: "knob_909",
        w: 12.0,
        h: 12.0,
        cx: 6.0,
        cy: 6.0,
    },
    KnobDesign {
        name: "knob_gray",
        w: 18.0,
        h: 19.0,
        cx: 9.0,
        cy: 9.5,
    },
];

/// The design of a knob control, by its layout part name.
pub fn knob_design(part: &str) -> &'static KnobDesign {
    KNOBS
        .iter()
        .find(|k| k.name == part)
        .unwrap_or_else(|| panic!("no knob design {part}"))
}

/// The window's icon: the `app_icon` part rendered at `size` x `size` pixels.
pub fn app_icon(size: u32) -> egui::IconData {
    let svg = part("app_icon").expect("app_icon part").svg;
    let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default())
        .expect("app_icon parses");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
    let scale = size as f32 / tree.size().width();
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    // IconData is straight (not premultiplied) RGBA.
    let rgba = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knob_designs_match_the_art_generator() {
        assert_eq!(KNOBS.len(), orb_ui_art::parts::KNOBS.len());
        for (a, b) in KNOBS.iter().zip(orb_ui_art::parts::KNOBS.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!((a.w, a.h, a.cx, a.cy), (b.w, b.h, b.cx, b.cy));
            // the pointer turns about the canvas's center: the art centers the knob
            assert_eq!((a.cx * 2.0, a.cy * 2.0), (a.w, a.h), "{}", a.name);
        }
    }

    #[test]
    fn all_generated_parts_are_embedded_and_sorted() {
        let generated = orb_ui_art::parts::all_parts();
        assert_eq!(PARTS.len(), generated.len());
        assert!(PARTS.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(part("btn_play_lit").is_some());
        assert!(part("panel").is_none());
        assert!(part("nope").is_none());
        // the bleed read from each file is the generator's
        for g in generated {
            let p = part(&g.name).unwrap();
            assert_eq!(p.bleed, g.bleed, "{}", g.name);
            assert!(
                (p.w - g.w).abs() < 1e-4 && (p.h - g.h).abs() < 1e-4,
                "{}",
                g.name
            );
        }
    }
}
