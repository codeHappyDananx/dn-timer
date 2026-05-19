#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HotkeySpec {
    pub modifiers: Vec<Modifier>,
    pub key: HotkeyKey,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyKey {
    Char(char),
    F(u8),
    Enter,
    Space,
    Esc,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Middle,
    XButton1,
    XButton2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    NextPreset,
    StartPauseToggle,
    Start,
    Pause,
    Reset,
    SlotToggle(usize),
    SlotReset(usize),
}

pub type HotkeyBindings = HashMap<String, String>;
