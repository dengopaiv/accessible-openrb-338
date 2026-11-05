//! Screen reader mode (A11Y-1, task-185): the setting `screen_reader` of `ui.toml` and the
//! system's answer to "is a screen reader running".
//!
//! The setting has three values: `auto` (the default: the mode is on while the system reports a
//! screen reader), `on` and `off`. [`ScreenReaderSetting::active`] combines the setting with
//! the system's answer; `App::screen_reader_mode` is the one place later tasks read.

/// The `screen_reader` setting of the settings file and the Options menu.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenReaderSetting {
    /// On while the system reports a screen reader.
    #[default]
    Auto,
    /// Always on.
    On,
    /// Always off: the program behaves as without the mode.
    Off,
}

impl ScreenReaderSetting {
    /// The three values, in the menu's order.
    pub const ALL: [Self; 3] = [Self::Auto, Self::On, Self::Off];

    /// The value as written in `ui.toml`.
    pub fn key(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Off => "off",
        }
    }

    /// The value written `key` (ASCII case ignored); `None` for anything else.
    pub fn from_key(s: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|v| v.key().eq_ignore_ascii_case(s))
    }

    /// The menu text of the choice.
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Automatic",
            Self::On => "On",
            Self::Off => "Off",
        }
    }

    /// Whether the mode is on, given whether the system reports a screen reader.
    pub fn active(self, system_reports: bool) -> bool {
        match self {
            Self::Auto => system_reports,
            Self::On => true,
            Self::Off => false,
        }
    }
}

/// Whether the system reports a running screen reader. Windows: the screen reader flag,
/// `SystemParametersInfoW(SPI_GETSCREENREADER)`, which screen readers set while they run.
/// Other platforms report false (VoiceOver and AT-SPI detection are a later task).
pub fn system_reports_screen_reader() -> bool {
    imp::system_reports()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of this machine, printed for the hand check of task-185:
    /// `cargo test -p orb-ui --lib screen_reader -- --nocapture`.
    #[test]
    fn the_system_answer_can_be_asked() {
        println!(
            "system_reports_screen_reader = {}",
            system_reports_screen_reader()
        );
    }

    #[test]
    fn the_keys_round_trip() {
        for v in ScreenReaderSetting::ALL {
            assert_eq!(ScreenReaderSetting::from_key(v.key()), Some(v));
        }
        assert_eq!(ScreenReaderSetting::from_key("x"), None);
    }
}

#[cfg(windows)]
mod imp {
    pub fn system_reports() -> bool {
        const SPI_GETSCREENREADER: u32 = 0x0046;
        unsafe extern "system" {
            fn SystemParametersInfoW(
                action: u32,
                param: u32,
                pv_param: *mut core::ffi::c_void,
                win_ini: u32,
            ) -> i32;
        }
        let mut flag: i32 = 0;
        // SAFETY: SPI_GETSCREENREADER writes one BOOL (an i32) through `pv_param`.
        let ok = unsafe {
            SystemParametersInfoW(SPI_GETSCREENREADER, 0, (&mut flag as *mut i32).cast(), 0)
        };
        ok != 0 && flag != 0
    }
}

#[cfg(not(windows))]
mod imp {
    pub fn system_reports() -> bool {
        false
    }
}
