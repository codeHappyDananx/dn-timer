use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_HOME,
    VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_MENU, VK_NEXT, VK_PRIOR, VK_RCONTROL,
    VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN,
    WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
};

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = RefCell::new(None);
}

struct HookState {
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
    app_handle: AppHandle,
    bindings: Arc<Mutex<HashMap<String, String>>>,
    pressed_keys: HashSet<i32>,
}

pub struct HotkeyManager {
    app_handle: AppHandle,
    bindings: Arc<Mutex<HashMap<String, String>>>,
}

impl HotkeyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            bindings: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_bindings(&mut self, bindings: HashMap<String, String>) {
        let mut current = self.bindings.lock().unwrap();
        *current = bindings;
    }

    pub fn start(&self) {
        let app_handle = self.app_handle.clone();
        let bindings = Arc::clone(&self.bindings);

        std::thread::spawn(move || {
            unsafe {
                let keyboard_hook =
                    SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0);

                let mouse_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0);

                if let (Ok(kb_hook), Ok(m_hook)) = (keyboard_hook, mouse_hook) {
                    HOOK_STATE.with(|state| {
                        *state.borrow_mut() = Some(HookState {
                            keyboard_hook: kb_hook,
                            mouse_hook: m_hook,
                            app_handle,
                            bindings,
                            pressed_keys: HashSet::new(),
                        });
                    });

                    // Message loop for hooks
                    use windows::Win32::UI::WindowsAndMessaging::{
                        DispatchMessageW, GetMessageW, TranslateMessage, MSG,
                    };
                    let mut msg: MSG = std::mem::zeroed();
                    while GetMessageW(&mut msg, HWND(null_mut()), 0, 0).into() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        });
    }

    pub fn stop(&self) {
        HOOK_STATE.with(|state| {
            if let Some(ref state) = *state.borrow() {
                unsafe {
                    let _ = UnhookWindowsHookEx(state.keyboard_hook);
                    let _ = UnhookWindowsHookEx(state.mouse_hook);
                }
            }
            *state.borrow_mut() = None;
        });
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = info.vkCode as i32;
        let is_key_down = wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;
        let is_key_up = wparam.0 as u32 == WM_KEYUP || wparam.0 as u32 == WM_SYSKEYUP;

        if is_key_down {
            let modifiers = get_current_modifiers();
            let key_str = vk_to_string(vk);

            if let Some(key_str) = key_str {
                let hotkey_str = format_hotkey(&modifiers, &key_str);

                HOOK_STATE.with(|state| {
                    if let Some(ref mut state) = *state.borrow_mut() {
                        if state.pressed_keys.insert(vk)
                            && state.bindings.lock().unwrap().contains_key(&hotkey_str)
                        {
                            let _ = state.app_handle.emit("hotkey:triggered", &hotkey_str);
                        }
                    }
                });
            }
        } else if is_key_up {
            HOOK_STATE.with(|state| {
                if let Some(ref mut state) = *state.borrow_mut() {
                    state.pressed_keys.remove(&vk);
                }
            });
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let msg = wparam.0 as u32;
        let is_button_down = msg == WM_LBUTTONDOWN
            || msg == WM_RBUTTONDOWN
            || msg == WM_MBUTTONDOWN
            || msg == WM_XBUTTONDOWN;

        if is_button_down {
            let key_str = match msg {
                WM_MBUTTONDOWN => Some("Middle".to_string()),
                WM_XBUTTONDOWN => {
                    let info = *(lparam.0 as *const MSLLHOOKSTRUCT);
                    let btn = (info.mouseData >> 16) as u16;
                    if btn == 1 {
                        Some("XButton1".to_string())
                    } else if btn == 2 {
                        Some("XButton2".to_string())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(key_str) = key_str {
                let modifiers = get_current_modifiers();
                let hotkey_str = format_hotkey(&modifiers, &key_str);

                HOOK_STATE.with(|state| {
                    if let Some(ref state) = *state.borrow() {
                        if state.bindings.lock().unwrap().contains_key(&hotkey_str) {
                            let _ = state.app_handle.emit("hotkey:triggered", &hotkey_str);
                        }
                    }
                });
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

fn get_current_modifiers() -> Vec<String> {
    let mut mods = Vec::new();
    unsafe {
        if GetAsyncKeyState(VK_CONTROL.0 as i32) < 0
            || GetAsyncKeyState(VK_LCONTROL.0 as i32) < 0
            || GetAsyncKeyState(VK_RCONTROL.0 as i32) < 0
        {
            mods.push("Ctrl".to_string());
        }
        if GetAsyncKeyState(VK_MENU.0 as i32) < 0
            || GetAsyncKeyState(VK_LMENU.0 as i32) < 0
            || GetAsyncKeyState(VK_RMENU.0 as i32) < 0
        {
            mods.push("Alt".to_string());
        }
        if GetAsyncKeyState(VK_SHIFT.0 as i32) < 0
            || GetAsyncKeyState(VK_LSHIFT.0 as i32) < 0
            || GetAsyncKeyState(VK_RSHIFT.0 as i32) < 0
        {
            mods.push("Shift".to_string());
        }
    }
    mods
}

fn format_hotkey(modifiers: &[String], key: &str) -> String {
    if modifiers.is_empty() {
        key.to_string()
    } else {
        format!("{}+{}", modifiers.join("+"), key)
    }
}

fn vk_to_string(vk: i32) -> Option<String> {
    match vk {
        0x41..=0x5A => Some((b'A' + (vk - 0x41) as u8) as char).map(|c| c.to_string()),
        0x30..=0x39 => Some((b'0' + (vk - 0x30) as u8) as char).map(|c| c.to_string()),
        v if v == VK_F1.0 as i32 => Some("F1".to_string()),
        v if v == VK_F1.0 as i32 + 1 => Some("F2".to_string()),
        v if v == VK_F1.0 as i32 + 2 => Some("F3".to_string()),
        v if v == VK_F1.0 as i32 + 3 => Some("F4".to_string()),
        v if v == VK_F1.0 as i32 + 4 => Some("F5".to_string()),
        v if v == VK_F1.0 as i32 + 5 => Some("F6".to_string()),
        v if v == VK_F1.0 as i32 + 6 => Some("F7".to_string()),
        v if v == VK_F1.0 as i32 + 7 => Some("F8".to_string()),
        v if v == VK_F1.0 as i32 + 8 => Some("F9".to_string()),
        v if v == VK_F1.0 as i32 + 9 => Some("F10".to_string()),
        v if v == VK_F1.0 as i32 + 10 => Some("F11".to_string()),
        v if v == VK_F1.0 as i32 + 11 => Some("F12".to_string()),
        v if v == VK_SPACE.0 as i32 => Some("Space".to_string()),
        v if v == VK_RETURN.0 as i32 => Some("Enter".to_string()),
        v if v == VK_ESCAPE.0 as i32 => Some("Esc".to_string()),
        v if v == VK_BACK.0 as i32 => Some("Backspace".to_string()),
        v if v == VK_TAB.0 as i32 => Some("Tab".to_string()),
        v if v == VK_LEFT.0 as i32 => Some("Left".to_string()),
        v if v == VK_RIGHT.0 as i32 => Some("Right".to_string()),
        v if v == VK_UP.0 as i32 => Some("Up".to_string()),
        v if v == VK_DOWN.0 as i32 => Some("Down".to_string()),
        v if v == VK_INSERT.0 as i32 => Some("Insert".to_string()),
        v if v == VK_DELETE.0 as i32 => Some("Delete".to_string()),
        v if v == VK_HOME.0 as i32 => Some("Home".to_string()),
        v if v == VK_END.0 as i32 => Some("End".to_string()),
        v if v == VK_PRIOR.0 as i32 => Some("PageUp".to_string()),
        v if v == VK_NEXT.0 as i32 => Some("PageDown".to_string()),
        _ => None,
    }
}

use windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN;
