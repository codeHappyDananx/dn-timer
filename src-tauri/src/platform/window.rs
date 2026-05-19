use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager};
use windows::core::PCWSTR;
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::store::Database;

const REG_PATH: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const REG_KEY_NAME: &str = "dn-timer";
pub const NORMAL_WIDTH: f64 = 340.0;
pub const NORMAL_HEIGHT: f64 = 500.0;
const MAX_SCREEN_WIDTH_RATIO: f64 = 0.92;

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn save_window_position<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    db: &Arc<Mutex<Database>>,
) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let pos = window.outer_position()?;
        let mut db = db.lock().unwrap();
        db.set_config("window_x", &pos.x.to_string())?;
        db.set_config("window_y", &pos.y.to_string())?;
    }
    Ok(())
}

pub fn restore_window_position<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    db: &Database,
) -> anyhow::Result<()> {
    let x = db.get_config("window_x")?.and_then(|v| v.parse().ok());
    let y = db.get_config("window_y")?.and_then(|v| v.parse().ok());

    if let Some(window) = app.get_webview_window("main") {
        if let (Some(x), Some(y)) = (x, y) {
            let _ =
                window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        }
        let _ = set_normal_window_size(&window);
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DockSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
struct DockSnapshot {
    side: DockSide,
    position: tauri::PhysicalPosition<i32>,
    size: tauri::PhysicalSize<u32>,
}

// Dock / undock state
static DOCKED: AtomicBool = AtomicBool::new(false);
static DOCK_SNAPSHOT: Mutex<Option<DockSnapshot>> = Mutex::new(None);
static DRAGGING: AtomicBool = AtomicBool::new(false);
static PRE_DOCK_ALWAYS_ON_TOP: AtomicBool = AtomicBool::new(false);
static DOCK_WIDTH: u32 = 62;
static DOCK_HEIGHT: u32 = 62;
static DOCK_MARGIN: i32 = 8;
static DOCK_CURSOR_THRESHOLD: i32 = 38;
static WINDOW_SNAP_THRESHOLD: i32 = 28;

pub fn is_docked() -> bool {
    DOCKED.load(Ordering::SeqCst)
}

pub fn dock_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> anyhow::Result<()> {
    dock_window_to_side(app, DockSide::Right)
}

pub fn begin_window_drag<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    dock_on_release: bool,
) -> anyhow::Result<()> {
    if DRAGGING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let app = app.clone();
    let Some(window) = app.get_webview_window("main") else {
        DRAGGING.store(false, Ordering::SeqCst);
        return Ok(());
    };

    let window_pos = window.outer_position()?;
    let window_size = window.outer_size()?;
    let mut cursor = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut cursor)?;
    }
    let offset_x = cursor.x - window_pos.x;
    let offset_y = cursor.y - window_pos.y;

    std::thread::spawn(move || {
        let mut last_pos: Option<tauri::PhysicalPosition<i32>> = None;
        let mut last_cursor = cursor;
        let mut preview_side: Option<DockSide> = None;
        loop {
            let pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) & 0x8000u16 as i16 != 0 };
            if !pressed {
                break;
            }

            let mut cursor = POINT { x: 0, y: 0 };
            if unsafe { GetCursorPos(&mut cursor).is_ok() } {
                last_cursor = cursor;
                let side = if dock_on_release {
                    dock_side_from_cursor(&window, cursor)
                } else {
                    None
                };

                if side != preview_side {
                    let _ = app.emit("window:dock-preview", side.is_some());
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_resizable(side.is_none());
                        let _ = window.set_shadow(false);
                        let _ = window.set_size(if side.is_some() {
                            tauri::Size::Physical(tauri::PhysicalSize {
                                width: DOCK_WIDTH,
                                height: DOCK_HEIGHT,
                            })
                        } else {
                            tauri::Size::Physical(window_size)
                        });
                    }
                    preview_side = side;
                    last_pos = None;
                }

                let next = if let Some(side) = side {
                    preview_position_from_cursor(&window, cursor, side)
                        .unwrap_or(tauri::PhysicalPosition {
                            x: cursor.x - (DOCK_WIDTH as i32 / 2),
                            y: cursor.y - (DOCK_HEIGHT as i32 / 2),
                        })
                } else if DOCKED.load(Ordering::SeqCst) {
                    visible_position_from_cursor(&window, cursor).unwrap_or(tauri::PhysicalPosition {
                        x: cursor.x - (DOCK_WIDTH as i32 / 2),
                        y: cursor.y - (DOCK_HEIGHT as i32 / 2),
                    })
                } else {
                    let next = tauri::PhysicalPosition {
                        x: cursor.x - offset_x,
                        y: cursor.y - offset_y,
                    };
                    if dock_on_release {
                        snap_window_position(&window, next, window_size).unwrap_or(next)
                    } else {
                        next
                    }
                };

                if last_pos != Some(next) {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.set_position(tauri::Position::Physical(next));
                    }
                    last_pos = Some(next);
                }
            }
            std::thread::sleep(Duration::from_millis(12));
        }

        if dock_on_release && !DOCKED.load(Ordering::SeqCst) {
            let side = app
                .get_webview_window("main")
                .and_then(|window| dock_side_from_cursor(&window, last_cursor));

            if let Some(side) = side {
                {
                    let mut snapshot = DOCK_SNAPSHOT.lock().unwrap();
                            *snapshot = Some(DockSnapshot {
                                side,
                                position: window_pos,
                                size: window_size,
                            });
                        }
                let _ = app.emit("window:docked", true);
                std::thread::sleep(Duration::from_millis(80));
                let _ = dock_window_to_side(&app, side);
            } else if preview_side.is_some() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_size(tauri::Size::Physical(window_size));
                    let _ = window.set_resizable(true);
                    let _ = window.set_shadow(false);
                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: last_cursor.x - offset_x,
                        y: last_cursor.y - offset_y,
                    }));
                }
                let _ = app.emit("window:dock-preview", false);
            }
        }

        DRAGGING.store(false, Ordering::SeqCst);
    });

    Ok(())
}

fn dock_window_to_side<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    side: DockSide,
) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let monitor = window
            .current_monitor()?
            .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let pos = window.outer_position()?;
        let size = window.outer_size()?;
        let center_x = pos.x + size.width as i32 / 2;
        let center_y = pos.y + size.height as i32 / 2;

        {
            let mut snapshot = DOCK_SNAPSHOT.lock().unwrap();
            if snapshot.is_none() {
                *snapshot = Some(DockSnapshot {
                    side,
                    position: pos,
                    size,
                });
            }
        }

        let dock_w = DOCK_WIDTH as i32;
        let dock_h = DOCK_HEIGHT as i32;
        let left = m_pos.x;
        let top = m_pos.y;
        let right_edge = m_pos.x + m_size.width as i32;
        let bottom_edge = m_pos.y + m_size.height as i32;
        let x_min = left + DOCK_MARGIN;
        let x_max = right_edge - dock_w - DOCK_MARGIN;
        let y_min = top + DOCK_MARGIN;
        let y_max = bottom_edge - dock_h - DOCK_MARGIN;
        let x = clamp_i32(center_x - dock_w / 2, x_min, x_max);
        let y = clamp_i32(center_y - dock_h / 2, y_min, y_max);

        let dock_pos = match side {
            DockSide::Left => tauri::PhysicalPosition { x: x_min, y },
            DockSide::Right => tauri::PhysicalPosition { x: x_max, y },
            DockSide::Top => tauri::PhysicalPosition { x, y: y_min },
            DockSide::Bottom => tauri::PhysicalPosition { x, y: y_max },
        };

        let _ = app.emit("window:docked", true);
        std::thread::sleep(Duration::from_millis(60));

        PRE_DOCK_ALWAYS_ON_TOP.store(window.is_always_on_top().unwrap_or(false), Ordering::SeqCst);
        window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: DOCK_WIDTH,
            height: DOCK_HEIGHT,
        }))?;
        window.set_position(tauri::Position::Physical(dock_pos))?;
        let _ = window.set_resizable(false);
        let _ = window.set_shadow(false);
        let _ = window.set_always_on_top(true);
        DOCKED.store(true, Ordering::SeqCst);
    }
    Ok(())
}

pub fn undock_window<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let snapshot = {
            let mut snapshot = DOCK_SNAPSHOT.lock().unwrap();
            snapshot.take()
        };

        if let Some(snapshot) = snapshot {
            window.set_size(tauri::Size::Physical(snapshot.size))?;
            let pos = restore_position_away_from_edge(&window, snapshot)?;
            window.set_position(tauri::Position::Physical(pos))?;
        } else {
            set_normal_window_size(&window)?;
        }
        let _ = window.set_resizable(true);
        let _ = window.set_shadow(false);
        let _ = window.set_always_on_top(PRE_DOCK_ALWAYS_ON_TOP.load(Ordering::SeqCst));
        DOCKED.store(false, Ordering::SeqCst);
        let _ = app.emit("window:docked", false);
    }
    Ok(())
}

// Simple mode dimensions
static SIMPLE_WIDTH: Mutex<f64> = Mutex::new(260.0);
static SIMPLE_HEIGHT_PER_SLOT: f64 = 96.0;
static SIMPLE_PADDING: f64 = 24.0;
static SIMPLE_MIN_HEIGHT: f64 = 120.0;
static SIMPLE_MIN_SCALE: f64 = 0.72;
static SIMPLE_MAX_SCALE: f64 = 1.08;

pub fn normalize_simple_scale(scale: f64) -> f64 {
    if scale.is_finite() {
        scale.clamp(SIMPLE_MIN_SCALE, SIMPLE_MAX_SCALE)
    } else {
        0.88
    }
}

pub fn enter_simple_mode<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    slot_count: usize,
    width: f64,
    scale: f64,
) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        let scale = normalize_simple_scale(scale);
        let width = (width * scale).max(*SIMPLE_WIDTH.lock().unwrap() * scale);
        let height = ((SIMPLE_PADDING + (slot_count.max(1) as f64) * SIMPLE_HEIGHT_PER_SLOT)
            * scale)
            .max(SIMPLE_MIN_HEIGHT * scale);

        window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))?;
        keep_window_inside_monitor(&window)?;
    }
    Ok(())
}

pub fn exit_simple_mode<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> anyhow::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        set_normal_window_size(&window)?;
    }
    Ok(())
}

fn set_normal_window_size<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> anyhow::Result<()> {
    set_content_window_size(window, NORMAL_WIDTH, NORMAL_HEIGHT)?;
    Ok(())
}

pub fn set_content_window_size<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    width: f64,
    height: f64,
) -> anyhow::Result<()> {
    let max_width = window
        .current_monitor()?
        .map(|monitor| monitor.size().width as f64 * MAX_SCREEN_WIDTH_RATIO)
        .unwrap_or(width);
    let width = width.clamp(NORMAL_WIDTH, max_width.max(NORMAL_WIDTH));
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))?;
    keep_window_inside_monitor(window)?;
    Ok(())
}

fn keep_window_inside_monitor<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> anyhow::Result<()> {
    let monitor = window
        .current_monitor()?
        .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let pos = window.outer_position()?;
    let size = window.outer_size()?;
    let margin = DOCK_MARGIN;
    let right_edge = m_pos.x + m_size.width as i32;
    let bottom_edge = m_pos.y + m_size.height as i32;
    let x_min = m_pos.x + margin;
    let y_min = m_pos.y + margin;
    let x_max = right_edge - size.width as i32 - margin;
    let y_max = bottom_edge - size.height as i32 - margin;
    let next = tauri::PhysicalPosition {
        x: clamp_i32(pos.x, x_min, x_max.max(x_min)),
        y: clamp_i32(pos.y, y_min, y_max.max(y_min)),
    };

    if next != pos {
        window.set_position(tauri::Position::Physical(next))?;
    }

    Ok(())
}


/// 后台线程保留为兼容入口，贴边收起只在自定义拖动释放时触发
pub fn start_dock_watcher(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            let _ = &app;
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
}

fn dock_side_from_cursor<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    cursor: POINT,
) -> Option<DockSide> {
    let monitor = window.current_monitor().ok().flatten()?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let left = (cursor.x - m_pos.x).abs();
    let top = (cursor.y - m_pos.y).abs();
    let right = (m_pos.x + m_size.width as i32 - cursor.x).abs();
    let bottom = (m_pos.y + m_size.height as i32 - cursor.y).abs();

    [
        (DockSide::Left, left),
        (DockSide::Right, right),
        (DockSide::Top, top),
        (DockSide::Bottom, bottom),
    ]
    .into_iter()
    .filter(|(_, distance)| *distance <= DOCK_CURSOR_THRESHOLD)
    .min_by_key(|(_, distance)| *distance)
    .map(|(side, _)| side)
}

fn preview_position_from_cursor<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    cursor: POINT,
    side: DockSide,
) -> Option<tauri::PhysicalPosition<i32>> {
    let monitor = window.current_monitor().ok().flatten()?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let dock_w = DOCK_WIDTH as i32;
    let dock_h = DOCK_HEIGHT as i32;
    let right_edge = m_pos.x + m_size.width as i32;
    let bottom_edge = m_pos.y + m_size.height as i32;
    let x_min = m_pos.x + DOCK_MARGIN;
    let x_max = right_edge - dock_w - DOCK_MARGIN;
    let y_min = m_pos.y + DOCK_MARGIN;
    let y_max = bottom_edge - dock_h - DOCK_MARGIN;
    let x = clamp_i32(cursor.x - dock_w / 2, x_min, x_max);
    let y = clamp_i32(cursor.y - dock_h / 2, y_min, y_max);

    Some(match side {
        DockSide::Left => tauri::PhysicalPosition { x: x_min, y },
        DockSide::Right => tauri::PhysicalPosition { x: x_max, y },
        DockSide::Top => tauri::PhysicalPosition { x, y: y_min },
        DockSide::Bottom => tauri::PhysicalPosition { x, y: y_max },
    })
}

fn snap_window_position<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    pos: tauri::PhysicalPosition<i32>,
    size: tauri::PhysicalSize<u32>,
) -> Option<tauri::PhysicalPosition<i32>> {
    let monitor = window.current_monitor().ok().flatten()?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let right_edge = m_pos.x + m_size.width as i32;
    let bottom_edge = m_pos.y + m_size.height as i32;
    let width = size.width as i32;
    let height = size.height as i32;

    let mut x = pos.x;
    let mut y = pos.y;

    if (pos.x - m_pos.x).abs() <= WINDOW_SNAP_THRESHOLD {
        x = m_pos.x;
    } else if (right_edge - (pos.x + width)).abs() <= WINDOW_SNAP_THRESHOLD {
        x = right_edge - width;
    }

    if (pos.y - m_pos.y).abs() <= WINDOW_SNAP_THRESHOLD {
        y = m_pos.y;
    } else if (bottom_edge - (pos.y + height)).abs() <= WINDOW_SNAP_THRESHOLD {
        y = bottom_edge - height;
    }

    Some(tauri::PhysicalPosition { x, y })
}

fn visible_position_from_cursor<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    cursor: POINT,
) -> Option<tauri::PhysicalPosition<i32>> {
    let monitor = window.current_monitor().ok().flatten()?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let dock_w = DOCK_WIDTH as i32;
    let dock_h = DOCK_HEIGHT as i32;
    let right_edge = m_pos.x + m_size.width as i32;
    let bottom_edge = m_pos.y + m_size.height as i32;
    let x = clamp_i32(cursor.x - dock_w / 2, m_pos.x + DOCK_MARGIN, right_edge - dock_w - DOCK_MARGIN);
    let y = clamp_i32(cursor.y - dock_h / 2, m_pos.y + DOCK_MARGIN, bottom_edge - dock_h - DOCK_MARGIN);

    Some(tauri::PhysicalPosition { x, y })
}

fn restore_position_away_from_edge<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
    snapshot: DockSnapshot,
) -> anyhow::Result<tauri::PhysicalPosition<i32>> {
    let monitor = window
        .current_monitor()?
        .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;
    let m_pos = monitor.position();
    let m_size = monitor.size();
    let width = snapshot.size.width as i32;
    let height = snapshot.size.height as i32;
    let right_edge = m_pos.x + m_size.width as i32;
    let bottom_edge = m_pos.y + m_size.height as i32;
    let release_gap = 96;
    let x_max = right_edge - width - DOCK_MARGIN;
    let y_max = bottom_edge - height - DOCK_MARGIN;

    let mut x = clamp_i32(snapshot.position.x, m_pos.x + DOCK_MARGIN, x_max);
    let mut y = clamp_i32(snapshot.position.y, m_pos.y + DOCK_MARGIN, y_max);

    match snapshot.side {
        DockSide::Left => x = m_pos.x + release_gap,
        DockSide::Right => x = right_edge - width - release_gap,
        DockSide::Top => y = m_pos.y + release_gap,
        DockSide::Bottom => y = bottom_edge - height - release_gap,
    }

    Ok(tauri::PhysicalPosition { x, y })
}

fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

fn get_exe_path() -> anyhow::Result<String> {
    let exe = std::env::current_exe()?;
    Ok(exe.to_string_lossy().to_string())
}

pub fn set_startup(enable: bool) -> anyhow::Result<()> {
    unsafe {
        let path_wide = to_wide(REG_PATH);
        let mut hkey = std::mem::zeroed();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(path_wide.as_ptr()),
            0,
            KEY_WRITE,
            &mut hkey,
        );
        if status.0 != 0 {
            anyhow::bail!("Failed to open registry key: {}", status.0);
        }

        if enable {
            let exe_path = get_exe_path()?;
            let value_wide = to_wide(&format!("\"{}\"", exe_path));
            let name_wide = to_wide(REG_KEY_NAME);
            let bytes =
                std::slice::from_raw_parts(value_wide.as_ptr() as *const u8, value_wide.len() * 2);
            let status = RegSetValueExW(hkey, PCWSTR(name_wide.as_ptr()), 0, REG_SZ, Some(bytes));
            let _ = RegCloseKey(hkey);
            if status.0 != 0 {
                anyhow::bail!("Failed to set registry value: {}", status.0);
            }
        } else {
            let name_wide = to_wide(REG_KEY_NAME);
            let status = RegDeleteValueW(hkey, PCWSTR(name_wide.as_ptr()));
            let _ = RegCloseKey(hkey);
            if status.0 != 0 && status.0 != 2 {
                anyhow::bail!("Failed to delete registry value: {}", status.0);
            }
        }
    }
    Ok(())
}

pub fn is_startup_enabled() -> bool {
    unsafe {
        let path_wide = to_wide(REG_PATH);
        let mut hkey = std::mem::zeroed();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(path_wide.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );
        if status.0 != 0 {
            return false;
        }

        let name_wide = to_wide(REG_KEY_NAME);
        let mut data_len: u32 = 0;
        let status = RegQueryValueExW(
            hkey,
            PCWSTR(name_wide.as_ptr()),
            None,
            None,
            None,
            Some(&mut data_len),
        );
        let _ = RegCloseKey(hkey);
        status.0 == 0
    }
}
