use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL,
    MOD_NOREPEAT, MOD_SHIFT, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1,
    VK_HOME, VK_INSERT, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_MENU, VK_NEXT, VK_PRIOR,
    VK_RCONTROL, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, PM_REMOVE, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_HOTKEY, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_RBUTTONDOWN,
    WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
};

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = RefCell::new(None);
}

struct HookState {
    keyboard_hook: HHOOK,
    mouse_hook: HHOOK,
    app_handle: AppHandle,
    bindings: Arc<Mutex<HashMap<String, String>>>,
    registered_keyboard_hotkeys: Arc<Mutex<HashSet<String>>>,
    pressed_keys: HashSet<i32>,
}

pub struct HotkeyManager {
    app_handle: AppHandle,
    bindings: Arc<Mutex<HashMap<String, String>>>,
    registered_keyboard_hotkeys: Arc<Mutex<HashSet<String>>>,
}

impl HotkeyManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            bindings: Arc::new(Mutex::new(HashMap::new())),
            registered_keyboard_hotkeys: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn update_bindings(&mut self, bindings: HashMap<String, String>) {
        let mut current = self.bindings.lock().unwrap();
        *current = bindings;
    }

    pub fn start(&self) {
        let app_handle = self.app_handle.clone();
        let bindings = Arc::clone(&self.bindings);
        let registered_keyboard_hotkeys = Arc::clone(&self.registered_keyboard_hotkeys);

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
                            app_handle: app_handle.clone(),
                            bindings,
                            registered_keyboard_hotkeys: Arc::clone(&registered_keyboard_hotkeys),
                            pressed_keys: HashSet::new(),
                        });
                    });

                    let mut registered = RegisteredHotkeys::new(app_handle.clone());
                    let mut last_sync = Instant::now() - Duration::from_millis(250);
                    let mut msg: MSG = std::mem::zeroed();
                    loop {
                        if last_sync.elapsed() >= Duration::from_millis(250) {
                            let keys = HOOK_STATE.with(|state| {
                                state
                                    .borrow()
                                    .as_ref()
                                    .map(|state| {
                                        state
                                            .bindings
                                            .lock()
                                            .unwrap()
                                            .keys()
                                            .cloned()
                                            .collect::<HashSet<_>>()
                                    })
                                    .unwrap_or_default()
                            });
                            registered.sync(&keys);
                            *registered_keyboard_hotkeys.lock().unwrap() =
                                registered.registered_keys();
                            last_sync = Instant::now();
                        }

                        while PeekMessageW(&mut msg, HWND(null_mut()), 0, 0, PM_REMOVE).into() {
                            if msg.message == WM_HOTKEY {
                                if let Some(hotkey) = registered.get(msg.wParam.0 as i32) {
                                    trigger_hotkey(&registered.app_handle, hotkey);
                                }
                            } else {
                                let _ = TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }

                        thread::sleep(Duration::from_millis(10));
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
                            && !state
                                .registered_keyboard_hotkeys
                                .lock()
                                .unwrap()
                                .contains(&hotkey_str)
                        {
                            trigger_hotkey(&state.app_handle, &hotkey_str);
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
                            trigger_hotkey(&state.app_handle, &hotkey_str);
                        }
                    }
                });
            }
        }
    }

    CallNextHookEx(None, code, wparam, lparam)
}

struct RegisteredHotkeys {
    app_handle: AppHandle,
    hotkeys_by_id: HashMap<i32, String>,
    ids_by_hotkey: HashMap<String, i32>,
    next_id: i32,
}

impl RegisteredHotkeys {
    fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            hotkeys_by_id: HashMap::new(),
            ids_by_hotkey: HashMap::new(),
            next_id: 1,
        }
    }

    fn sync(&mut self, keys: &HashSet<String>) {
        let active_keys = self.ids_by_hotkey.keys().cloned().collect::<HashSet<_>>();
        for key in active_keys.difference(keys) {
            if let Some(id) = self.ids_by_hotkey.remove(key) {
                unsafe {
                    let _ = UnregisterHotKey(HWND(null_mut()), id);
                }
                self.hotkeys_by_id.remove(&id);
            }
        }

        for key in keys {
            if self.ids_by_hotkey.contains_key(key) {
                continue;
            }
            let Some((modifiers, vk)) = parse_keyboard_hotkey(key) else {
                continue;
            };
            let id = self.next_id;
            self.next_id += 1;
            if unsafe { RegisterHotKey(HWND(null_mut()), id, modifiers, vk) }.is_ok() {
                self.ids_by_hotkey.insert(key.clone(), id);
                self.hotkeys_by_id.insert(id, key.clone());
            }
        }
    }

    fn get(&self, id: i32) -> Option<&str> {
        self.hotkeys_by_id.get(&id).map(String::as_str)
    }

    fn registered_keys(&self) -> HashSet<String> {
        self.ids_by_hotkey.keys().cloned().collect()
    }
}

impl Drop for RegisteredHotkeys {
    fn drop(&mut self) {
        for id in self.hotkeys_by_id.keys() {
            unsafe {
                let _ = UnregisterHotKey(HWND(null_mut()), *id);
            }
        }
    }
}

fn parse_keyboard_hotkey(hotkey: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let mut modifiers = MOD_NOREPEAT.0;
    let mut key = None;

    for part in hotkey.split('+') {
        match part {
            "Ctrl" => modifiers |= MOD_CONTROL.0,
            "Alt" => modifiers |= MOD_ALT.0,
            "Shift" => modifiers |= MOD_SHIFT.0,
            value => key = Some(value),
        }
    }

    key.and_then(key_to_vk)
        .map(|vk| (HOT_KEY_MODIFIERS(modifiers), vk as u32))
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

fn trigger_hotkey(app_handle: &AppHandle, hotkey: &str) {
    let Some(state) = app_handle.try_state::<crate::AppState>() else {
        return;
    };

    let bound_indices = {
        let slots = state.current_slots.lock().unwrap();
        slots
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| {
                if slot.hotkey.as_deref() == Some(hotkey) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };

    if bound_indices.is_empty() {
        return;
    }

    let engine = state.timer_engine.lock().unwrap();
    if bound_indices.len() == 1 {
        engine.send(crate::core::EngineCommand::ToggleSlot(bound_indices[0]));
    } else {
        engine.send(crate::core::EngineCommand::StartSlotsBatch(bound_indices));
    }
}

fn key_to_vk(key: &str) -> Option<i32> {
    if key.len() == 1 {
        let value = key.as_bytes()[0];
        if value.is_ascii_uppercase() || value.is_ascii_digit() {
            return Some(value as i32);
        }
    }

    match key {
        "F1" => Some(VK_F1.0 as i32),
        "F2" => Some(VK_F1.0 as i32 + 1),
        "F3" => Some(VK_F1.0 as i32 + 2),
        "F4" => Some(VK_F1.0 as i32 + 3),
        "F5" => Some(VK_F1.0 as i32 + 4),
        "F6" => Some(VK_F1.0 as i32 + 5),
        "F7" => Some(VK_F1.0 as i32 + 6),
        "F8" => Some(VK_F1.0 as i32 + 7),
        "F9" => Some(VK_F1.0 as i32 + 8),
        "F10" => Some(VK_F1.0 as i32 + 9),
        "F11" => Some(VK_F1.0 as i32 + 10),
        "F12" => Some(VK_F1.0 as i32 + 11),
        "Space" => Some(VK_SPACE.0 as i32),
        "Enter" => Some(VK_RETURN.0 as i32),
        "Esc" => Some(VK_ESCAPE.0 as i32),
        "Backspace" => Some(VK_BACK.0 as i32),
        "Tab" => Some(VK_TAB.0 as i32),
        "Left" => Some(VK_LEFT.0 as i32),
        "Right" => Some(VK_RIGHT.0 as i32),
        "Up" => Some(VK_UP.0 as i32),
        "Down" => Some(VK_DOWN.0 as i32),
        "Insert" => Some(VK_INSERT.0 as i32),
        "Delete" => Some(VK_DELETE.0 as i32),
        "Home" => Some(VK_HOME.0 as i32),
        "End" => Some(VK_END.0 as i32),
        "PageUp" => Some(VK_PRIOR.0 as i32),
        "PageDown" => Some(VK_NEXT.0 as i32),
        _ => None,
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
