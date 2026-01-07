use anyhow::{anyhow, Result};
use image::{Rgba, RgbaImage};
use tray_icon::Icon;

pub fn load_icon() -> Result<Icon> {
    // Generate a simple blue square icon
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 64;

    let mut image = RgbaImage::new(WIDTH, HEIGHT);

    // 1Panel color #0052cc or similar blue
    let color = Rgba([0, 82, 204, 255]);

    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            // Circle shape for cleaner look
            let dx = (x as i32) - (WIDTH as i32 / 2);
            let dy = (y as i32) - (HEIGHT as i32 / 2);
            if dx * dx + dy * dy <= ((WIDTH / 2 - 2) * (WIDTH / 2 - 2)) as i32 {
                image.put_pixel(x, y, color);
            }
        }
    }

    let (icon_rgba, icon_width, icon_height) = (image.into_raw(), WIDTH, HEIGHT);

    Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .map_err(|e| anyhow!("Failed to create icon: {}", e))
}
