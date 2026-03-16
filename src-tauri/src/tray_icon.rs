/// Generates a 16×16 RGBA image with a solid filled circle of the given colour.
/// The background is fully transparent; a thin 1-px anti-aliased edge is added
/// so the dot looks sharp at small tray sizes.
pub fn make_circle_icon(r: u8, g: u8, b: u8) -> tauri::image::Image<'static> {
    const SIZE: u32 = 16;
    const CENTER: f32 = 7.5;
    const RADIUS: f32 = 6.5;

    let mut rgba: Vec<u8> = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - CENTER;
            let dy = y as f32 - CENTER;
            let dist = (dx * dx + dy * dy).sqrt();
            // Smooth edge: full alpha inside, fading just at the boundary.
            let alpha = ((RADIUS - dist + 1.0).clamp(0.0, 1.0) * 255.0) as u8;
            rgba.extend_from_slice(&[r, g, b, alpha]);
        }
    }
    tauri::image::Image::new_owned(rgba, SIZE, SIZE)
}

/// Green circle (Tailwind green-500) — shown while the user is productive.
pub fn productive_icon() -> tauri::image::Image<'static> {
    make_circle_icon(34, 197, 94)
}

/// Yellow circle (Tailwind yellow-500) — shown while the tracker is idle / suspended.
pub fn idle_icon() -> tauri::image::Image<'static> {
    make_circle_icon(234, 179, 8)
}
