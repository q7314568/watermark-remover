mod detector;
mod remover;

use base64::{engine::general_purpose, Engine as _};
use detector::WatermarkDetector;
use remover::{InpaintMethod, WatermarkRemover};
use std::path::Path;
use tauri::Emitter;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(serde::Serialize)]
struct ImageInfo {
    width: u32,
    height: u32,
    preview: String, // Base64 representation
}

// Load image and return base64 preview + dimensions
#[tauri::command]
fn load_image(path: String) -> Result<ImageInfo, String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgb = img.to_rgb8();

    // Store image buffer? For now just return preview.
    // In a real app we might want to cache the loaded image in State to avoid reloading.
    // For simplicity, we'll reload/process from disk or keep it simple.

    // Convert to base64 for display
    let mut buffer = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;

    let encoded = general_purpose::STANDARD.encode(buffer.into_inner());
    let preview = format!("data:image/png;base64,{}", encoded);

    Ok(ImageInfo {
        width: rgb.width(),
        height: rgb.height(),
        preview,
    })
}

#[tauri::command]
fn remove_watermark(path: String, output_path: String, region: Region) -> Result<String, String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgb_img = img.to_rgb8();

    let method = InpaintMethod::Gaussian;
    let remover = WatermarkRemover::new(method);

    // Convert crate::Region to internal logic if needed, but we used same struct name/fields.
    // However the modules expect crate::cli::Region or crate::Region.
    // We will update modules to use crate::Region.

    // The remover expects &[Region].
    let regions = vec![region];
    let result = remover.remove(&rgb_img, &regions);

    result.save(&output_path).map_err(|e| e.to_string())?;

    // Return base64 of result for immediate display
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(result)
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|e| e.to_string())?;

    let encoded = general_purpose::STANDARD.encode(buffer.into_inner());
    Ok(format!("data:image/png;base64,{}", encoded))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![load_image, remove_watermark])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
