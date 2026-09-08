<template>
  <div class="main-layout">
    <!-- 左侧栏：服务器列表 -->
    <div class="sidebar" :style="{ width: sidebarWidth + 'px' }">
      <ServerList @logout="emit('logout')" @open-asset="onOpenAsset" />
    </div>

    <!-- 分隔条 -->
    <div class="resize-handle" @mousedown="startResize">
      <div class="resize-line"></div>
    </div>

    <!-- 右侧：多标签终端面板 -->
    <div class="main-content">
      <TerminalPanel ref="terminalPanelRef" />
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import ServerList from './ServerList.vue'
import TerminalPanel from './TerminalPanel.vue'

const emit = defineEmits(['logout'])

const terminalPanelRef = ref(null)

const sidebarWidth = ref(200)
const minWidth = 200
const maxWidth = 400
let isResizing = false

onMounted(async () => {
  const settings = await window.electronAPI?.getSettings()
  if (settings?.sidebar_width) {
    sidebarWidth.value = Math.min(maxWidth, Math.max(minWidth, settings.sidebar_width))
  }
})

function onOpenAsset(asset, command) {
  terminalPanelRef.value?.openAssetTab(asset, command)
}

function startResize(e) {
  isResizing = true
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'

  const startX = e.clientX
  const startWidth = sidebarWidth.value

  function onMouseMove(e) {
    if (!isResizing) return
    const delta = e.clientX - startX
    const newWidth = startWidth + delta
    sidebarWidth.value = Math.min(maxWidth, Math.max(minWidth, newWidth))
  }

  function onMouseUp() {
    isResizing = false
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
    window.electronAPI?.saveSettings({ sidebar_width: sidebarWidth.value }).catch(() => {})
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

onBeforeUnmount(() => {
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
})
</script>

<style scoped>
.main-layout {
  display: flex;
  width: 100%;
  height: 100vh;
  min-height: 100vh;
  overflow: hidden;
}

.sidebar {
  flex-shrink: 0;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
}

.resize-handle {
  width: 4px;
  height: 100%;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  z-index: 10;
  background: transparent;
  transition: background 0.15s;
  display: flex;
  align-items: center;
  justify-content: center;
}
.resize-handle:hover {
  background: var(--accent);
}

.resize-line {
  width: 2px;
  height: 24px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 1px;
  transition: background 0.15s, height 0.15s;
}
.resize-handle:hover .resize-line {
  background: rgba(255, 255, 255, 0.5);
  height: 40px;
}

.main-content {
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
</style>
