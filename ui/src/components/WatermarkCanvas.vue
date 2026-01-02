<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';

const props = defineProps<{
  imageSrc: string | null;
}>();

const emit = defineEmits<{
  (e: 'update:selection', region: { x: u32, y: u32, width: u32, height: u32 } | null): void;
}>();

const containerRef = ref<HTMLDivElement | null>(null);
const imageRef = ref<HTMLImageElement | null>(null);

const isDragging = ref(false);
const startX = ref(0);
const startY = ref(0);
const currentX = ref(0);
const currentY = ref(0);

const selection = ref<{ x: number, y: number, w: number, h: number } | null>(null);

const handleMouseDown = (e: MouseEvent) => {
  if (!imageRef.value || !props.imageSrc) return;
  
  const rect = imageRef.value.getBoundingClientRect();
  startX.value = e.clientX - rect.left;
  startY.value = e.clientY - rect.top;
  currentX.value = startX.value;
  currentY.value = startY.value;
  
  isDragging.value = true;
  selection.value = { x: startX.value, y: startY.value, w: 0, h: 0 };
  
  emit('update:selection', null); // Reset selection
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value || !imageRef.value) return;
  
  const rect = imageRef.value.getBoundingClientRect();
  currentX.value = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
  currentY.value = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
  
  const x = Math.min(startX.value, currentX.value);
  const y = Math.min(startY.value, currentY.value);
  const w = Math.abs(currentX.value - startX.value);
  const h = Math.abs(currentY.value - startY.value);
  
  selection.value = { x, y, w, h };
};

const handleMouseUp = () => {
  if (!isDragging.value || !selection.value || !imageRef.value) return;
  isDragging.value = false;
  
  // Calculate actual pixel coordinates
  // The displayed image might be scaled via CSS object-fit or simple width/height
  // But here we display it as natural size mostly? 
  // Wait, if image is large, we scale it to fit screen. We need to map screen coords to image coords.
  
  const img = imageRef.value;
  const naturalWidth = img.naturalWidth;
  const naturalHeight = img.naturalHeight;
  const clientWidth = img.clientWidth;
  const clientHeight = img.clientHeight;
  
  const scaleX = naturalWidth / clientWidth;
  const scaleY = naturalHeight / clientHeight;
  
  const actualX = Math.round(selection.value.x * scaleX);
  const actualY = Math.round(selection.value.y * scaleY);
  const actualW = Math.round(selection.value.w * scaleX);
  const actualH = Math.round(selection.value.h * scaleY);
  
  if (actualW > 0 && actualH > 0) {
    emit('update:selection', { x: actualX, y: actualY, width: actualW, height: actualH });
  } else {
    selection.value = null;
    emit('update:selection', null);
  }
};

onMounted(() => {
  window.addEventListener('mouseup', () => {
    if (isDragging.value) handleMouseUp();
  });
});
</script>

<template>
  <div class="canvas-container" ref="containerRef">
    <div v-if="!imageSrc" class="placeholder">
      <p>請開啟圖片以開始</p>
    </div>
    <div v-else class="image-wrapper" 
         @mousedown="handleMouseDown" 
         @mousemove="handleMouseMove"
         @mouseup="handleMouseUp">
      <img ref="imageRef" :src="imageSrc" class="target-image" draggable="false" />
      
      <div v-if="selection" class="selection-box"
           :style="{
             left: selection.x + 'px',
             top: selection.y + 'px',
             width: selection.w + 'px',
             height: selection.h + 'px'
           }">
      </div>
    </div>
  </div>
</template>

<style scoped>
.canvas-container {
  flex: 1;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #121212;
  overflow: hidden;
  padding: 20px;
}

.placeholder {
  color: #666;
  font-size: 1.2rem;
}

.image-wrapper {
  position: relative;
  max-width: 100%;
  max-height: 100%;
  box-shadow: 0 4px 20px rgba(0,0,0,0.5);
  user-select: none;
}

.target-image {
  display: block;
  max-width: 100%;
  max-height: 80vh; /* Limit height to viewport */
  pointer-events: none; /* Let wrapper handle events? No, we need size. */
}

/* Re-enable events on wrapper, disable on img to prevent native drag */
.image-wrapper {
  cursor: crosshair;
}

.selection-box {
  position: absolute;
  border: 2px solid #ffeb3b;
  background-color: rgba(255, 235, 59, 0.2);
  pointer-events: none;
}
</style>
