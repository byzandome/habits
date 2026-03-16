use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static ICON_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, String>> {
    ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Extracts the icon directly from a known exe path and returns it as a
/// `"data:image/png;base64,..."` string.
/// Returns an empty string on any failure or if `exe_path` is empty.
/// Results are cached in memory for the lifetime of the process.
pub fn get_icon_base64_from_path(exe_path: &str) -> String {
    if exe_path.is_empty() { return String::new(); }

    if let Some(hit) = cache().lock().unwrap().get(exe_path).cloned() {
        return hit;
    }

    let result = extract_icon_base64(exe_path).unwrap_or_default();
    cache().lock().unwrap().insert(exe_path.to_string(), result.clone());
    result
}

/// Clears the in-process icon cache, forcing icons to be re-extracted on the
/// next request.
pub fn clear_in_memory_cache() {
    cache().lock().unwrap().clear();
}

// ── Icon extraction ───────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn extract_icon_base64(exe_path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::mem::{size_of, zeroed};
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::shellapi::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use winapi::um::wingdi::DeleteObject;
    use winapi::um::winuser::{DestroyIcon, GetIconInfo, ICONINFO};

    let wide: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // ── 1. Get HICON from the exe via SHGetFileInfoW ──────────────────────
        let mut shfi: SHFILEINFOW = zeroed();
        let r = SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &mut shfi,
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if r == 0 || shfi.hIcon.is_null() {
            return None;
        }
        let hicon = shfi.hIcon;

        // ── 2. Decompose HICON into bitmap handles ────────────────────────────
        let mut icon_info: ICONINFO = zeroed();
        let got_info = GetIconInfo(hicon, &mut icon_info) != 0;

        // Extract pixel data, then clean up regardless of success/failure.
        let result = if got_info && !icon_info.hbmColor.is_null() {
            do_extract(icon_info.hbmColor)
        } else {
            None
        };

        // Cleanup – null checks protect against partially-initialised structs
        DestroyIcon(hicon);
        if got_info {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor as *mut _);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask as *mut _);
            }
        }

        result
    }
}

#[cfg(target_os = "windows")]
unsafe fn do_extract(hbm_color: winapi::shared::windef::HBITMAP) -> Option<String> {
    use std::mem::{size_of, zeroed};

    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, DIB_RGB_COLORS, BI_RGB,
    };

    // ── Get bitmap dimensions ──────────────────────────────────────────────
    let mut bmp: BITMAP = zeroed();
    if GetObjectW(
        hbm_color as *mut _,
        size_of::<BITMAP>() as i32,
        &mut bmp as *mut _ as *mut _,
    ) == 0
    {
        return None;
    }

    let width = bmp.bmWidth.unsigned_abs();
    let height = bmp.bmHeight.unsigned_abs();
    if width == 0 || height == 0 {
        return None;
    }

    // ── Pull raw BGRA pixels via GetDIBits ─────────────────────────────────
    let dc = CreateCompatibleDC(std::ptr::null_mut());
    if dc.is_null() {
        return None;
    }

    let mut bmi: BITMAPINFO = zeroed();
    bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32); // negative → top-down scanlines
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let mut pixels = vec![0u8; (width * height * 4) as usize];
    let lines = GetDIBits(
        dc,
        hbm_color,
        0,
        height,
        pixels.as_mut_ptr() as *mut _,
        &mut bmi,
        DIB_RGB_COLORS,
    );
    DeleteDC(dc);

    if lines == 0 {
        return None;
    }

    // ── BGRA → RGBA + alpha fixup for old-style (opaque) icons ────────────
    let has_alpha = pixels.chunks_exact(4).any(|p| p[3] != 0);
    for px in pixels.chunks_exact_mut(4) {
        px.swap(0, 2); // B ↔ R
        if !has_alpha {
            px[3] = 255; // legacy icon: force fully opaque
        }
    }

    // ── PNG encode ─────────────────────────────────────────────────────────
    let mut png_bytes: Vec<u8> = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut png_bytes, width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }

    // ── Base64 encode ──────────────────────────────────────────────────────
    use base64::Engine as _;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Some(format!("data:image/png;base64,{}", b64))
}

#[cfg(not(target_os = "windows"))]
fn extract_icon_base64(_exe_path: &str) -> Option<String> {
    None
}
