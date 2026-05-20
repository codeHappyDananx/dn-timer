// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    ensure_admin();

    dn_timer_lib::run()
}

#[cfg(windows)]
fn ensure_admin() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{IsUserAnAdmin, ShellExecuteW};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    unsafe {
        if IsUserAnAdmin().as_bool() {
            return;
        }
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };

    let operation = wide("runas");
    let file = wide_os(exe.as_os_str());
    let result = unsafe {
        ShellExecuteW(
            HWND(null_mut()),
            PCWSTR(operation.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 as isize > 32 {
        std::process::exit(0);
    }

    std::process::exit(1);

    fn wide(value: &str) -> Vec<u16> {
        OsStr::new(value).encode_wide().chain(Some(0)).collect()
    }

    fn wide_os(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(Some(0)).collect()
    }
}
