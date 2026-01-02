<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import WatermarkCanvas from './components/WatermarkCanvas.vue';

const currentImage = ref<string | null>(null);
const currentPath = ref<string | null>(null);
const currentRegion = ref<{ x: number, y: number, width: number, height: number } | null>(null);
const isProcessing = ref(false);
const statusMessage = ref('就緒');

const openImage = async () => {
    try {
        const file = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp']
            }]
        });

        if (file) {
            statusMessage.value = '載入中...';
            currentPath.value = file;
            // If v2 plugin-dialog, it returns null | FileResponse | FileResponse[].
            // FileResponse has `path`.
            
            // Invoke backend to load image preview
            // Wait, native <img src="file://..."> might work if CSP allows, but Tauri recommends using convertFileSrc or backend command.
            // I implemented `load_image` returning base64. Let's use that.
            
            const info = await invoke<{ preview: string, width: number, height: number }>('load_image', { path: file });
            currentImage.value = info.preview;
            currentRegion.value = null;
            const fileName = file.split(/[\\/]/).pop() || 'unknown';
            statusMessage.value = `已載入: ${fileName}`;
        }
    } catch (e) {
        console.error(e);
        statusMessage.value = `錯誤: ${e}`;
    }
};

const handleRemove = async () => {
    if (!currentPath.value || !currentRegion.value) return;
    
    try {
        isProcessing.value = true;
        statusMessage.value = '處理中...';
        
        // Ask for save location
        const savePath = await save({
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg']
            }],
            defaultPath: 'output.png'
        });
        
        if (!savePath) {
            isProcessing.value = false;
            statusMessage.value = '已取消';
            return;
        }
        
        const resultPreview = await invoke<string>('remove_watermark', {
            path: currentPath.value,
            outputPath: savePath, // Backend arg is snake_case 'output_path' usually?
            // Rust args are snake_case. Tauri invoke converts camelCase to snake_case automatically?
            // Yes, Tauri defaults to camelCase in JS -> snake_case in Rust.
            // My rust fn: fn remove_watermark(path: String, output_path: String, region: Region)
            // So JS: { path, outputPath, region }
            region: currentRegion.value
        });
        
        currentImage.value = resultPreview;
        statusMessage.value = '完成!';
        currentRegion.value = null; // Clear selection
    } catch (e) {
        console.error(e);
        statusMessage.value = `失敗: ${e}`;
    } finally {
        isProcessing.value = false;
    }
};

const updateRegion = (region: any) => {
    currentRegion.value = region;
};
</script>

<template>
  <div class="app-container">
    <header class="toolbar">
      <h1>Watermark Remover</h1>
      <div class="actions">
        <button @click="openImage" :disabled="isProcessing">開啟圖片</button>
        <button @click="handleRemove" :disabled="!currentRegion || isProcessing" class="primary">
            {{ isProcessing ? '處理中' : '移除浮水印' }}
        </button>
      </div>
    </header>
    
    <main class="workspace">
      <WatermarkCanvas 
        :image-src="currentImage" 
        @update:selection="updateRegion" 
      />
    </main>
    
    <footer class="statusbar">
      <span>{{ statusMessage }}</span>
      <span v-if="currentRegion">
          選取區域: {{ currentRegion.width }}x{{ currentRegion.height }} 
          @ ({{ currentRegion.x }}, {{ currentRegion.y }})
      </span>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: #1e1e1e;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 20px;
  height: 60px;
  background-color: #252526;
  border-bottom: 1px solid #333;
}

.toolbar h1 {
  font-size: 1.2rem;
  margin: 0;
  color: #fff;
}

.actions {
  display: flex;
  gap: 10px;
}

.workspace {
  flex: 1;
  display: flex;
  overflow: hidden;
  background-color: #1e1e1e;
}

.statusbar {
  height: 30px;
  background-color: #007acc;
  color: white;
  display: flex;
  align-items: center;
  padding: 0 20px;
  font-size: 0.8rem;
  justify-content: space-between;
}

button.primary {
  background-color: #0e639c;
  color: white;
}
button.primary:hover:not(:disabled) {
  background-color: #1177bb;
}
button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
