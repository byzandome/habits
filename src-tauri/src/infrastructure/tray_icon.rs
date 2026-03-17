// Path is relative to this file: src/infrastructure/ → src-tauri/icons/
static APP_ICON_BYTES: &[u8] = include_bytes!("../../icons/32x32.png");

/// Decodes the embedded 32×32 app icon PNG into RGBA pixels.
fn load_app_icon_rgba() -> (Vec<u8>, u32, u32) {
    use std::io::Cursor;
    let decoder = png::Decoder::new(Cursor::new(APP_ICON_BYTES));
    let mut reader = decoder.read_info().expect("valid app icon PNG");
    let width;
    let height;
    let color_type;
    {
        let info = reader.info();
        width = info.width;
        height = info.height;
        color_type = info.color_type;
    }
    let buf_size = reader.output_buffer_size();
    let mut buf = vec![0u8; buf_size];
    reader.next_frame(&mut buf).expect("valid PNG frame");

    let rgba = match color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255u8])
            .collect(),
        _ => vec![0u8; (width * height * 4) as usize],
    };

    (rgba, width, height)
}

/// Composites a small solid-colour badge circle at the bottom-right corner of
/// the app icon to indicate the current tracker status.
pub fn make_tray_icon_with_badge(r: u8, g: u8, b: u8) -> tauri::image::Image<'static> {
    let (mut pixels, width, height) = load_app_icon_rgba();

    let badge_radius: f32 = 8.0;
    let border_width: f32 = 2.0;
    let outer_radius = badge_radius + border_width;
    let badge_cx = width as f32 - outer_radius - 1.0;
    let badge_cy = height as f32 - outer_radius - 1.0;

    for py in 0..height {
        for px in 0..width {
            let dx = px as f32 - badge_cx;
            let dy = py as f32 - badge_cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > outer_radius + 1.0 {
                continue;
            }

            let idx = ((py * width + px) * 4) as usize;
            let ea = pixels[idx + 3] as f32 / 255.0;

            if dist > badge_radius - 0.5 {
                let border_alpha = ((outer_radius - dist + 1.0).clamp(0.0, 1.0) * 255.0) as u8;
                if border_alpha == 0 {
                    continue;
                }
                let ba = border_alpha as f32 / 255.0;
                let out_a = ba + ea * (1.0 - ba);
                if out_a > 0.0 {
                    pixels[idx] =
                        ((255.0_f32 * ba + pixels[idx] as f32 * ea * (1.0 - ba)) / out_a) as u8;
                    pixels[idx + 1] = ((255.0_f32 * ba + pixels[idx + 1] as f32 * ea * (1.0 - ba))
                        / out_a) as u8;
                    pixels[idx + 2] = ((255.0_f32 * ba + pixels[idx + 2] as f32 * ea * (1.0 - ba))
                        / out_a) as u8;
                    pixels[idx + 3] = (out_a * 255.0) as u8;
                }
            } else {
                let badge_alpha = ((badge_radius - dist + 1.0).clamp(0.0, 1.0) * 255.0) as u8;
                if badge_alpha == 0 {
                    continue;
                }
                let ba = badge_alpha as f32 / 255.0;
                let out_a = ba + ea * (1.0 - ba);
                if out_a > 0.0 {
                    pixels[idx] =
                        ((r as f32 * ba + pixels[idx] as f32 * ea * (1.0 - ba)) / out_a) as u8;
                    pixels[idx + 1] = ((g as f32 * ba + pixels[idx + 1] as f32 * ea * (1.0 - ba))
                        / out_a) as u8;
                    pixels[idx + 2] = ((b as f32 * ba + pixels[idx + 2] as f32 * ea * (1.0 - ba))
                        / out_a) as u8;
                    pixels[idx + 3] = (out_a * 255.0) as u8;
                }
            }
        }
    }

    tauri::image::Image::new_owned(pixels, width, height)
}

/// App icon + green badge (Tailwind green-500) — shown while the user is productive.
pub fn productive_icon() -> tauri::image::Image<'static> {
    make_tray_icon_with_badge(34, 197, 94)
}

/// App icon + yellow badge (Tailwind yellow-500) — shown while idle / suspended.
pub fn idle_icon() -> tauri::image::Image<'static> {
    make_tray_icon_with_badge(234, 179, 8)
}
