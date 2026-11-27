//! The bundled desktop UI (milestone m-3): the song window of the original, drawn from the
//! SVG art set `assets/ui` with egui, with the original's widget behavior
//! (docs/ui_spec.md). `model` holds the state and the events the engine must hear; `link`
//! turns them into the original panel's messages and the real-time player's commands and
//! takes the player's snapshot back into the panel (task-045); `session` opens songs and the
//! built-in kit; `behavior` is the widgets' interaction math; `skin` is a look of the panel
//! (its layout and renderer; the modern one is `skin::svg::modern`, the classic one
//! `skin::classic`, drawn from the kit's bitmaps); `controller` runs all the
//! panel's input on the current skin's layout; `app` is the window.

pub mod a11y;
pub mod about;
pub mod app;
pub mod art;
pub mod audio_window;
pub mod backdrop;
pub mod behavior;
pub mod browser;
pub mod clipboard;
pub mod controller;
pub mod crash;
pub mod document;
pub mod drum_editor;
pub mod editor_frame;
pub mod embed;
pub mod error_box;
/// The standalone window's graphics adapter rule (task-217): eframe's wgpu setup, so only with
/// the `app` feature.
#[cfg(feature = "app")]
pub mod graphics;
pub mod info;
pub mod key_help;
pub mod keys;
pub mod layout;
pub mod link;
pub mod logo_anim;
pub mod menu;
pub mod mod_chooser;
pub mod mod_info;
pub mod model;
pub mod numpad;
pub mod palette;
pub mod pattern_editor;
pub mod pattern_json;
pub mod portable;
pub mod routing_window;
pub mod runlog;
pub mod screen_reader;
pub mod session;
pub mod settings;
pub mod shortcuts;
pub mod skin;
pub mod system_dialog;
pub mod template;
pub mod version;
