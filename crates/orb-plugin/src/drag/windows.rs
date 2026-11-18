//! The drag on Windows: OLE's `DoDragDrop` with a data object that holds the clip's file as
//! `CF_HDROP` (as a file dragged from Explorer) and a drop source that drops on the left button's
//! release and cancels on Escape.
//!
//! - The drag is begun from a thread timer of 0 ms (`SetTimer` without a window, as the resize
//!   corner's request in `editor.rs`): `DoDragDrop` runs a modal loop that dispatches the window's
//!   messages, which must not happen inside the egui frame. Nothing is begun when the button is up
//!   already or the window is gone.
//! - The two COM objects are made by hand ([`DataObject`], [`DropSource`]): vtables of `extern
//!   "system"` functions, reference counts, freed at the last `Release`. `EnumFormatEtc` is the
//!   shell's (`SHCreateStdEnumFmtEtc`); `SetData`, the advise sink and `GetDataHere` are not
//!   supported.
//! - `DoDragDrop` takes the button's release: after it the window is posted a `WM_LBUTTONUP` at the
//!   pointer, so that egui sees it.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use crate::editor::guard;

type Hresult = i32;
type Hwnd = *mut c_void;

/// A COM interface id.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Guid {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

const IID_IUNKNOWN: Guid = Guid {
    d1: 0,
    d2: 0,
    d3: 0,
    d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
};
const IID_IDATAOBJECT: Guid = Guid {
    d1: 0x0000_010E,
    d2: 0,
    d3: 0,
    d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
};
const IID_IDROPSOURCE: Guid = Guid {
    d1: 0x0000_0121,
    d2: 0,
    d3: 0,
    d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46],
};

/// `FORMATETC`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FormatEtc {
    pub cf: u16,
    pub ptd: *mut c_void,
    pub aspect: u32,
    pub lindex: i32,
    pub tymed: u32,
}

/// `STGMEDIUM` (its union is a handle or a pointer).
#[repr(C)]
#[derive(Debug)]
pub struct StgMedium {
    pub tymed: u32,
    pub data: *mut c_void,
    pub release: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

/// `DROPFILES`, followed by the file names (wide, each NUL-terminated, then a NUL).
#[repr(C)]
struct DropFiles {
    files: u32,
    pt: Point,
    nc: i32,
    wide: i32,
}

pub const S_OK: Hresult = 0;
const E_NOTIMPL: Hresult = 0x8000_4001_u32 as i32;
const E_NOINTERFACE: Hresult = 0x8000_4002_u32 as i32;
const E_POINTER: Hresult = 0x8000_4003_u32 as i32;
const E_OUTOFMEMORY: Hresult = 0x8007_000E_u32 as i32;
pub const DV_E_FORMATETC: Hresult = 0x8004_0064_u32 as i32;
const OLE_E_ADVISENOTSUPPORTED: Hresult = 0x8004_0003_u32 as i32;
const DATA_S_SAMEFORMATETC: Hresult = 0x0004_0130;
pub const DRAGDROP_S_DROP: Hresult = 0x0004_0100;
pub const DRAGDROP_S_CANCEL: Hresult = 0x0004_0101;
const DRAGDROP_S_USEDEFAULTCURSORS: Hresult = 0x0004_0102;

pub const CF_HDROP: u16 = 15;
pub const TYMED_HGLOBAL: u32 = 1;
pub const DVASPECT_CONTENT: u32 = 1;
const DATADIR_GET: u32 = 1;
const DROPEFFECT_COPY: u32 = 1;
pub const MK_LBUTTON: u32 = 1;
pub const MK_RBUTTON: u32 = 2;
const GMEM_MOVEABLE: u32 = 0x0002;
const GMEM_ZEROINIT: u32 = 0x0040;
const WM_LBUTTONUP: u32 = 0x0202;
const VK_LBUTTON: i32 = 1;
const VK_RBUTTON: i32 = 2;
const SM_SWAPBUTTON: i32 = 23;

type TimerProc = unsafe extern "system" fn(Hwnd, u32, usize, u32);

#[link(name = "ole32")]
unsafe extern "system" {
    fn OleInitialize(reserved: *mut c_void) -> Hresult;
    fn OleUninitialize();
    fn DoDragDrop(data: *mut c_void, source: *mut c_void, ok: u32, effect: *mut u32) -> Hresult;
    #[cfg(test)]
    pub fn ReleaseStgMedium(medium: *mut StgMedium);
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> *mut c_void;
    pub fn GlobalLock(h: *mut c_void) -> *mut c_void;
    pub fn GlobalUnlock(h: *mut c_void) -> i32;
    fn GlobalFree(h: *mut c_void) -> *mut c_void;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn SHCreateStdEnumFmtEtc(n: u32, formats: *const FormatEtc, out: *mut *mut c_void) -> Hresult;
    #[cfg(test)]
    pub fn DragQueryFileW(drop: *mut c_void, i: u32, file: *mut u16, len: u32) -> u32;
}

#[link(name = "user32")]
unsafe extern "system" {
    fn SetTimer(hwnd: Hwnd, id: usize, ms: u32, proc_: Option<TimerProc>) -> usize;
    fn KillTimer(hwnd: Hwnd, id: usize) -> i32;
    fn PostMessageW(hwnd: Hwnd, msg: u32, w: usize, l: isize) -> i32;
    fn GetCursorPos(p: *mut Point) -> i32;
    fn ScreenToClient(hwnd: Hwnd, p: *mut Point) -> i32;
    fn GetAsyncKeyState(key: i32) -> i16;
    fn GetSystemMetrics(index: i32) -> i32;
    fn IsWindow(hwnd: Hwnd) -> i32;
}

/// The one format offered.
pub fn hdrop_format() -> FormatEtc {
    FormatEtc {
        cf: CF_HDROP,
        ptd: std::ptr::null_mut(),
        aspect: DVASPECT_CONTENT,
        lindex: -1,
        tymed: TYMED_HGLOBAL,
    }
}

/// A drag posted to this thread.
struct Job {
    hwnd: Hwnd,
    path: PathBuf,
}

thread_local! {
    /// The drag of each pending timer of this thread.
    static JOBS: RefCell<BTreeMap<usize, Job>> = const { RefCell::new(BTreeMap::new()) };
}

/// Posts the drag of `path` from the window `hwnd` to this thread's message loop.
pub fn start(hwnd: Hwnd, path: PathBuf) -> bool {
    unsafe extern "system" fn tick(_: Hwnd, _: u32, timer: usize, _: u32) {
        // SAFETY: a timer of this thread, without a window.
        unsafe { KillTimer(std::ptr::null_mut(), timer) };
        if let Some(job) = JOBS.with(|j| j.borrow_mut().remove(&timer)) {
            let _ = guard("drag", || run(&job));
        }
    }
    // SAFETY: a thread timer with a callback of the right signature.
    let timer = unsafe { SetTimer(std::ptr::null_mut(), 0, 0, Some(tick)) };
    if timer == 0 {
        return false;
    }
    JOBS.with(|j| j.borrow_mut().insert(timer, Job { hwnd, path }));
    true
}

/// Kills this thread's pending drags (the editor closes).
pub fn cancel() {
    let timers: Vec<usize> = JOBS
        .try_with(|j| std::mem::take(&mut *j.borrow_mut()).into_keys().collect())
        .unwrap_or_default();
    for t in timers {
        // SAFETY: a timer of this thread, without a window.
        unsafe { KillTimer(std::ptr::null_mut(), t) };
    }
}

/// The left button as the user sees it (the right one when the buttons are swapped), its
/// virtual key and its `MK_` flag.
fn left_button() -> (i32, u32) {
    // SAFETY: no pointers.
    if unsafe { GetSystemMetrics(SM_SWAPBUTTON) } != 0 {
        (VK_RBUTTON, MK_RBUTTON)
    } else {
        (VK_LBUTTON, MK_LBUTTON)
    }
}

/// The drag, from the timer.
fn run(job: &Job) {
    let (vk, _) = left_button();
    // SAFETY: plain calls; `hwnd` is only tested and posted to.
    unsafe {
        if IsWindow(job.hwnd) == 0 || GetAsyncKeyState(vk) >= 0 {
            // the window is gone or the button is up already
            return;
        }
        let init = OleInitialize(std::ptr::null_mut()) >= 0;
        if !init {
            // a thread of the multithreaded apartment cannot drag
            return;
        }
        let data = DataObject::new(&job.path);
        let source = DropSource::new();
        let mut effect = 0u32;
        DoDragDrop(data.cast(), source.cast(), DROPEFFECT_COPY, &mut effect);
        data_release(data);
        source_release(source);
        OleUninitialize();
        // the window did not get the button's release
        let mut p = Point::default();
        if GetCursorPos(&mut p) != 0 && ScreenToClient(job.hwnd, &mut p) != 0 {
            let l = ((p.y as u16 as u32) << 16 | p.x as u16 as u32) as isize;
            PostMessageW(job.hwnd, WM_LBUTTONUP, 0, l);
        }
    }
}

/// `IDataObject` with one file as `CF_HDROP`.
#[repr(C)]
pub struct DataObject {
    vtbl: *const DataVtbl,
    refs: AtomicU32,
    /// The path, wide, NUL-terminated.
    path: Vec<u16>,
}

/// `IDataObject`'s vtable; COM reads its fields through the object's pointer.
#[repr(C)]
#[allow(dead_code)]
pub struct DataVtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut DataObject, *const Guid, *mut *mut c_void) -> Hresult,
    pub add_ref: unsafe extern "system" fn(*mut DataObject) -> u32,
    pub release: unsafe extern "system" fn(*mut DataObject) -> u32,
    pub get_data:
        unsafe extern "system" fn(*mut DataObject, *const FormatEtc, *mut StgMedium) -> Hresult,
    pub get_data_here:
        unsafe extern "system" fn(*mut DataObject, *const FormatEtc, *mut StgMedium) -> Hresult,
    pub query_get_data: unsafe extern "system" fn(*mut DataObject, *const FormatEtc) -> Hresult,
    pub get_canonical_format_etc:
        unsafe extern "system" fn(*mut DataObject, *const FormatEtc, *mut FormatEtc) -> Hresult,
    pub set_data: unsafe extern "system" fn(
        *mut DataObject,
        *const FormatEtc,
        *mut StgMedium,
        i32,
    ) -> Hresult,
    pub enum_format_etc:
        unsafe extern "system" fn(*mut DataObject, u32, *mut *mut c_void) -> Hresult,
    pub d_advise: unsafe extern "system" fn(
        *mut DataObject,
        *const FormatEtc,
        u32,
        *mut c_void,
        *mut u32,
    ) -> Hresult,
    pub d_unadvise: unsafe extern "system" fn(*mut DataObject, u32) -> Hresult,
    pub enum_d_advise: unsafe extern "system" fn(*mut DataObject, *mut *mut c_void) -> Hresult,
}

static DATA_VTBL: DataVtbl = DataVtbl {
    query_interface: data_query_interface,
    add_ref: data_add_ref,
    release: data_release,
    get_data,
    get_data_here,
    query_get_data,
    get_canonical_format_etc,
    set_data,
    enum_format_etc,
    d_advise,
    d_unadvise,
    enum_d_advise,
};

impl DataObject {
    /// A new data object of the file `path`, with one reference.
    pub fn new(path: &Path) -> *mut DataObject {
        let path = path.as_os_str().encode_wide().chain([0]).collect();
        Box::into_raw(Box::new(DataObject {
            vtbl: &DATA_VTBL,
            refs: AtomicU32::new(1),
            path,
        }))
    }

    /// The object's vtable (tests call through it as a host does).
    ///
    /// # Safety
    /// `this` is a live data object.
    #[cfg(test)]
    pub unsafe fn vtbl(this: *mut DataObject) -> &'static DataVtbl {
        unsafe { &*(*this).vtbl }
    }
}

/// Whether `f` asks for what the object has: `CF_HDROP` in global memory.
///
/// # Safety
/// `f` is null or a valid `FORMATETC`.
unsafe fn offered(f: *const FormatEtc) -> Result<(), Hresult> {
    let Some(f) = (unsafe { f.as_ref() }) else {
        return Err(E_POINTER);
    };
    if f.cf == CF_HDROP && f.tymed & TYMED_HGLOBAL != 0 && f.aspect == DVASPECT_CONTENT {
        Ok(())
    } else {
        Err(DV_E_FORMATETC)
    }
}

unsafe extern "system" fn data_query_interface(
    this: *mut DataObject,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> Hresult {
    if out.is_null() || iid.is_null() {
        return E_POINTER;
    }
    // SAFETY: valid pointers from the caller.
    unsafe {
        if *iid == IID_IUNKNOWN || *iid == IID_IDATAOBJECT {
            data_add_ref(this);
            *out = this.cast();
            S_OK
        } else {
            *out = std::ptr::null_mut();
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn data_add_ref(this: *mut DataObject) -> u32 {
    // SAFETY: a live object.
    unsafe { (*this).refs.fetch_add(1, Ordering::AcqRel) + 1 }
}

unsafe extern "system" fn data_release(this: *mut DataObject) -> u32 {
    // SAFETY: a live object; the last reference frees it.
    unsafe {
        let n = (*this).refs.fetch_sub(1, Ordering::AcqRel) - 1;
        if n == 0 {
            drop(Box::from_raw(this));
        }
        n
    }
}

unsafe extern "system" fn get_data(
    this: *mut DataObject,
    f: *const FormatEtc,
    medium: *mut StgMedium,
) -> Hresult {
    // SAFETY: valid pointers from the caller; the memory is allocated and filled within its size.
    unsafe {
        if let Err(e) = offered(f) {
            return e;
        }
        if medium.is_null() {
            return E_POINTER;
        }
        let path = &(*this).path;
        let head = std::mem::size_of::<DropFiles>();
        // the name and its NUL, then the list's NUL
        let size = head + (path.len() + 1) * 2;
        let h = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, size);
        if h.is_null() {
            return E_OUTOFMEMORY;
        }
        let p = GlobalLock(h).cast::<u8>();
        if p.is_null() {
            GlobalFree(h);
            return E_OUTOFMEMORY;
        }
        p.cast::<DropFiles>().write_unaligned(DropFiles {
            files: head as u32,
            pt: Point::default(),
            nc: 0,
            wide: 1,
        });
        std::ptr::copy_nonoverlapping(path.as_ptr(), p.add(head).cast::<u16>(), path.len());
        GlobalUnlock(h);
        *medium = StgMedium {
            tymed: TYMED_HGLOBAL,
            data: h,
            release: std::ptr::null_mut(),
        };
        S_OK
    }
}

unsafe extern "system" fn get_data_here(
    _: *mut DataObject,
    _: *const FormatEtc,
    _: *mut StgMedium,
) -> Hresult {
    E_NOTIMPL
}

unsafe extern "system" fn query_get_data(_: *mut DataObject, f: *const FormatEtc) -> Hresult {
    // SAFETY: a valid pointer from the caller, or null.
    match unsafe { offered(f) } {
        Ok(()) => S_OK,
        Err(e) => e,
    }
}

unsafe extern "system" fn get_canonical_format_etc(
    _: *mut DataObject,
    _: *const FormatEtc,
    out: *mut FormatEtc,
) -> Hresult {
    if out.is_null() {
        return E_POINTER;
    }
    // SAFETY: a valid pointer from the caller.
    unsafe { (*out).ptd = std::ptr::null_mut() };
    DATA_S_SAMEFORMATETC
}

unsafe extern "system" fn set_data(
    _: *mut DataObject,
    _: *const FormatEtc,
    _: *mut StgMedium,
    _: i32,
) -> Hresult {
    E_NOTIMPL
}

unsafe extern "system" fn enum_format_etc(
    _: *mut DataObject,
    direction: u32,
    out: *mut *mut c_void,
) -> Hresult {
    if out.is_null() {
        return E_POINTER;
    }
    if direction != DATADIR_GET {
        return E_NOTIMPL;
    }
    let f = hdrop_format();
    // SAFETY: one format, copied by the shell's enumerator.
    unsafe { SHCreateStdEnumFmtEtc(1, &f, out) }
}

unsafe extern "system" fn d_advise(
    _: *mut DataObject,
    _: *const FormatEtc,
    _: u32,
    _: *mut c_void,
    _: *mut u32,
) -> Hresult {
    OLE_E_ADVISENOTSUPPORTED
}

unsafe extern "system" fn d_unadvise(_: *mut DataObject, _: u32) -> Hresult {
    OLE_E_ADVISENOTSUPPORTED
}

unsafe extern "system" fn enum_d_advise(_: *mut DataObject, _: *mut *mut c_void) -> Hresult {
    OLE_E_ADVISENOTSUPPORTED
}

/// `IDropSource`: drop at the left button's release, cancel on Escape.
#[repr(C)]
pub struct DropSource {
    vtbl: *const SourceVtbl,
    refs: AtomicU32,
}

/// `IDropSource`'s vtable; COM reads its fields through the object's pointer.
#[repr(C)]
#[allow(dead_code)]
pub struct SourceVtbl {
    pub query_interface:
        unsafe extern "system" fn(*mut DropSource, *const Guid, *mut *mut c_void) -> Hresult,
    pub add_ref: unsafe extern "system" fn(*mut DropSource) -> u32,
    pub release: unsafe extern "system" fn(*mut DropSource) -> u32,
    pub query_continue_drag: unsafe extern "system" fn(*mut DropSource, i32, u32) -> Hresult,
    pub give_feedback: unsafe extern "system" fn(*mut DropSource, u32) -> Hresult,
}

static SOURCE_VTBL: SourceVtbl = SourceVtbl {
    query_interface: source_query_interface,
    add_ref: source_add_ref,
    release: source_release,
    query_continue_drag,
    give_feedback,
};

impl DropSource {
    /// A new drop source with one reference.
    pub fn new() -> *mut DropSource {
        Box::into_raw(Box::new(DropSource {
            vtbl: &SOURCE_VTBL,
            refs: AtomicU32::new(1),
        }))
    }

    /// The object's vtable (tests).
    ///
    /// # Safety
    /// `this` is a live drop source.
    #[cfg(test)]
    pub unsafe fn vtbl(this: *mut DropSource) -> &'static SourceVtbl {
        unsafe { &*(*this).vtbl }
    }
}

unsafe extern "system" fn source_query_interface(
    this: *mut DropSource,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> Hresult {
    if out.is_null() || iid.is_null() {
        return E_POINTER;
    }
    // SAFETY: valid pointers from the caller.
    unsafe {
        if *iid == IID_IUNKNOWN || *iid == IID_IDROPSOURCE {
            source_add_ref(this);
            *out = this.cast();
            S_OK
        } else {
            *out = std::ptr::null_mut();
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn source_add_ref(this: *mut DropSource) -> u32 {
    // SAFETY: a live object.
    unsafe { (*this).refs.fetch_add(1, Ordering::AcqRel) + 1 }
}

unsafe extern "system" fn source_release(this: *mut DropSource) -> u32 {
    // SAFETY: a live object; the last reference frees it.
    unsafe {
        let n = (*this).refs.fetch_sub(1, Ordering::AcqRel) - 1;
        if n == 0 {
            drop(Box::from_raw(this));
        }
        n
    }
}

unsafe extern "system" fn query_continue_drag(
    _: *mut DropSource,
    escape: i32,
    keys: u32,
) -> Hresult {
    if escape != 0 {
        DRAGDROP_S_CANCEL
    } else if keys & left_button().1 == 0 {
        DRAGDROP_S_DROP
    } else {
        S_OK
    }
}

unsafe extern "system" fn give_feedback(_: *mut DropSource, _: u32) -> Hresult {
    DRAGDROP_S_USEDEFAULTCURSORS
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The data object as a drop target reads it: the format query, the enumeration and the
    /// `CF_HDROP` holding the path; the drop source's answers.
    #[test]
    fn the_data_object_holds_the_file_as_a_drop() {
        let path = std::env::temp_dir()
            .join("OpenRB-338 test")
            .join("303-1 A1.mid");
        let d = DataObject::new(&path);
        // SAFETY: the object's own vtable on a live object, as COM callers do.
        unsafe {
            let v = DataObject::vtbl(d);
            let f = hdrop_format();
            assert_eq!((v.query_get_data)(d, &f), S_OK);
            let other = FormatEtc { cf: 1, ..f };
            assert_eq!((v.query_get_data)(d, &other), DV_E_FORMATETC);
            // QueryInterface hands out the object with a new reference
            let mut out = std::ptr::null_mut();
            assert_eq!((v.query_interface)(d, &IID_IDATAOBJECT, &mut out), S_OK);
            assert_eq!(out, d.cast());
            assert_eq!((v.release)(d), 1);
            assert_eq!(
                (v.query_interface)(d, &IID_IDROPSOURCE, &mut out),
                E_NOINTERFACE
            );
            // the enumerator offers CF_HDROP
            let mut e: *mut c_void = std::ptr::null_mut();
            assert_eq!((v.enum_format_etc)(d, DATADIR_GET, &mut e), S_OK);
            assert!(!e.is_null());
            // IEnumFORMATETC: QueryInterface, AddRef, Release, Next, ...
            type Next =
                unsafe extern "system" fn(*mut c_void, u32, *mut FormatEtc, *mut u32) -> Hresult;
            type Release = unsafe extern "system" fn(*mut c_void) -> u32;
            let vt = *(e as *const *const usize);
            let next: Next = std::mem::transmute(*vt.add(3));
            let release: Release = std::mem::transmute(*vt.add(2));
            let mut got = hdrop_format();
            got.cf = 0;
            let mut n = 0u32;
            assert_eq!(next(e, 1, &mut got, &mut n), S_OK);
            assert_eq!((n, got.cf, got.tymed), (1, CF_HDROP, TYMED_HGLOBAL));
            release(e);
            // the drop: one file, the path
            let mut m = StgMedium {
                tymed: 0,
                data: std::ptr::null_mut(),
                release: std::ptr::null_mut(),
            };
            assert_eq!((v.get_data)(d, &f, &mut m), S_OK);
            assert_eq!(m.tymed, TYMED_HGLOBAL);
            assert_eq!(
                DragQueryFileW(m.data, 0xFFFF_FFFF, std::ptr::null_mut(), 0),
                1
            );
            let mut buf = [0u16; 512];
            let len = DragQueryFileW(m.data, 0, buf.as_mut_ptr(), buf.len() as u32) as usize;
            assert_eq!(
                String::from_utf16(&buf[..len]).unwrap(),
                path.to_str().unwrap()
            );
            ReleaseStgMedium(&mut m);
            assert_eq!((v.release)(d), 0);
        }
        let s = DropSource::new();
        // SAFETY: as above.
        unsafe {
            let v = DropSource::vtbl(s);
            let (_, mk) = left_button();
            assert_eq!((v.query_continue_drag)(s, 0, mk), S_OK);
            assert_eq!((v.query_continue_drag)(s, 0, 0), DRAGDROP_S_DROP);
            assert_eq!((v.query_continue_drag)(s, 1, mk), DRAGDROP_S_CANCEL);
            assert_eq!((v.release)(s), 0);
        }
    }
}
