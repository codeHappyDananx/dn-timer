#![allow(dead_code)]

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowRect};

extern "system" {
    fn PrintWindow(
        hwnd: HWND,
        hdc_blt: windows::Win32::Graphics::Gdi::HDC,
        n_flags: u32,
    ) -> windows::Win32::Foundation::BOOL;
}

const PW_RENDERFULLCONTENT: u32 = 0x00000002;

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn find_game_window() -> Option<HWND> {
    // 尝试查找龙之谷游戏窗口（窗口类名或标题包含 "DragonNest" 或 "龙之谷"）
    let candidates = [
        (None::<&str>, Some("DragonNest")),
        (None::<&str>, Some("龙之谷")),
        (Some("DragonNestClass"), None::<&str>),
    ];

    for (class, title) in candidates {
        let class_wide = class.map(|c| to_wide(c));
        let title_wide = title.map(|t| to_wide(t));

        let class_ptr = class_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());
        let title_ptr = title_wide
            .as_ref()
            .map(|v| PCWSTR(v.as_ptr()))
            .unwrap_or(PCWSTR::null());

        let hwnd = unsafe { FindWindowW(class_ptr, title_ptr).ok()? };
        if !hwnd.0.is_null() {
            return Some(hwnd);
        }
    }

    None
}

/// 捕获窗口内容，优先使用 PrintWindow + PW_RENDERFULLCONTENT，失败则 fallback 到 BitBlt
pub fn capture_window(hwnd: HWND) -> anyhow::Result<Vec<u8>> {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect)?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width <= 0 || height <= 0 {
            anyhow::bail!("Invalid window size: {}x{}", width, height);
        }

        let hdc_screen = GetDC(HWND(0 as _));
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.0.is_null() {
            let _ = ReleaseDC(HWND(0 as _), hdc_screen);
            anyhow::bail!("Failed to create compatible DC");
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.0.is_null() {
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0 as _), hdc_screen);
            anyhow::bail!("Failed to create compatible bitmap");
        }

        let old = SelectObject(hdc_mem, hbm);

        // 优先尝试 PrintWindow with PW_RENDERFULLCONTENT
        let pw_ok = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT).as_bool();

        if !pw_ok {
            // fallback: BitBlt
            let hdc_window = GetDC(hwnd);
            let _ = BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, SRCCOPY);
            let _ = ReleaseDC(hwnd, hdc_window);
        }

        let bgra = bitmap_to_bgra(hbm, width, height)?;

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND(0 as _), hdc_screen);

        Ok(bgra)
    }
}

/// 截取屏幕指定区域，返回 BGRA 字节流
pub fn capture_screen_region(x: i32, y: i32, width: i32, height: i32) -> anyhow::Result<Vec<u8>> {
    if width <= 0 || height <= 0 {
        anyhow::bail!("Invalid region size: {}x{}", width, height);
    }

    unsafe {
        let hdc_screen = GetDC(HWND(0 as _));
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.0.is_null() {
            let _ = ReleaseDC(HWND(0 as _), hdc_screen);
            anyhow::bail!("Failed to create compatible DC");
        }

        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbm.0.is_null() {
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND(0 as _), hdc_screen);
            anyhow::bail!("Failed to create compatible bitmap");
        }

        let old = SelectObject(hdc_mem, hbm);
        let _ = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, x, y, SRCCOPY);

        let bgra = bitmap_to_bgra(hbm, width, height)?;

        SelectObject(hdc_mem, old);
        let _ = DeleteObject(hbm);
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(HWND(0 as _), hdc_screen);

        Ok(bgra)
    }
}

/// 将 HBITMAP 转换为 BGRA Vec<u8>
unsafe fn bitmap_to_bgra(
    hbm: windows::Win32::Graphics::Gdi::HBITMAP,
    width: i32,
    height: i32,
) -> anyhow::Result<Vec<u8>> {
    let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // 负值表示自顶向下
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [Default::default(); 1],
    };

    let row_bytes = ((width * 4 + 3) / 4) * 4;
    let mut buffer = vec![0u8; (row_bytes * height) as usize];

    let hdc = GetDC(HWND(0 as _));
    let lines = GetDIBits(
        hdc,
        hbm,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    let _ = ReleaseDC(HWND(0 as _), hdc);

    if lines == 0 {
        anyhow::bail!("GetDIBits failed");
    }

    Ok(buffer)
}
