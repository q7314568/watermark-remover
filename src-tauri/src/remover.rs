use crate::Region;
use image::{Rgb, RgbImage};
use imageproc::distance_transform::Norm;
use imageproc::morphology::dilate;

/// 浮水印移除器
pub struct WatermarkRemover {
    method: InpaintMethod,
}

#[derive(Debug, Clone, Copy)]
pub enum InpaintMethod {
    Simple,   // 簡單平均
    Gaussian, // 高斯加權
}

impl InpaintMethod {
    pub fn from_str(s: &str) -> Self {
        match s {
            "simple" => InpaintMethod::Simple,
            "gaussian" => InpaintMethod::Gaussian,
            _ => InpaintMethod::Gaussian,
        }
    }
}

impl WatermarkRemover {
    pub fn new(method: InpaintMethod) -> Self {
        Self { method }
    }

    /// 移除浮水印
    pub fn remove(&self, image: &RgbImage, regions: &[Region]) -> RgbImage {
        let (width, height) = image.dimensions();
        let mut result = image.clone();

        // 建立智慧遮罩 (Smart Masking)
        let mut mask = self.create_smart_mask(image, regions);

        // 建立信心度地圖 (Confidence Map)
        // 原始像素信心度為 1.0, 待修復區域初始為 0.0
        let mut confidence = vec![vec![1.0f32; height as usize]; width as usize];
        for y in 0..height {
            for x in 0..width {
                if mask.get_pixel(x, y).0[0] > 0 {
                    confidence[x as usize][y as usize] = 0.0;
                }
            }
        }

        // 執行 Inpainting (洋蔥剝皮法: 由外向內修復)
        let max_iterations = 500; // 增加迭代次數

        for _ in 0..max_iterations {
            if !self.inpaint_iteration(&mut result, &mut mask, &mut confidence) {
                break;
            }
        }

        result
    }

    /// 建立智慧遮罩 (Smart Masking)
    /// 透過亮度分析，只標記區域內的顯著差異像素 (浮水印文字)，保留背景
    fn create_smart_mask(&self, image: &RgbImage, regions: &[Region]) -> image::GrayImage {
        let (width, height) = image.dimensions();
        let mut mask = image::GrayImage::new(width, height);

        for region in regions {
            // 1. 收集該區域的亮度分佈
            let mut lumas = Vec::new();
            let start_x = region.x;
            let start_y = region.y;
            let end_x = (region.x + region.width).min(width);
            let end_y = (region.y + region.height).min(height);

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let pixel = image.get_pixel(x, y);
                    // Luma = 0.299*R + 0.587*G + 0.114*B
                    let luma = (0.299 * pixel.0[0] as f32
                        + 0.587 * pixel.0[1] as f32
                        + 0.114 * pixel.0[2] as f32) as u8;
                    lumas.push(luma);
                }
            }

            if lumas.is_empty() {
                continue;
            }

            // 2. 計算中位數 (作為背景基準)
            lumas.sort_unstable();
            let median_luma = lumas[lumas.len() / 2] as i32;

            // 3. 標記差異過大的像素 (Thresholding)
            let threshold = 30; // 亮度差異閾值

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let pixel = image.get_pixel(x, y);
                    let luma = (0.299 * pixel.0[0] as f32
                        + 0.587 * pixel.0[1] as f32
                        + 0.114 * pixel.0[2] as f32) as i32;

                    if (luma - median_luma).abs() > threshold {
                        mask.put_pixel(x, y, image::Luma([255]));
                    }
                }
            }
        }

        // 4. 形態學膨脹 (Dilation)
        // 擴大一點遮罩範圍，確保覆蓋到文字邊緣的抗鋸齒部分
        let dilated_mask = dilate(&mask, Norm::LInf, 1);

        // 只保留原始 Region 範圍內的膨脹結果 (避免溢出到非選取區)
        let mut final_mask = image::GrayImage::new(width, height);
        for region in regions {
            let start_x = region.x;
            let start_y = region.y;
            let end_x = (region.x + region.width).min(width);
            let end_y = (region.y + region.height).min(height);

            for y in start_y..end_y {
                for x in start_x..end_x {
                    if dilated_mask.get_pixel(x, y).0[0] > 0 {
                        final_mask.put_pixel(x, y, image::Luma([255]));
                    }
                }
            }
        }

        final_mask
    }

    /// 單次修復迭代 - 回傳是否還有像素被修復
    fn inpaint_iteration(
        &self,
        image: &mut RgbImage,
        mask: &mut image::GrayImage,
        confidence: &mut Vec<Vec<f32>>,
    ) -> bool {
        let (width, height) = image.dimensions();
        let mut updates = Vec::new();

        // 尋找邊界像素 (Boundary Pixels): 在遮罩內，但至少有一個鄰居在遮罩外(或已知)
        for y in 0..height {
            for x in 0..width {
                // 如果此像素是待修復區域
                if mask.get_pixel(x, y).0[0] > 0 {
                    // 檢查是否有"有效"鄰居
                    if self.has_valid_neighbor(mask, x, y, width, height) {
                        let (new_color, new_conf) =
                            self.calculate_pixel_value(image, mask, confidence, x, y);
                        updates.push((x, y, new_color, new_conf));
                    }
                }
            }
        }

        if updates.is_empty() {
            return false;
        }

        // 應用更新
        for (x, y, color, conf) in updates {
            image.put_pixel(x, y, color);
            mask.put_pixel(x, y, image::Luma([0])); // 標記為已修復 (有效)
            confidence[x as usize][y as usize] = conf; // 更新信心度
        }

        true
    }

    // 檢查是否有至少一個有效鄰居
    fn has_valid_neighbor(
        &self,
        mask: &image::GrayImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> bool {
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    if mask.get_pixel(nx as u32, ny as u32).0[0] == 0 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 計算像素的新顏色與信心度
    fn calculate_pixel_value(
        &self,
        image: &RgbImage,
        mask: &image::GrayImage,
        confidence: &Vec<Vec<f32>>,
        x: u32,
        y: u32,
    ) -> (Rgb<u8>, f32) {
        let (width, height) = image.dimensions();

        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;
        let mut sum_weight = 0.0;
        let mut sum_conf_weight = 0.0;
        let mut sum_conf = 0.0;

        let radius = 2i32; // 搜索半徑
        let sigma = 1.5; // 用於距離權重的 Sigma

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                    let nx_u = nx as u32;
                    let ny_u = ny as u32;

                    // 只參考已知/已修復的像素
                    if mask.get_pixel(nx_u, ny_u).0[0] == 0 {
                        let pixel = image.get_pixel(nx_u, ny_u);
                        let conf = confidence[nx_u as usize][ny_u as usize];

                        // 權重 = 距離權重 * 像素信心度
                        // 距離越近權重越高，信心度越高權重越高
                        let dist_sq = (dx * dx + dy * dy) as f32;
                        let dist_weight = (-dist_sq / (2.0 * sigma * sigma)).exp(); // Gaussian decay

                        // 定向權重 (簡化版): 這裡主要是距離權重
                        let weight = dist_weight * conf;

                        sum_r += pixel.0[0] as f32 * weight;
                        sum_g += pixel.0[1] as f32 * weight;
                        sum_b += pixel.0[2] as f32 * weight;
                        sum_weight += weight;

                        // 計算新信心度所需的累加
                        sum_conf += conf * dist_weight;
                        sum_conf_weight += dist_weight;
                    }
                }
            }
        }

        let new_color = if sum_weight > 0.0 {
            Rgb([
                (sum_r / sum_weight).clamp(0.0, 255.0) as u8,
                (sum_g / sum_weight).clamp(0.0, 255.0) as u8,
                (sum_b / sum_weight).clamp(0.0, 255.0) as u8,
            ])
        } else {
            *image.get_pixel(x, y)
        };

        // 更新信心度: 隨距離/迭代衰減
        let decay = 0.98; // 衰減因子
        let new_conf = if sum_conf_weight > 0.0 {
            (sum_conf / sum_conf_weight) * decay
        } else {
            0.0
        };

        (new_color, new_conf)
    }
}
