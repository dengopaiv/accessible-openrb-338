//! Portable mode on Windows (task-289, GitHub issue 19): a file `OpenRB-338.toml` in the folder
//! of the program keeps the settings, the crash logs and the plugin's log next to it instead of
//! in the user's profile.
//!
//! - The standalone app's folder is the one of its `OpenRB-338.exe`.
//! - The plugin's folder is the one that holds the single file `OpenRB-338.vst3` (the zip), or the
//!   `OpenRB-338.vst3` bundle (the installer; never inside it). Both are found the same way: the
//!   folder of the module (exe or dll) that holds this code, from `GetModuleHandleExW` with the
//!   address of a function here and `GetModuleFileNameW`; the plugin links this crate in, so its
//!   module is the plugin's binary `OpenRB-338.vst3\Contents\x86_64-win\OpenRB-338.vst3`
//!   ([`folder_of`] goes up to the bundle's parent) or the single file's own folder.
//! - [`folder`] is that folder when the file is in it. macOS and Linux never have one.
//!
//! The rules for what the file holds are in `settings.rs`.

use std::path::{Path, PathBuf};

/// The settings file's name, in the per-user folder and in the portable one.
pub const FILE_NAME: &str = "OpenRB-338.toml";

/// The folder of the program for the path of its module `module` (the exe, or the plugin's
/// binary): the exe's folder, or for `X.vst3\Contents\<arch>\X.vst3` the folder that holds the
/// bundle `X.vst3`.
pub fn folder_of(module: &Path) -> Option<PathBuf> {
    let dir = module.parent()?;
    let is_vst3 = |p: &Path| {
        p.extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("vst3"))
    };
    if is_vst3(module)
        && let Some(contents) = dir.parent()
        && contents
            .file_name()
            .is_some_and(|n| n.eq_ignore_ascii_case("Contents"))
        && let Some(bundle) = contents.parent()
        && is_vst3(bundle)
    {
        return bundle.parent().map(Path::to_path_buf);
    }
    Some(dir.to_path_buf())
}

/// The folder that would hold the portable file (the program's), or `None` off Windows.
pub fn candidate() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        static CANDIDATE: std::sync::OnceLock<Option<PathBuf>> = std::sync::OnceLock::new();
        CANDIDATE
            .get_or_init(|| module_path().and_then(|m| folder_of(&m)))
            .clone()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// The portable folder: the program's folder when `OpenRB-338.toml` is in it (Windows only).
pub fn folder() -> Option<PathBuf> {
    candidate().filter(|d| d.join(FILE_NAME).is_file())
}

/// The path of the module (exe or dll) that holds this function.
#[cfg(windows)]
fn module_path() -> Option<PathBuf> {
    use std::ffi::{OsString, c_void};
    use std::os::windows::ffi::OsStringExt;

    const FROM_ADDRESS: u32 = 0x4;
    const UNCHANGED_REFCOUNT: u32 = 0x2;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleExW(flags: u32, address: *const c_void, module: *mut *mut c_void) -> i32;
        fn GetModuleFileNameW(module: *mut c_void, name: *mut u16, size: u32) -> u32;
    }
    let mut module = std::ptr::null_mut();
    // SAFETY: the address is of a function of this module; the handle is not retained
    // (UNCHANGED_REFCOUNT), the out pointer is a local.
    let ok = unsafe {
        GetModuleHandleExW(
            FROM_ADDRESS | UNCHANGED_REFCOUNT,
            module_path as *const c_void,
            &mut module,
        )
    };
    if ok == 0 {
        return None;
    }
    let mut size = 512usize;
    while size <= 32768 {
        let mut buf = vec![0u16; size];
        // SAFETY: `buf` holds `size` units.
        let n = unsafe { GetModuleFileNameW(module, buf.as_mut_ptr(), size as u32) } as usize;
        if n == 0 {
            return None;
        }
        if n < size {
            return Some(PathBuf::from(OsString::from_wide(&buf[..n])));
        }
        size *= 2;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_exe_folder_is_the_exes() {
        assert_eq!(
            folder_of(Path::new("/z/app/OpenRB-338.exe")),
            Some(PathBuf::from("/z/app"))
        );
    }

    #[test]
    fn the_plugins_folder_is_the_one_that_holds_the_bundle() {
        let m = Path::new("/z/vst/OpenRB-338.vst3/Contents/x86_64-win/OpenRB-338.vst3");
        assert_eq!(folder_of(m), Some(PathBuf::from("/z/vst")));
        let m = Path::new("/z/vst/openrb-338.VST3/contents/x86_64-win/openrb-338.VST3");
        assert_eq!(folder_of(m), Some(PathBuf::from("/z/vst")));
    }

    #[test]
    fn a_single_file_vst3_is_in_its_folder() {
        let m = Path::new("/z/vst/OpenRB-338.vst3");
        assert_eq!(folder_of(m), Some(PathBuf::from("/z/vst")));
    }

    #[test]
    fn there_is_no_portable_folder_off_windows() {
        if !cfg!(windows) {
            assert_eq!(candidate(), None);
            assert_eq!(folder(), None);
        }
    }
}
