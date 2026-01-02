use crate::Region;
use image::{Rgb, RgbImage};
use imageproc::distance_transform::Norm;
use imageproc::edges::canny;
use imageproc::morphology::dilate;
use imageproc::region_labelling::{connected_components, Connectivity};

/// 浮水印偵測器
pub struct WatermarkDetector {
    sensitivity: f32,
    hue_min: f32,
    hue_max: f32,
    sat_min: f32,
    val_min: f32,
}

impl WatermarkDetector {
    pub fn new(sensitivity: f32, hue_min: f32, hue_max: f32, sat_min: f32, val_min: f32) -> Self {
        Self {
            sensitivity,
            hue_min,
            hue_max,
            sat_min,
            val_min,
        }
    }

    /// 偵測圖片中的浮水印區域
    pub fn detect(&self, image: &RgbImage) -> Vec<Region> {
        // 1. 轉換為灰階
        let gray = image::imageops::grayscale(image);

        // 2. 偵測特定顏色/異常顏色區域
        let color_mask = self.detect_color_regions(image);

        // 3. Canny 邊緣檢測
        let low_threshold = (50.0 * self.sensitivity) as f32;
        let high_threshold = (150.0 * self.sensitivity) as f32;
        let edges = canny(&gray, low_threshold, high_threshold);

        // 4. 結合顏色遮罩與邊緣
        let combined = self.combine_masks(&color_mask, &edges);

        // 5. 形態學運算 - 膨脹以連接斷開的區域
        let dilated = dilate(&combined, Norm::LInf, 3);

        // 6. 連通區域分析
        let regions = self.find_regions(&dilated, image.width(), image.height());

        regions
    }

    /// 偵測特定顏色區域 (HSV 色彩空間)
    fn detect_color_regions(&self, image: &RgbImage) -> image::GrayImage {
        let (width, height) = image.dimensions();
        let mut mask = image::GrayImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let hsv = rgb_to_hsv(pixel);

                // 顏色範圍判斷
                let is_target_color = hsv.0 >= self.hue_min
                    && hsv.0 <= self.hue_max
                    && hsv.1 >= self.sat_min
                    && hsv.2 >= self.val_min;

                if is_target_color {
                    mask.put_pixel(x, y, image::Luma([255]));
                }
            }
        }

        mask
    }

    /// 結合兩個遮罩
    fn combine_masks(
        &self,
        mask1: &image::GrayImage,
        mask2: &image::GrayImage,
    ) -> image::GrayImage {
        let (width, height) = mask1.dimensions();
        let mut combined = image::GrayImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let v1 = mask1.get_pixel(x, y).0[0];
                let v2 = mask2.get_pixel(x, y).0[0];
                let value = if v1 > 0 || v2 > 0 { 255 } else { 0 };
                combined.put_pixel(x, y, image::Luma([value]));
            }
        }

        combined
    }

    /// 從二值圖中找出連通區域
    fn find_regions(
        &self,
        mask: &image::GrayImage,
        img_width: u32,
        img_height: u32,
    ) -> Vec<Region> {
        let components = connected_components(mask, Connectivity::Eight, image::Luma([0u8]));
        let mut regions = Vec::new();

        // 統計每個標籤的邊界框
        let mut label_bounds: std::collections::HashMap<u32, (u32, u32, u32, u32)> =
            std::collections::HashMap::new();

        for y in 0..components.height() {
            for x in 0..components.width() {
                let label = components.get_pixel(x, y).0[0];
                if label == 0 {
                    continue; // 背景
                }

                label_bounds
                    .entry(label as u32)
                    .and_modify(|(min_x, min_y, max_x, max_y)| {
                        *min_x = (*min_x).min(x);
                        *min_y = (*min_y).min(y);
                        *max_x = (*max_x).max(x);
                        *max_y = (*max_y).max(y);
                    })
                    .or_insert((x, y, x, y));
            }
        }

        // 過濾太小的區域
        let min_area = (img_width * img_height) as f32 * 0.001 * self.sensitivity; // 至少 0.1% 圖片大小

        for (_label, (min_x, min_y, max_x, max_y)) in label_bounds {
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;
            let area = width * height;

            if area as f32 >= min_area {
                regions.push(Region {
                    x: min_x,
                    y: min_y,
                    width,
                    height,
                });
            }
        }

        regions
    }
}

/// RGB 轉 HSV
fn rgb_to_hsv(rgb: &Rgb<u8>) -> (f32, f32, f32) {
    let r = rgb.0[0] as f32 / 255.0;
    let g = rgb.0[1] as f32 / 255.0;
    let b = rgb.0[2] as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // 色相 (Hue)
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    // 飽和度 (Saturation)
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // 亮度 (Value)
    let v = max;

    (h, s, v)
}
