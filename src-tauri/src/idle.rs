/// Returns the number of seconds since the last user input (keyboard/mouse).
/// On non-Windows platforms returns 0 (no idle detection).
#[cfg(target_os = "windows")]
pub fn get_idle_seconds() -> u64 {
    use winapi::um::sysinfoapi::GetTickCount;
    use winapi::um::winuser::{GetLastInputInfo, LASTINPUTINFO};

    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut lii) != 0 {
            let idle_ms = GetTickCount().wrapping_sub(lii.dwTime);
            u64::from(idle_ms / 1000)
        } else {
            0
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_idle_seconds() -> u64 {
    0
}
