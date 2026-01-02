# Watermark Remover

一個基於 Rust 的命令列工具,能夠自動偵測並移除圖片中的浮水印。

## 功能特色

- ✅ **自動偵測**: 使用色彩分析與邊緣檢測自動識別浮水印
- ✅ **智能移除**: 採用 Inpainting 演算法自然填補浮水印區域
- ✅ **批次處理**: 支援一次處理整個目錄的圖片
- ✅ **手動模式**: 可手動指定浮水印區域座標
- ✅ **純 Rust 實作**: 無需外部 C++ 依賴,編譯簡單

## 支援格式

- JPG / JPEG
- PNG
- WebP
- BMP

## 安裝

### 前置需求

- Rust 1.70+ (安裝方式: https://rustup.rs/)

### 編譯

```bash
git clone <repository-url>
cd watermark-remover
cargo build --release
```

編譯完成後,執行檔位於 `target/release/watermark-remover`

## 使用方式

### 基本用法

```bash
# 單檔案處理 (自動偵測)
watermark-remover -i input.jpg -o output.jpg

# 批次處理整個目錄
watermark-remover -i ./images/ -o ./cleaned/

# 調整偵測敏感度 (0.0-1.0)
watermark-remover -i input.jpg -o output.jpg --sensitivity 0.7

# 手動指定浮水印區域 (x,y,width,height)
watermark-remover -i input.jpg -o output.jpg --region 100,50,200,80

# 指定多個區域
watermark-remover -i input.jpg -o output.jpg --region 100,50,200,80 --region 300,400,150,60

# 選擇 Inpainting 方法
watermark-remover -i input.jpg -o output.jpg --method gaussian  # 或 simple

# 顯示詳細處理過程
watermark-remover -i input.jpg -o output.jpg --verbose
```

### 參數說明

| 參數 | 簡寫 | 說明 | 預設值 |
|------|------|------|--------|
| `--input` | `-i` | 輸入檔案或目錄路徑 | (必填) |
| `--output` | `-o` | 輸出檔案或目錄路徑 | (必填) |
| `--sensitivity` | `-s` | 偵測敏感度 (0.0-1.0) | 0.5 |
| `--region` | `-r` | 手動指定區域 (x,y,w,h) | 無 |
| `--method` | `-m` | Inpainting 方法 (simple/gaussian) | gaussian |
| `--verbose` | `-v` | 顯示詳細日誌 | false |
| `--help` | `-h` | 顯示幫助訊息 | - |

## 演算法原理

### 1. 浮水印偵測

1. **色彩分析**: 將圖片轉換為 HSV 色彩空間,偵測異常顏色區域(如金色文字)
2. **邊緣檢測**: 使用 Canny 演算法找出文字與圖形的邊界
3. **形態學運算**: 使用膨脹運算連接斷開的區域
4. **連通區域分析**: 識別並標記連續的浮水印區域

### 2. 浮水印移除 (Inpainting)

提供兩種方法:

- **Simple**: 簡單平均法 - 取周圍 8 鄰域的平均值,速度快
- **Gaussian**: 高斯加權法 - 距離越近權重越高,效果更自然(推薦)

演算法會迭代多次以獲得更平滑的結果。

## 範例

### 處理單張圖片

```bash
cargo run --release -- -i test_input.jpg -o test_output.jpg -v
```

輸出:
```
=== Watermark Remover ===
輸入: "test_input.jpg"
輸出: "test_output.jpg"
敏感度: 0.5
方法: gaussian
載入圖片: "test_input.jpg"
圖片尺寸: 1024x768
自動偵測浮水印區域...
偵測到 2 個浮水印區域
  區域 1: (200, 100) 600x200
  區域 2: (800, 650) 150x80
移除浮水印...
儲存結果: "test_output.jpg"
✓ 處理完成: "test_output.jpg"
```

### 批次處理

```bash
cargo run --release -- -i ./test_images/ -o ./output/
```

輸出:
```
找到 15 個圖片檔案
[00:00:23] ========================================> 15/15 完成

=== 處理結果 ===
總計: 15 個檔案
成功: 14 個
失敗: 1 個
```

## 注意事項

> **⚠️ 版權聲明**
> 
> 此工具僅供個人學習與合法用途使用。使用者應確保:
> - 擁有圖片的合法使用權
> - 不侵犯他人版權
> - 遵守當地法律法規

> **⚠️ 免責聲明**
> 
> 開發者不對使用此工具產生的任何法律問題負責。

## 限制

- 對於覆蓋大面積的浮水印,效果可能不佳
- 複雜背景上的浮水印可能難以完美移除
- 處理大尺寸圖片時可能需要較長時間

## 授權

MIT License

## 貢獻

歡迎提交 Issue 與 Pull Request!
