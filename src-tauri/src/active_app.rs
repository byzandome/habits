/// Foreground-window app info returned by [`get_active_app`].
pub struct ActiveApp {
    /// Executable file-stem, e.g. `"chrome"`, `"ms-teams"`.
    pub name: String,
    /// Full path to the executable, e.g. `"C:\...\chrome.exe"`.
    /// Empty string when the path could not be determined.
    pub exe_path: String,
}

impl ActiveApp {
    fn unknown() -> Self {
        ActiveApp { name: "Unknown".to_string(), exe_path: String::new() }
    }
}

/// Returns the name (file-stem) and full exe path of the foreground window's
/// process.  Falls back to `"Unknown"` / empty path on any error or on
/// non-Windows platforms.
#[cfg(target_os = "windows")]
pub fn get_active_app() -> ActiveApp {
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
            return ActiveApp::unknown();
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return ActiveApp::unknown();
        }

        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return ActiveApp::unknown();
        }

        let mut buf = vec![0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(handle);

        if ok == 0 {
            return ActiveApp::unknown();
        }

        let os_str = OsString::from_wide(&buf[..size as usize]);
        let path = Path::new(&os_str);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let exe_path = path.to_str().unwrap_or("").to_string();

        ActiveApp { name, exe_path }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_active_app() -> ActiveApp {
    ActiveApp::unknown()
}
