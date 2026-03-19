use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use base64::Engine as _;

// ── Windows icon extraction ───────────────────────────────────────────────────

/// Extracts the icon from an executable as RGBA pixels (32×32).
/// Returns `(rgba_bytes, width, height)` or `None` on failure.
#[cfg(target_os = "windows")]
pub fn extract_icon_rgba(exe_path: &str) -> Option<(Vec<u8>, u32, u32)> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use winapi::um::shellapi::ExtractIconExW;
    use winapi::um::wingdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use winapi::um::winuser::{DestroyIcon, GetIconInfo, ICONINFO};

    const SIZE: u32 = 32;

    let wide: Vec<u16> = OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut large_icon = ptr::null_mut();
        let count = ExtractIconExW(wide.as_ptr(), 0, &mut large_icon, ptr::null_mut(), 1);
        if count == 0 || large_icon.is_null() {
            return None;
        }

        let mut icon_info: ICONINFO = std::mem::zeroed();
        if GetIconInfo(large_icon, &mut icon_info) == 0 {
            DestroyIcon(large_icon);
            return None;
        }

        let hdc = CreateCompatibleDC(ptr::null_mut());
        if hdc.is_null() {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor as _);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask as _);
            }
            DestroyIcon(large_icon);
            return None;
        }

        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = SIZE as i32;
        bmi.bmiHeader.biHeight = -(SIZE as i32); // top-down
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let bmp = if !icon_info.hbmColor.is_null() {
            icon_info.hbmColor
        } else {
            icon_info.hbmMask
        };

        let old = SelectObject(hdc, bmp as _);

        let mut buf = vec![0u8; (SIZE * SIZE * 4) as usize];
        let rows = GetDIBits(
            hdc,
            bmp,
            0,
            SIZE,
            buf.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc, old);
        DeleteDC(hdc);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor as _);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask as _);
        }
        DestroyIcon(large_icon);

        if rows == 0 {
            return None;
        }

        // GDI returns BGRA — convert to RGBA
        for pixel in buf.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        Some((buf, SIZE, SIZE))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn extract_icon_rgba(_exe_path: &str) -> Option<(Vec<u8>, u32, u32)> {
    None
}

// ── Dominant colour extraction ────────────────────────────────────────────────

/// Analyse RGBA pixels and return the dominant non-white, non-black colour as
/// `"R,G,B"`.  Mirrors the frontend `extractDominantColor` algorithm.
pub fn extract_dominant_color(rgba: &[u8]) -> Option<String> {
    let mut counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

    for pixel in rgba.chunks_exact(4) {
        let (r, g, b, a) = (pixel[0], pixel[1], pixel[2], pixel[3]);
        // Skip transparent
        if a < 128 {
            continue;
        }
        // Skip near-white
        if r > 220 && g > 220 && b > 220 {
            continue;
        }
        // Skip near-black
        if r < 30 && g < 30 && b < 30 {
            continue;
        }
        // Quantise to 32-step grid
        let key = (
            (r / 32) * 32,
            (g / 32) * 32,
            (b / 32) * 32,
        );
        *counts.entry(key).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|((r, g, b), _)| format!("{r},{g},{b}"))
}

// ── PNG persistence ───────────────────────────────────────────────────────────

fn icon_path(icons_dir: &Path, app_name: &str) -> PathBuf {
    icons_dir.join(format!("{app_name}.png"))
}

/// Encode RGBA pixels as a PNG file and write it to `{icons_dir}/{app_name}.png`.
pub fn save_icon_png(
    icons_dir: &Path,
    app_name: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<PathBuf, String> {
    let path = icon_path(icons_dir, app_name);
    let file = fs::File::create(&path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(rgba).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Read a cached PNG from disk and return it as a `data:image/png;base64,...` URI.
pub fn read_icon_as_data_uri(icons_dir: &Path, app_name: &str) -> Option<String> {
    let path = icon_path(icons_dir, app_name);
    let bytes = fs::read(&path).ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:image/png;base64,{b64}"))
}

// ── Public orchestrator ───────────────────────────────────────────────────────

/// Ensures an icon is cached on disk for `app_name`.
///
/// Returns `(data_uri, dominant_color)` — either or both may be `None` when the
/// icon cannot be extracted from the executable.
pub fn ensure_icon_cached(
    icons_dir: &Path,
    app_name: &str,
    exe_path: &str,
) -> (Option<String>, Option<String>) {
    // Fast path: icon already on disk
    if icon_path(icons_dir, app_name).exists() {
        let uri = read_icon_as_data_uri(icons_dir, app_name);
        // Re-derive colour from the on-disk PNG pixels for the caller.
        let color = read_icon_rgba_from_png(icons_dir, app_name)
            .and_then(|(rgba, _, _)| extract_dominant_color(&rgba));
        return (uri, color);
    }

    // Extract icon from the executable
    let (rgba, w, h) = match extract_icon_rgba(exe_path) {
        Some(data) => data,
        None => return (None, None),
    };

    let color = extract_dominant_color(&rgba);

    let uri = match save_icon_png(icons_dir, app_name, &rgba, w, h) {
        Ok(_) => read_icon_as_data_uri(icons_dir, app_name),
        Err(_) => {
            // Encode directly without going through disk
            let mut cursor = Cursor::new(Vec::new());
            let mut encoder = png::Encoder::new(&mut cursor, w, h);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            if let Ok(mut writer) = encoder.write_header() {
                let _ = writer.write_image_data(&rgba);
            }
            let b64 = base64::engine::general_purpose::STANDARD.encode(cursor.into_inner());
            Some(format!("data:image/png;base64,{b64}"))
        }
    };

    (uri, color)
}

/// Decode a cached PNG back to RGBA pixels (used to derive colour from cache).
fn read_icon_rgba_from_png(icons_dir: &Path, app_name: &str) -> Option<(Vec<u8>, u32, u32)> {
    let path = icon_path(icons_dir, app_name);
    let bytes = fs::read(&path).ok()?;
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let width = reader.info().width;
    let height = reader.info().height;
    let color_type = reader.info().color_type;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).ok()?;

    let rgba = match color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255u8])
            .collect(),
        _ => return None,
    };

    Some((rgba, width, height))
}

// ── Cache management ──────────────────────────────────────────────────────────

/// Deletes all PNG files in the icons cache directory.
pub fn clear_cache(icons_dir: &Path) -> Result<(), String> {
    if !icons_dir.exists() {
        return Ok(());
    }
    let entries = fs::read_dir(icons_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("png") {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}
