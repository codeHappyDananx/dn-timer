use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

pub fn play_alert_sound(sound_key: &str) {
    let sound_file = match sound_key {
        "soft_chime" => "sounds/soft_chime.wav",
        "digital_ping" => "sounds/digital_ping.wav",
        "urgent_alarm" => "sounds/urgent_alarm.wav",
        "emergency_triple" => "sounds/emergency_triple.wav",
        "bell" => "sounds/bell.wav",
        _ => {
            println!("Unknown sound key: {}", sound_key);
            return;
        }
    };

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        let path = dir.join(sound_file);
        if path.exists() {
            // 文件存在但可能为空占位，PlaySoundW 对无效文件会静默失败（SND_NODEFAULT）
            play_wav_file(&path);
            return;
        }
    }

    // 文件不存在时静默返回，不报错
}

fn play_wav_file(path: &PathBuf) {
    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    unsafe {
        let _ = PlaySoundW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            None,
            SND_ASYNC | SND_FILENAME | SND_NODEFAULT,
        );
    }
}
