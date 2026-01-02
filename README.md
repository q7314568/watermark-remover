# Watermark Remover Desktop App

一個高效、智慧的浮水印移除桌面應用程式，專為還原圖片細節而設計。
本專案採用 **Rust** (後端) 與 **Vue 3** (前端) 構建，透過 **Tauri** 框架實現輕量級與高效能。

## ✨ 核心特色 (New!)

### 1. 🧠 智慧遮罩 (Smart Masking)
不再粗糙地移除整個選取框！
- **自動摳圖**：演算法會自動分析選取區域內的亮度與對比度。
- **背景保留**：精準識別浮水印文字/圖形，保留文字縫隙間的原始背景，避免不必要的重繪。
- **邊緣優化**：自動處理抗鋸齒邊緣，確保移除乾淨無殘留。

### 2. 💧 信心度修復 (Confidence-based Inpainting)
告別塗抹感！
- **洋蔥剝皮法 (Onion-Peel)**：由外向內層層修復，模擬自然紋理延伸。
- **信心度權重**：修復時優先參考「高信心度」(即原始影像) 的像素。
- **抗塗抹**：有效解決傳統演算法產生的單色長條塗抹痕跡，還原更自然的紋理。

### 3. 🚀 現代化桌面體驗
- **即時預覽**：載入圖片與處理結果即時顯示。
- **直覺操作**：拖曳滑鼠即可選取浮水印區域。
- **跨平台**：支援 Windows, macOS, Linux (基於 Tauri)。

---

## 🛠️ 技術堆疊

- **Frontend**: [Vue 3](https://vuejs.org/) (TypeScript, Composition API)
- **Backend**: [Rust](https://www.rust-lang.org/)
- **Framework**: [Tauri v2](https://tauri.app/)
- **Image Processing**: `image`, `imageproc` (Rust Crates)

---

## 🚀 快速開始

### 前置需求
- **Node.js** (建議 v18+)
- **Rust** (安裝方式: [rustup.rs](https://rustup.rs/))
- **Build Tools** (Windows 需安裝 C++ 生成工具)

### 安裝與執行

1. **複製專案**
   ```bash
   git clone https://github.com/q7314568/watermark-remover.git
   cd watermark-remover
   ```

2. **安裝前端依賴**
   ```bash
   cd ui
   npm install
   ```

3. **開發模式執行**
   ```bash
   # 在專案根目錄執行 (會同時啟動 Rust 後端與 Vue 前端)
   npm run tauri dev
   ```

4. **編譯發布版**
   ```bash
   npm run tauri build
   ```
   執行檔將產生於 `src-tauri/target/release/` 目錄下。

---

## 📖 使用指南

1. 點擊左上角的 **「開啟圖片」** 按鈕，選擇含有浮水印的圖片。
2. 在圖片上 **拖曳滑鼠**，框選浮水印的大致範圍（不需要非常精確，智慧遮罩會自動處理）。
3. 點擊 **「移除浮水印」** 按鈕。
4. 完成！您可以選擇儲存處理後的圖片。

---

## ⚠️ 免責聲明

此工具僅供技術研究與個人學習使用。
- 請勿用於非法用途。
- 使用者應確保擁有圖片的合法使用權，並不侵犯他人版權。

---

## 🤝 貢獻

歡迎提交 Issue 或 Pull Request 來協助改進演算法或 UI！

## 📄 授權

MIT License
