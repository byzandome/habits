/// Returns the executable file-stem of the foreground window's process
/// (e.g. `"chrome"`, `"Code"`, `"notepad"`).
/// Falls back to `"Unknown"` on any error or on non-Windows platforms.
#[cfg(target_os = "windows")]
pub fn get_active_app() -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::path::Path;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winbase::QueryFullProcessImageNameW;
    use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;
    use winapi::um::winuser::{GetForegroundWindow, GetWindowThreadProcessId};

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return "Unknown".to_string();
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return "Unknown".to_string();
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return "Unknown".to_string();
        }

        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(handle);

        if ok == 0 {
            return "Unknown".to_string();
        }

        let os_str = OsString::from_wide(&buf[..size as usize]);
        let path = Path::new(&os_str);
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_app() -> String {
    "Unknown".to_string()
}
