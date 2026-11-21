//! The activation log (task-244): with `ORB_PLUGIN_LOAD_LOG` set to a file, every activation
//! appends one line naming the plugin library that runs and the host's rate, so that a test
//! driving a real host (`tests/reaper.rs`) can tell which copy of the plugin the host loaded.
//! Off by default; a file that cannot be opened is skipped. Main thread only (`activate`).

use std::path::PathBuf;

/// The variable that names the log's file.
pub const LOAD_LOG_VAR: &str = "ORB_PLUGIN_LOAD_LOG";

/// The path of the loaded library that holds this code (the bundle's binary), from the dynamic
/// loader; `None` where it is not known (Windows: not implemented).
pub fn image_path() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        use std::ffi::{CStr, c_char, c_int, c_void};
        use std::os::unix::ffi::OsStrExt;
        #[repr(C)]
        struct DlInfo {
            fname: *const c_char,
            fbase: *mut c_void,
            sname: *const c_char,
            saddr: *mut c_void,
        }
        unsafe extern "C" {
            fn dladdr(addr: *const c_void, info: *mut DlInfo) -> c_int;
        }
        let mut info = DlInfo {
            fname: std::ptr::null(),
            fbase: std::ptr::null_mut(),
            sname: std::ptr::null(),
            saddr: std::ptr::null_mut(),
        };
        let addr = image_path as fn() -> Option<PathBuf> as *const c_void;
        if unsafe { dladdr(addr, &mut info) } == 0 || info.fname.is_null() {
            return None;
        }
        let name = unsafe { CStr::from_ptr(info.fname) };
        Some(PathBuf::from(std::ffi::OsStr::from_bytes(name.to_bytes())))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// Appends `<library path>\t<rate>` to the file `ORB_PLUGIN_LOAD_LOG` names, if it is set; the
/// path is `?` when it is not known.
pub fn log_activation(rate: u32) {
    use std::io::Write;
    let Some(path) = std::env::var_os(LOAD_LOG_VAR).filter(|p| !p.is_empty()) else {
        return;
    };
    let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    let image = image_path().map_or_else(|| "?".to_string(), |p| p.display().to_string());
    let _ = writeln!(f, "{image}\t{rate}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_image_is_this_library() {
        if cfg!(unix) {
            // in a unit test the code is in the test binary itself
            let p = super::image_path().expect("dladdr finds the image");
            let exe = std::env::current_exe().unwrap();
            assert_eq!(
                p.canonicalize().unwrap(),
                exe.canonicalize().unwrap(),
                "{p:?}"
            );
        }
    }
}
