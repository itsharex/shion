use std::cell::RefCell;
use std::ffi::OsStr;
use std::io::Cursor;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::ptr::null_mut;
use std::{mem, thread};

use image::{ImageBuffer, ImageFormat, Rgba};
use nodio_win32::SessionState;
use nodio_win32::{AudioSessionEvent, Win32Context};
use winapi::shared::minwindef::DWORD;
use winapi::shared::minwindef::LPCVOID;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::minwindef::WORD;
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::HICON__;
use winapi::shared::windef::HWND;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::{EnumProcessModulesEx, GetModuleFileNameExW};
use winapi::um::shellapi::ExtractIconExW;
use winapi::um::wingdi::GetDIBits;
use winapi::um::wingdi::GetObjectW;
use winapi::um::wingdi::BITMAP;
use winapi::um::wingdi::BITMAPINFOHEADER;
use winapi::um::wingdi::BI_RGB;
use winapi::um::wingdi::DIB_RGB_COLORS;
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
use winapi::um::winnt::PROCESS_VM_READ;
use winapi::um::winuser::CallNextHookEx;
use winapi::um::winuser::GetDC;
use winapi::um::winuser::GetDesktopWindow;
use winapi::um::winuser::GetMessageW;
use winapi::um::winuser::GetWindowThreadProcessId;
use winapi::um::winuser::ReleaseDC;
use winapi::um::winuser::EVENT_SYSTEM_FOREGROUND;
use winapi::um::winuser::ICONINFO;
use winapi::um::winuser::MSG;
use winapi::um::winuser::{
    DispatchMessageW, GetIconInfo, GetWindowTextLengthW, GetWindowTextW, SetWinEventHook,
    TranslateMessage, UnhookWinEvent, WINEVENT_OUTOFCONTEXT,
};
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

use super::shared::WatchAudioOption;
use super::shared::{Program, WatchWindowOption};

thread_local! {
    static WINDOW: RefCell<Option<Box<dyn Fn(Program) -> ()>>> = RefCell::new(None);
}

unsafe extern "system" fn handle_event(
    _: winapi::shared::windef::HWINEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    _: LONG,
    _: LONG,
    _: DWORD,
    _: DWORD,
) {
    let is_switch_window = event == EVENT_SYSTEM_FOREGROUND;

    if !is_switch_window {
        return;
    }

    let application_path = get_application_path(hwnd);

    if application_path.is_none() {
        return;
    }

    let path = application_path.unwrap();

    let description = get_application_description(path.clone());
    let description = if description.is_some() && description.clone().unwrap().len() != 0 {
        description.unwrap()
    } else {
        let file_stem = Path::new(&path).file_stem().unwrap().to_str().unwrap();
        file_stem.to_string()
    };

    let buffer = get_image_by_path(path.clone());

    let program = Program {
        path,
        description,
        icon: buffer,
    };

    WINDOW.with(|i| {
        if let Some(f) = &*i.borrow() {
            f(program);
        }
    });
}

fn to_u16(str: String) -> Vec<u16> {
    OsStr::new(&str).encode_wide().chain(once(0)).collect()
}

fn get_application_title(hwnd: HWND) -> Option<String> {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len == 0 {
        return None;
    }
    let mut title: Vec<u16> = vec![0; len as usize + 1];
    let ret = unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), len + 1) };
    if ret == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&title[..ret as usize]))
}

fn get_application_path(hwnd: HWND) -> Option<String> {
    let mut process_id = 0;
    let result = unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
    if result == 0 {
        return None;
    }
    let process_handle =
        unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };

    if process_handle.is_null() {
        return None;
    }
    let mut h_mod = null_mut();
    let mut cb_needed = 0;
    let result = unsafe {
        EnumProcessModulesEx(
            process_handle,
            &mut h_mod,
            std::mem::size_of_val(&h_mod) as u32,
            &mut cb_needed,
            0x03,
        )
    };
    if result == 0 {
        return None;
    }
    let mut path: [u16; MAX_PATH] = [0u16; MAX_PATH];
    let len = unsafe {
        GetModuleFileNameExW(process_handle, h_mod, path.as_mut_ptr(), path.len() as u32)
    } as usize;
    if len == 0 {
        return None;
    }
    let result = unsafe { CloseHandle(process_handle) };
    if result == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&path[..len]))
}

fn get_application_description(application_path: String) -> Option<String> {
    let application_path = to_u16(application_path);

    let mut handle = 0;
    let size = unsafe { GetFileVersionInfoSizeW(application_path.as_ptr(), &mut handle) };
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    let result = unsafe {
        GetFileVersionInfoW(
            application_path.as_ptr(),
            handle,
            size,
            data.as_mut_ptr() as LPVOID,
        )
    };
    if result == 0 {
        return None;
    }

    #[repr(C)]
    struct LANGANDCODEPAGE {
        language: WORD,
        codepage: WORD,
    }

    let mut value_ptr: LPVOID = ptr::null_mut();
    let mut value_size: DWORD = 0;
    let result = unsafe {
        VerQueryValueW(
            data.as_ptr() as LPCVOID,
            to_u16("\\VarFileInfo\\Translation".to_string()).as_ptr(),
            &mut value_ptr,
            &mut value_size,
        )
    };
    if result == 0 || value_size == 0 {
        return None;
    }
    let codepage = value_ptr as *const LANGANDCODEPAGE;
    let translation_key = unsafe {
        format!(
            "\\StringFileInfo\\{:04X}{:04X}",
            (*codepage).language,
            (*codepage).codepage
        )
    };

    let mut value_ptr: LPVOID = ptr::null_mut();
    let mut value_size: DWORD = 0;
    let result = unsafe {
        VerQueryValueW(
            data.as_ptr() as LPVOID,
            to_u16(format!("{}\\FileDescription", translation_key).to_string()).as_ptr(),
            &mut value_ptr,
            &mut value_size,
        )
    };
    if result == 0 || value_size == 0 {
        return None;
    }
    let value_ptr = value_ptr as *const u16;
    let description: Vec<u16> =
        unsafe { std::slice::from_raw_parts(value_ptr, (value_size - 1) as usize).to_vec() };
    Some(String::from_utf16_lossy(&description))
}

fn get_icon_handle(appliaction_path: String) -> Option<*mut HICON__> {
    let path_wide = to_u16(appliaction_path);
    let mut large_icon = ptr::null_mut();
    let mut small_icon = ptr::null_mut();
    unsafe { ExtractIconExW(path_wide.as_ptr(), 0, &mut large_icon, &mut small_icon, 1) };
    if large_icon.is_null() {
        return None;
    }
    Some(large_icon)
}

fn get_image_buffer(icon_handle: *mut HICON__) -> Vec<u8> {
    let mut icon_info = ICONINFO {
        fIcon: 0,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: ptr::null_mut(),
        hbmColor: ptr::null_mut(),
    };
    unsafe { GetIconInfo(icon_handle, &mut icon_info) };
    let mut bmp = BITMAP {
        bmType: 0,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ptr::null_mut(),
    };
    unsafe {
        GetObjectW(
            icon_info.hbmColor as *mut _,
            std::mem::size_of::<BITMAP>().try_into().unwrap(),
            &mut bmp as *mut BITMAP as *mut _,
        )
    };
    let hwnd = unsafe { GetDesktopWindow() };
    let hdc = unsafe { GetDC(hwnd) };
    let width = bmp.bmWidth;
    let height = bmp.bmHeight;

    let mut bitmap_info = BITMAPINFOHEADER {
        biSize: mem::size_of::<BITMAPINFOHEADER>() as DWORD,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };
    let mut bgra_bitmap_data = vec![0u8; (width * height * 4) as usize];
    unsafe {
        GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height as u32,
            bgra_bitmap_data.as_mut_ptr() as *mut _,
            &mut bitmap_info as *mut _ as *mut _,
            DIB_RGB_COLORS,
        )
    };

    unsafe { ReleaseDC(hwnd, hdc) };

    let mut rgba_bitmap_data = Vec::with_capacity(bgra_bitmap_data.len());
    for y in 0..height {
        for x in 0..width {
            let offset = ((y * width + x) * 4) as usize;
            let b = bgra_bitmap_data[offset];
            let g = bgra_bitmap_data[offset + 1];
            let r = bgra_bitmap_data[offset + 2];
            let a = bgra_bitmap_data[offset + 3];
            rgba_bitmap_data.extend_from_slice(&[r, g, b, a]);
        }
    }

    let icon_image =
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_vec(width as u32, height as u32, rgba_bitmap_data)
            .unwrap();

    let mut buffer = Vec::new();
    icon_image
        .write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
        .unwrap();

    buffer
}

pub fn get_image_by_path(appliaction_path: String) -> Vec<u8> {
    if let Some(handle) = get_icon_handle(appliaction_path) {
        return get_image_buffer(handle);
    }
    vec![]
}

fn watch_input(window: WatchWindowOption) {
    WINDOW.with(|f| *f.borrow_mut() = Some(Box::new(window)));

    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            ptr::null_mut(),
            Some(handle_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_null() {
        return;
    }

    unsafe {
        let mut msg: MSG = mem::zeroed();
        loop {
            let ret = GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
            if ret == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
            CallNextHookEx(null_mut(), 0, 0, 0);
        }
    }

    unsafe { UnhookWinEvent(hook) };
}

fn watch_audio<T>(audio: T)
where
    T: Fn(SessionState, String) + Send + Sync + 'static,
{
    Win32Context::new(move |event, name| match event {
        AudioSessionEvent::StateChange(state) => {
            audio(state, name);
        }
        _ => {}
    });
    loop {}
}

pub fn run(audio: WatchAudioOption, window: WatchWindowOption) {
    thread::spawn(|| {
        watch_audio(audio);
    });
    thread::spawn(|| {
        watch_input(window);
    });
}
