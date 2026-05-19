use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub struct OverlayManager;

impl OverlayManager {
    pub fn create_overlay_window(app: &AppHandle) -> anyhow::Result<()> {
        if app.get_webview_window("overlay").is_some() {
            return Ok(());
        }

        let window =
            WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
                .title("Overlay")
                .inner_size(320.0, 160.0)
                .position(100.0, 100.0)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .build()?;

        // Windows: 设置点击穿透和分层窗口样式
        #[cfg(windows)]
        unsafe {
            use windows::Win32::Foundation::HWND;
            use windows::Win32::UI::WindowsAndMessaging::{
                GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
            };

            if let Ok(hwnd_val) = window.hwnd() {
                // tauri 依赖的 windows crate 版本可能与项目直接依赖的版本不同，
                // 因此通过原始指针重新构造 HWND 以避免类型不匹配
                let raw = hwnd_val.0 as *mut std::ffi::c_void;
                let hwnd = HWND(raw);
                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                SetWindowLongW(
                    hwnd,
                    GWL_EXSTYLE,
                    ex_style | WS_EX_TRANSPARENT.0 as i32 | WS_EX_LAYERED.0 as i32,
                );
            }
        }

        // 焦点变化时自动切换点击穿透：获得焦点时可交互，失去焦点后穿透到游戏
        let focus_window = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                let _ = Self::set_click_through(&focus_window, !focused);
            }
        });

        // 边缘吸附监听
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Moved(position) = event {
                if let Ok(Some(monitor)) = window_clone.current_monitor() {
                    let screen_size = monitor.size();
                    let margin = 20i32;
                    let mut new_x = position.x;
                    let mut new_y = position.y;
                    let mut snapped = false;

                    if let Ok(win_size) = window_clone.outer_size() {
                        // 左边缘
                        if position.x < margin {
                            new_x = 0;
                            snapped = true;
                        }
                        // 右边缘
                        else if position.x + win_size.width as i32
                            > screen_size.width as i32 - margin
                        {
                            new_x = screen_size.width as i32 - win_size.width as i32;
                            snapped = true;
                        }

                        // 上边缘
                        if position.y < margin {
                            new_y = 0;
                            snapped = true;
                        }
                        // 下边缘
                        else if position.y + win_size.height as i32
                            > screen_size.height as i32 - margin
                        {
                            new_y = screen_size.height as i32 - win_size.height as i32;
                            snapped = true;
                        }

                        if snapped {
                            let _ = window_clone.set_position(tauri::Position::Physical(
                                tauri::PhysicalPosition { x: new_x, y: new_y },
                            ));
                            let _ = window_clone.emit("overlay:docked", true);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn toggle_overlay(app: &AppHandle) -> anyhow::Result<()> {
        if let Some(window) = app.get_webview_window("overlay") {
            if window.is_visible()? {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        } else {
            Self::create_overlay_window(app)?;
        }
        Ok(())
    }

    /// 设置 overlay 窗口的点击穿透状态（Windows）
    #[cfg(windows)]
    pub fn set_click_through(window: &WebviewWindow, enable: bool) -> anyhow::Result<()> {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_TRANSPARENT,
        };

        unsafe {
            let hwnd_val = window.hwnd()?;
            let raw = hwnd_val.0 as *mut std::ffi::c_void;
            let hwnd = HWND(raw);
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            let new_style = if enable {
                ex_style | WS_EX_TRANSPARENT.0 as i32
            } else {
                ex_style & !(WS_EX_TRANSPARENT.0 as i32)
            };
            SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_click_through(_window: &WebviewWindow, _enable: bool) -> anyhow::Result<()> {
        Ok(())
    }
}
