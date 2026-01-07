use anyhow::{anyhow, Result};
// use image::{Rgba, RgbaImage};
use tray_icon::Icon;

pub fn load_icon() -> Result<Icon> {
    const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

    let image = image::load_from_memory(ICON_BYTES)
        .map_err(|e| anyhow!("Failed to load icon image: {}", e))?
        .to_rgba8();

    let (width, height) = image.dimensions();
    let icon_rgba = image.into_raw();

    Icon::from_rgba(icon_rgba, width, height)
        .map_err(|e| anyhow!("Failed to create tray icon: {}", e))
}
