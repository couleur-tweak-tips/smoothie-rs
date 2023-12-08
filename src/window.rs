use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use windows_sys::minwindef::{BOOL, DWORD, HMONITOR, UINT};
use windows_sys::windef::{HWND, RECT};
use windows_sys::wincon::{GetConsoleWindow, SetConsoleTitleA};
use windows_sys::winuser::{
    GetMonitorInfoA, GetWindow, MonitorFromWindow, SetWindowLongA, SetWindowPos, ShowWindowAsync,
    HWND_NOTOPMOST, HWND_TOP, HWND_TOPMOST, MONITOR_DEFAULTTONEAREST, SW_NORMAL, SW_SHOW,
    SWP_NOMOVE, SWP_NOSIZE, WS_CLIPSIBLINGS, WS_EX_ACCEPTFILES, WS_EX_APPWINDOW, WS_EX_WINDOWEDGE,
    WS_OVERLAPPEDWINDOW, WS_POPUP, WS_VISIBLE,
};

fn set_console_window_title(wnd_title: *const c_char) -> BOOL {
    unsafe { SetConsoleTitleA(wnd_title as *const i8) }
}

fn is_console_host(b_relaunch: BOOL) -> BOOL {
    let b_console_host = unsafe { GetWindow(GetConsoleWindow(), GW_OWNER) == ptr::null_mut() };
    if b_console_host == 0 && b_relaunch != 0 {
        let conhost_exe = CString::new("conhost.exe").unwrap();
        let command_line = CString::new("").unwrap();
        unsafe {
            ShellExecuteA(
                ptr::null_mut(),
                b"open\0" as *const u8 as *const i8,
                conhost_exe.as_ptr(),
                command_line.as_ptr(),
                ptr::null(),
                SW_SHOW,
            );
        }
        std::process::exit(0);
    }
    b_console_host
}

fn set_console_params(
    b_borderless: BOOL,
    b_always_on_top: BOOL,
    wnd_pos: i32,
    wnd_cx: i32,
    wnd_cy: i32,
) -> BOOL {
    if is_console_host(0) == 0 {
        return 0;
    }
    let mut wnd_x = 0;
    let mut wnd_y = 0;
    let mut u_flags = 0;
    let h_wnd = unsafe { GetConsoleWindow() };
    let h_wnd_pos = if b_always_on_top != 0 {
        HWND_TOPMOST
    } else {
        HWND_TOP
    };
    let mut wnd_style = WS_VISIBLE | WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_VSCROLL;
    let mut wnd_ex_style = WS_EX_ACCEPTFILES | WS_EX_WINDOWEDGE | WS_EX_APPWINDOW;
    let h_monitor = unsafe { MonitorFromWindow(h_wnd, MONITOR_DEFAULTTONEAREST) };
    let mut mi: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if h_wnd.is_null()
        || h_monitor.is_null()
        || unsafe { GetMonitorInfoA(h_monitor, &mut mi as *mut RECT) } == 0
        || unsafe { ShowWindowAsync(h_wnd, SW_NORMAL) } == 0
        || unsafe {
            SetWindowPos(
                h_wnd,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            )
        } == 0
    {
        return 0;
    }
    if wnd_cx == 0 || wnd_cy == 0 {
        u_flags = SWP_NOSIZE;
    }
    if b_borderless != 0 {
        wnd_style = WS_VISIBLE | WS_POPUP;
        wnd_ex_style = WS_EX_APPWINDOW;
    }
    match wnd_pos {
        // Top Left
        1 => {
            wnd_x = mi.left;
            wnd_y = mi.top;
        }
        // Top Right
        2 => {
            wnd_x = mi.right - wnd_cx;
            wnd_y = mi.top;
        }
        // Bottom Left
        3 => {
            wnd_x = mi.left;
            wnd_y = mi.bottom - wnd_cy;
        }
        // Bottom Right
        4 => {
            wnd_x = mi.right - wnd_cx;
            wnd_y = mi.bottom - wnd_cy;
        }
        _ => return 0,
    }
    unsafe {
        SetWindowLongA(h_wnd, GWL_STYLE, wnd_style);
        SetWindowLongA(h_wnd, GWL_EXSTYLE, wnd_ex_style);
        SetWindowPos(
            h_wnd,
            h_wnd_pos,
            wnd_x,
            wnd_y,
            wnd_cx,
            wnd_cy,
            u_flags,
        );
    }
    1
}
