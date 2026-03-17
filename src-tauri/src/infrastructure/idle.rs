#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

// winapi 0.3 does not expose WTSRegisterSessionNotification, so we declare
// the subset of the WTS API we need directly.
#[cfg(target_os = "windows")]
#[link(name = "wtsapi32")]
extern "system" {
    fn WTSRegisterSessionNotification(
        hWnd: winapi::shared::windef::HWND,
        dwFlags: winapi::shared::minwindef::DWORD,
    ) -> winapi::shared::minwindef::BOOL;
}

#[cfg(target_os = "windows")]
const NOTIFY_FOR_THIS_SESSION: winapi::shared::minwindef::DWORD = 0;

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

// ── Session lock detection ────────────────────────────────────────────────────

/// Set to `true` when `WTS_SESSION_LOCK` arrives; `false` on `WTS_SESSION_UNLOCK`.
/// Updated exclusively by the thread spawned in `start_session_monitor()`.
#[cfg(target_os = "windows")]
static SESSION_LOCKED: AtomicBool = AtomicBool::new(false);

/// When set, poked on every lock/unlock event so the tracker loop wakes
/// immediately instead of waiting up to 5 s.
static TRACKER_WAKE: OnceLock<Arc<tokio::sync::Notify>> = OnceLock::new();

/// Call once at startup (before `start_session_monitor`) to hand the WTS
/// thread a way to poke the async tracker loop immediately.
pub fn set_tracker_wake(notify: Arc<tokio::sync::Notify>) {
    let _ = TRACKER_WAKE.set(notify);
}

/// Returns `true` when the current Windows session is locked.
///
/// Accurate only after `start_session_monitor()` has been called once.
pub fn is_session_locked() -> bool {
    #[cfg(target_os = "windows")]
    {
        SESSION_LOCKED.load(Ordering::Relaxed)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Spawns a background Win32 message-loop thread that listens for WTS
/// session-change events (`Win+L`, lock screen) and keeps `SESSION_LOCKED`
/// up to date.
///
/// Must be called once during application startup.
pub fn start_session_monitor() {
    #[cfg(target_os = "windows")]
    {
        std::thread::Builder::new()
            .name("wts-session-monitor".into())
            .spawn(|| unsafe { run_session_monitor() })
            .ok();
    }
}

/// Creates a message-only (invisible) window, registers for WTS session
/// notifications, then runs a Win32 message loop until the process exits.
#[cfg(target_os = "windows")]
unsafe fn run_session_monitor() {
    use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winapi::um::winuser::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassExW,
        TranslateMessage, MSG, WNDCLASSEXW,
    };

    const WM_WTSSESSION_CHANGE: UINT = 0x02B1;
    const WTS_SESSION_LOCK: WPARAM = 7;
    const WTS_SESSION_UNLOCK: WPARAM = 8;

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if msg == WM_WTSSESSION_CHANGE {
            match wparam {
                WTS_SESSION_LOCK => {
                    SESSION_LOCKED.store(true, Ordering::Relaxed);
                    if let Some(w) = TRACKER_WAKE.get() {
                        w.notify_one();
                    }
                }
                WTS_SESSION_UNLOCK => {
                    SESSION_LOCKED.store(false, Ordering::Relaxed);
                    if let Some(w) = TRACKER_WAKE.get() {
                        w.notify_one();
                    }
                }
                _ => {}
            }
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    let hmod = GetModuleHandleW(std::ptr::null());
    let class_name: Vec<u16> = "HabitsSessionMonitor\0".encode_utf16().collect();

    let mut wc: WNDCLASSEXW = std::mem::zeroed();
    wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
    wc.lpfnWndProc = Some(wnd_proc);
    wc.hInstance = hmod;
    wc.lpszClassName = class_name.as_ptr();
    RegisterClassExW(&wc);

    // HWND_MESSAGE (-3isize) creates a message-only window.
    let hwnd = CreateWindowExW(
        0,
        class_name.as_ptr(),
        class_name.as_ptr(),
        0,
        0,
        0,
        0,
        0,
        (-3isize) as HWND, // HWND_MESSAGE
        std::ptr::null_mut(),
        hmod,
        std::ptr::null_mut(),
    );

    if hwnd.is_null() {
        return;
    }

    if WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION) == 0 {
        return;
    }

    let mut msg: MSG = std::mem::zeroed();
    while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) != 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }
}
