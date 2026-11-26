//! The keyboard shortcuts of the menu and the browser (task-092): one definition each, used
//! both to catch the key ([`consume`]) and to write the text next to a menu item ([`text`]), so
//! the two cannot drift. The command modifier is Cmd on macOS and Ctrl elsewhere (egui's
//! `Modifiers::COMMAND`).

use egui::{Context, Key, KeyboardShortcut, ModifierNames, Modifiers, Ui};

pub const NEW: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::N);
pub const OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
pub const SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
/// File > Info.
pub const INFO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
/// Edit > Undo.
pub const UNDO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
/// Edit > Cut, Copy and Paste Pattern, for their text only: the keys arrive as egui's Cut, Copy
/// and Paste events (docs/ui_spec.md section 16).
pub const CUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::X);
pub const COPY: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::C);
pub const PASTE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::V);
/// Edit > Duplicate Pattern.
pub const DUPLICATE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::D);
/// Ctrl/Cmd+E: the focused row's pattern editor, Edit > 303-1, 303-2, 808 or 909 Pattern
/// Editor... (tasks 274, 280, 281 and 290).
pub const PATTERN_EDITOR: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::E);
/// "Program Synth from Keyboard".
pub const KEYS_SYNTH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::F);
/// "Select Patterns from Keyboard".
pub const KEYS_PATTERNS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::G);
/// "Choose Mod...", screen reader mode only (task-195).
pub const CHOOSE_MOD: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::M);
/// In the mod chooser: Mod Info of the selected entry (task-195). The same key as [`INFO`]
/// (File > Info): while the chooser is open it takes the key first, elsewhere File > Info does.
pub const CHOOSER_INFO: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::I);
/// The file browser's path field.
pub const EDIT_PATH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::L);

/// The text of a shortcut for a platform: "Cmd+N" on macOS, "Ctrl+N" elsewhere (the macOS
/// symbol form is not in egui's default font).
pub fn text_for(sc: &KeyboardShortcut, is_mac: bool) -> String {
    sc.format(&ModifierNames::NAMES, is_mac)
}

/// The text of a shortcut on the running platform.
pub fn text(ctx: &Context, sc: &KeyboardShortcut) -> String {
    text_for(sc, ctx.os().is_mac())
}

/// Takes the shortcut's key press from the input, true if it was pressed.
pub fn consume(ui: &Ui, sc: &KeyboardShortcut) -> bool {
    ui.input_mut(|i| i.consume_shortcut(sc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_names_cmd_on_mac_and_ctrl_elsewhere() {
        let all = [
            NEW,
            OPEN,
            SAVE,
            INFO,
            UNDO,
            CUT,
            COPY,
            PASTE,
            DUPLICATE,
            PATTERN_EDITOR,
            KEYS_SYNTH,
            KEYS_PATTERNS,
            CHOOSE_MOD,
            CHOOSER_INFO,
            EDIT_PATH,
        ];
        let keys = [
            "N", "O", "S", "I", "Z", "X", "C", "V", "D", "E", "F", "G", "M", "I", "L",
        ];
        for (sc, k) in all.iter().zip(keys) {
            assert_eq!(text_for(sc, true), format!("Cmd+{k}"));
            assert_eq!(text_for(sc, false), format!("Ctrl+{k}"));
        }
    }
}
