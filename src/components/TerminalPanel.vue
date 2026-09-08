<template>
  <div class="terminal-panel">
    <!-- Tab 栏 -->
    <TerminalTabBar
      @select-tab="handleSelectTab"
      @close-tab="handleCloseTab"
    />

    <!-- 终端内容区 -->
    <div class="terminal-content">
      <!-- 终端内容搜索栏 -->
      <div v-if="searchVisible" class="terminal-search-bar">
        <input
          ref="searchInputRef"
          v-model="searchQuery"
          class="search-input"
          type="text"
          placeholder="搜索终端内容"
          spellcheck="false"
          autocomplete="off"
          @keydown.enter.prevent="onSearchEnter($event)"
          @keydown.esc.prevent="closeSearch"
          @input="onSearchInput"
        />
        <span class="search-status" v-if="searchQuery">{{ searchStatusText }}</span>
        <button class="search-btn" :title="'上一个 (Shift+Enter)'" @click="findPrev">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M3 8L6 4L9 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button class="search-btn" :title="'下一个 (Enter)'" @click="findNext">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M3 4L6 8L9 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </button>
        <button
          class="search-toggle"
          :class="{ active: searchCaseSensitive }"
          :title="'区分大小写'"
          @click="toggleCaseSensitive"
        >Aa</button>
        <button class="search-close" :title="'关闭 (Esc)'" @click="closeSearch">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>

      <!-- 无 Tab 时的欢迎页 -->
      <div v-if="tabs.length === 0" class="welcome-page">
        <div class="welcome-icon">
          <img :src="getIconSrc(appStore.appIcon)" alt="JumpServer Client" width="64" height="64" style="border-radius: 12px;" />
        </div>
        <p class="welcome-text">在左侧选择一个 Linux 服务器开始连接</p>
        <p class="welcome-hint">点击服务器将创建新标签页，支持同时连接多台服务器</p>
      </div>

      <!-- 终端容器（每个 tab 一个） -->
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="terminal-container"
        :class="{ 'terminal-hidden': tab.id !== activeTabId }"
        :ref="(el) => setTerminalRef(tab.id, el)"
      ></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, reactive, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { useMessage } from 'naive-ui'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { SearchAddon } from '@xterm/addon-search'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { Unicode11Addon } from '@xterm/addon-unicode11'
import { useAppStore } from '../stores/app'
import TerminalTabBar from './TerminalTabBar.vue'
import { getColorScheme } from '../styles/terminal-color-schemes'
import { getIconSrc } from '../utils/icons'
import '@xterm/xterm/css/xterm.css'

const appStore = useAppStore()
const message = useMessage()

const tabs = computed(() => appStore.tabs)
const activeTabId = computed(() => appStore.activeTabId)

// 每个 tab 的 xterm 实例映射
const terminalRefs = ref({})
const terminals = ref({})    // tabId -> Terminal
const fitAddons = ref({})    // tabId -> FitAddon
const searchAddons = ref({})  // tabId -> SearchAddon

// ==================== 终端内容搜索 ====================
const searchVisible = ref(false)
const searchQuery = ref('')
const searchCaseSensitive = ref(false)
const searchInputRef = ref(null)
// 每个 tab 的搜索结果统计 { index, count }，index 为 -1 表示超过高亮上限
const searchResultsByTab = reactive({})
const SEARCH_HIGHLIGHT_LIMIT_REACHED = -1

const searchStatusText = computed(() => {
  if (!searchQuery.value) return ''
  const info = searchResultsByTab[activeTabId.value]
  if (!info) return '无结果'
  if (info.count === 0) return '无结果'
  if (info.index === SEARCH_HIGHLIGHT_LIMIT_REACHED) return `${info.count}+ 个结果`
  return `${info.index + 1}/${info.count}`
})

// 搜索高亮配色（适配多数终端背景）
const searchDecorations = {
  matchBackground: '#3a3d41',
  matchOverviewRuler: '#3a3d41',
  activeMatchBackground: '#f5c518',
  activeMatchBorder: '#f5c518',
  activeMatchColorOverviewRuler: '#f5c518'
}

function buildSearchOptions() {
  return {
    caseSensitive: searchCaseSensitive.value,
    incremental: true,
    decorations: searchDecorations
  }
}

function getActiveSearchAddon() {
  const id = activeTabId.value
  return id ? searchAddons.value[id] : null
}

// 执行一次搜索（direction: 'next' | 'prev'）
function runSearch(direction) {
  const addon = getActiveSearchAddon()
  if (!addon) return
  const q = searchQuery.value
  if (!q) {
    addon.clearDecorations()
    return
  }
  const opts = buildSearchOptions()
  try {
    if (direction === 'prev') {
      addon.findPrevious(q, opts)
    } else {
      addon.findNext(q, opts)
    }
  } catch (e) { /* ignore */ }
}

function onSearchInput() {
  // 输入变化时增量搜索当前 tab
  runSearch('next')
}

function onSearchEnter(e) {
  // Enter 下一个，Shift+Enter 上一个
  runSearch(e.shiftKey ? 'prev' : 'next')
}

function findNext() { runSearch('next') }
function findPrev() { runSearch('prev') }

function toggleCaseSensitive() {
  searchCaseSensitive.value = !searchCaseSensitive.value
  if (searchQuery.value) {
    // 重新从头搜索
    const addon = getActiveSearchAddon()
    if (addon) {
      try { addon.findNext(searchQuery.value, buildSearchOptions()) } catch (e) { /* ignore */ }
    }
  }
}

function openSearch() {
  searchVisible.value = true
  nextTick(() => {
    const el = searchInputRef.value
    if (el) {
      el.focus()
      el.select()
    }
  })
}

function closeSearch() {
  searchVisible.value = false
  // 清除当前 tab 的搜索高亮
  const addon = getActiveSearchAddon()
  if (addon) {
    try { addon.clearDecorations() } catch (e) { /* ignore */ }
  }
  // 将焦点交还给终端
  const id = activeTabId.value
  const term = id ? terminals.value[id] : null
  if (term) {
    try { term.focus() } catch (e) { /* ignore */ }
  }
}

function toggleSearch() {
  if (searchVisible.value) {
    // 已打开：聚焦并全选输入框
    const el = searchInputRef.value
    if (el) {
      el.focus()
      el.select()
    }
  } else {
    openSearch()
  }
}

// xterm 主题配置（使用用户选择的配色方案）
function getCurrentTheme() {
  return getColorScheme(appStore.terminalColorScheme, appStore.theme)
}

let cleanupDataListener = null
let cleanupStatusListener = null
let resizeRafId = null
let resizeTimeoutId = null
let contentResizeObserver = null
let searchKeyListener = null

// 注册全局搜索快捷键 (Ctrl/Cmd + F)
function registerSearchShortcut() {
  const handler = (e) => {
    if ((e.ctrlKey || e.metaKey) && (e.key === 'f' || e.key === 'F')) {
      e.preventDefault()
      toggleSearch()
    }
  }
  window.addEventListener('keydown', handler, true)
  searchKeyListener = () => window.removeEventListener('keydown', handler, true)
}

// RAF 轮询：等待容器获得正确尺寸后再 fit，并强制将新尺寸同步给 SSH 后端
function tryFitWhenReady(tabId, fitAddon, attempts = 0) {
  const container = terminalRefs.value[tabId]
  if (!container || !fitAddon) return

  if (container.offsetWidth > 50 && container.offsetHeight > 50) {
    const term = terminals.value[tabId]
    if (!term) return
    let cols = 0, rows = 0
    try {
      fitAddon.fit()
      cols = term.cols
      rows = term.rows
    } catch (e) { /* ignore */ }
    // fit 后通知 SSH 服务端新的终端尺寸（避免乱码）
    if (cols > 0 && rows > 0) {
      window.electronAPI?.sendTerminalResize(tabId, cols, rows)
    }
  } else if (attempts < 60) {
    requestAnimationFrame(() => tryFitWhenReady(tabId, fitAddon, attempts + 1))
  }
}

// 对单个 tab 强制 fit 并同步 SSH（用于 resize 事件）
function fitAndSync(tabId) {
  const term = terminals.value[tabId]
  const fitAddon = fitAddons.value[tabId]
  const container = terminalRefs.value[tabId]
  if (!term || !fitAddon || !container) return
  // 跳过不可见容器（避免取到错误尺寸）
  if (container.offsetWidth < 50 || container.offsetHeight < 50) return
  let cols = 0, rows = 0
  try {
    fitAddon.fit()
    cols = term.cols
    rows = term.rows
  } catch (e) { /* ignore */ }
  if (cols > 0 && rows > 0) {
    window.electronAPI?.sendTerminalResize(tabId, cols, rows)
  }
}

function setTerminalRef(tabId, el) {
  if (el) {
    terminalRefs.value[tabId] = el
    // 如果 template ref 在 xterm 创建之后才被设置，初始化终端
    if (!terminals.value[tabId]) {
      initTerminalForTab(tabId)
    }
  }
}

// 为指定 tab 初始化 xterm 实例
function initTerminalForTab(tabId) {
  const container = terminalRefs.value[tabId]
  if (!container || terminals.value[tabId]) return

  const term = new Terminal({
    cursorBlink: true,
    cursorStyle: 'bar',
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: getCurrentTheme(),
    allowProposedApi: true,
    scrollback: 5000
  })

  const fitAddon = new FitAddon()
  term.loadAddon(fitAddon)

  const searchAddon = new SearchAddon()
  term.loadAddon(searchAddon)
  searchAddon.onDidChangeResults(({ resultIndex, resultCount }) => {
    // 仅当前活跃 tab 的结果用于状态显示，其他 tab 的结果存档
    searchResultsByTab[tabId] = { index: resultIndex, count: resultCount }
  })

  const webLinksAddon = new WebLinksAddon()
  term.loadAddon(webLinksAddon)

  const unicode11Addon = new Unicode11Addon()
  term.loadAddon(unicode11Addon)
  term.unicode.activeVersion = '11'

  try {
    term.open(container)
  } catch (err) {
    console.error('[TerminalPanel] xterm.open() failed:', err)
    return
  }

  terminals.value[tabId] = term
  fitAddons.value[tabId] = fitAddon
  searchAddons.value[tabId] = searchAddon

  // 用户输入 -> 发送到对应 tab 的 SSH
  term.onData((data) => {
    window.electronAPI?.sendTerminalData(tabId, data)
  })

  // 适配终端大小（RAF 轮询确保 DOM 尺寸已计算）
  tryFitWhenReady(tabId, fitAddon)
}

// 销毁指定 tab 的终端（即使 xterm 未初始化成功，也要清理所有映射，避免残留）
function destroyTerminalForTab(tabId) {
  // 顺带清理未发送的快捷指令排队（若有）
  clearPendingCommand(tabId)
  const term = terminals.value[tabId]
  if (term) {
    try { term.dispose() } catch (e) { /* ignore */ }
  }
  delete terminals.value[tabId]
  delete fitAddons.value[tabId]
  delete searchAddons.value[tabId]
  delete searchResultsByTab[tabId]
  delete terminalRefs.value[tabId]
}

// ==================== 快捷指令排队发送 ====================
// 绑定资产的快捷指令：等对应 tab 连接成功（ssh-status: connected）后再发送，
// 替代不可靠的固定延时；连接失败/超时/关闭时清理队列并提示
const pendingCommands = {}   // tabId -> command
const pendingTimeouts = {}   // tabId -> 超时兜底 timer

function clearPendingCommand(tabId, reason) {
  const cmd = pendingCommands[tabId]
  if (cmd === undefined) return
  delete pendingCommands[tabId]
  if (pendingTimeouts[tabId]) {
    clearTimeout(pendingTimeouts[tabId])
    delete pendingTimeouts[tabId]
  }
  if (reason) {
    message.warning(`快捷指令「${cmd}」未发送：${reason}`)
  }
}

// 连接 SSH 到指定 tab
async function connectSSHForTab(tabId, asset, cols, rows) {
  try {
    const result = await window.electronAPI.connectToAsset(
      appStore.jmsUrl,
      appStore.keyId,
      appStore.secret,
      appStore.userInfo?.username || '',
      asset.id,
      tabId,
      cols,
      rows
    )
    if (!result.success) {
      message.error(result.error || 'SSH 连接失败')
      const term = terminals.value[tabId]
      if (term) {
        term.writeln(`\r\n\x1b[31m连接失败: ${result.error}\x1b[0m`)
      }
      clearPendingCommand(tabId)
      return
    }
    appStore.setTabConnected(tabId, true)
  } catch (err) {
    message.error(`连接失败: ${err.message}`)
    clearPendingCommand(tabId)
  }
}

// ==================== Tab 操作 ====================
function handleSelectTab(tabId) {
  appStore.setActiveTab(tabId)
}

function handleCloseTab(tabId) {
  // 断开该 tab 的 SSH 连接（TabBar 只负责 emit，关闭逻辑统一在这里收口）
  window.electronAPI?.disconnectSSH(tabId)?.catch(() => {})
  // 销毁 xterm 实例并清理映射，避免内存泄漏
  destroyTerminalForTab(tabId)
  // 从 store 中移除 tab
  appStore.closeTab(tabId)
}

// 当用户点击左侧服务器列表时调用；command 为快捷指令绑定的命令（可选）
async function openAssetTab(asset, command = null) {
  const tab = appStore.createTab(asset)

  // 有绑定命令时先排队，连接成功后由 ssh-status 事件触发发送；超时兜底防止无声丢失
  if (command) {
    pendingCommands[tab.id] = command
    pendingTimeouts[tab.id] = setTimeout(() => {
      clearPendingCommand(tab.id, '连接超时（15 秒未连接成功）')
    }, 15000)
  }

  await nextTick()
  await nextTick() // 确保 DOM 渲染和 ref 绑定完成

  if (!terminals.value[tab.id]) {
    initTerminalForTab(tab.id)
  }

  // 获取终端实际尺寸
  const term = terminals.value[tab.id]
  const fitAddon = fitAddons.value[tab.id]
  if (fitAddon && term) {
    tryFitWhenReady(tab.id, fitAddon)
    // 等待 fit 完成后获取准确的 cols/rows
    await new Promise(resolve => requestAnimationFrame(resolve))
  }
  const cols = term?.cols || 80
  const rows = term?.rows || 24

  // 建立 SSH 连接（传入实际终端尺寸）
  await connectSSHForTab(tab.id, asset, cols, rows)

  // 连接后重新 fit 确保终端填满容器
  const fitAddonAfterConnect = fitAddons.value[tab.id]
  if (fitAddonAfterConnect) {
    tryFitWhenReady(tab.id, fitAddonAfterConnect)
  }
}

// ==================== 监听 Tab 切换 ====================
watch(activeTabId, (newId, oldId) => {
  const fitAddon = fitAddons.value[newId]
  if (fitAddon) {
    tryFitWhenReady(newId, fitAddon)
  }
  // 切换 tab 时，如果搜索栏打开，重新在新 tab 中搜索
  if (searchVisible.value && searchQuery.value) {
    nextTick(() => {
      // 确保输入框保持焦点
      const el = searchInputRef.value
      if (el) {
        el.focus()
      }
      runSearch('next')
    })
  }
})

// ==================== 监听搜索栏显隐 ====================
watch(searchVisible, (visible) => {
  if (visible) {
    nextTick(() => {
      const el = searchInputRef.value
      if (el) {
        el.focus()
        el.select()
      }
      if (searchQuery.value) {
        runSearch('next')
      }
    })
  } else {
    // 关闭时清除所有 tab 的搜索高亮
    Object.values(searchAddons.value).forEach(addon => {
      try { addon.clearDecorations() } catch (e) { /* ignore */ }
    })
    Object.keys(searchResultsByTab).forEach(k => delete searchResultsByTab[k])
  }
})

// ==================== 监听主题变化，更新所有终端 ====================
watch([() => appStore.theme, () => appStore.terminalColorScheme], () => {
  const newTheme = getCurrentTheme()
  Object.keys(terminals.value).forEach(tabId => {
    const term = terminals.value[tabId]
    if (term) {
      term.options.theme = newTheme
    }
  })
})

// ==================== 暴露方法给父组件 ====================
defineExpose({ openAssetTab })

// ==================== 生命周期 ====================
onMounted(async () => {
  // 全局搜索快捷键（最先注册，确保不受后续 async 操作影响）
  registerSearchShortcut()

  // 监听来自后端的终端数据
  if (window.electronAPI?.onTerminalData) {
    try {
      cleanupDataListener = await window.electronAPI.onTerminalData((tabId, data) => {
        const term = terminals.value[tabId]
        if (term) {
          term.write(data)
        }
      })
    } catch (e) {
      console.error('[TerminalPanel] onTerminalData failed:', e)
    }
  }

  // 监听 SSH 状态变化
  if (window.electronAPI?.onSSHStatus) {
    try {
      cleanupStatusListener = await window.electronAPI.onSSHStatus((tabId, status) => {
        if (status === 'disconnected' || status === 'error') {
          appStore.setTabConnected(tabId, false)
          if (status === 'error') {
            message.error('SSH 连接异常断开')
          }
        } else if (status === 'connected') {
          appStore.setTabConnected(tabId, true)
          // 连接成功：立即发送排队的快捷指令（此时后端连接已就绪，不会丢失）
          const pending = pendingCommands[tabId]
          if (pending !== undefined) {
            clearPendingCommand(tabId)
            window.electronAPI?.sendTerminalData(tabId, pending + '\n')
          }
        }
      })
    } catch (e) {
      console.error('[TerminalPanel] onSSHStatus failed:', e)
    }
  }

  // 窗口 resize / 容器尺寸变化时重新 fit 所有终端
  const handleResize = () => {
    if (resizeRafId) cancelAnimationFrame(resizeRafId)
    resizeRafId = requestAnimationFrame(() => {
      resizeRafId = null
      Object.keys(terminals.value).forEach(tabId => fitAndSync(tabId))
    })
  }
  window.addEventListener('resize', handleResize)

  // 用 ResizeObserver 监听 .terminal-content 容器尺寸变化
  // 这比 window.resize 更精准，能捕获 flex/侧边栏拖动导致的尺寸变化
  const contentEl = document.querySelector('.terminal-content')
  if (contentEl && typeof ResizeObserver !== 'undefined') {
    contentResizeObserver = new ResizeObserver(() => {
      // 200ms 防抖：避免拖动侧边栏时频繁 fit 导致卡顿/乱码
      if (resizeTimeoutId) clearTimeout(resizeTimeoutId)
      resizeTimeoutId = setTimeout(() => {
        Object.keys(terminals.value).forEach(tabId => fitAndSync(tabId))
      }, 80)
    })
    contentResizeObserver.observe(contentEl)
  }

  const origCleanup = cleanupDataListener
  cleanupDataListener = () => {
    origCleanup?.()
    window.removeEventListener('resize', handleResize)
    if (resizeRafId) {
      cancelAnimationFrame(resizeRafId)
      resizeRafId = null
    }
    if (resizeTimeoutId) {
      clearTimeout(resizeTimeoutId)
      resizeTimeoutId = null
    }
    if (contentResizeObserver) {
      contentResizeObserver.disconnect()
      contentResizeObserver = null
    }
    searchKeyListener?.()
  }
})

onBeforeUnmount(() => {
  cleanupDataListener?.()
  cleanupStatusListener?.()
  // 销毁所有终端
  Object.keys(terminals.value).forEach(tabId => {
    destroyTerminalForTab(tabId)
  })
})
</script>

<style scoped>
.terminal-panel {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  background: var(--terminal-bg);
}

.terminal-content {
  flex: 1;
  min-height: 0;
  position: relative;
}

.terminal-container {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.terminal-container :deep(.xterm) {
  width: 100%;
  height: 100%;
}

.terminal-hidden {
  display: none;
}

/* 欢迎页 */
.welcome-page {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  gap: 12px;
  user-select: none;
}

.welcome-icon {
  opacity: 0.3;
  margin-bottom: 8px;
}

.welcome-text {
  font-size: 14px;
  color: var(--text-secondary);
}

.welcome-hint {
  font-size: 12px;
  color: var(--text-muted);
}

/* ==================== 终端内容搜索栏 ==================== */
.terminal-search-bar {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 8px;
  border-radius: 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
  user-select: none;
}

.search-input {
  width: 180px;
  height: 24px;
  padding: 0 8px;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  user-select: text;
}
.search-input:focus {
  border-color: var(--accent);
}

.search-status {
  min-width: 52px;
  text-align: center;
  font-size: 12px;
  color: var(--text-secondary);
}

.search-btn,
.search-toggle,
.search-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 3px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: background 0.1s, color 0.1s, border-color 0.1s;
}
.search-btn:hover,
.search-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.search-toggle {
  font-size: 12px;
  font-weight: 600;
  border-color: var(--border-color);
}
.search-toggle.active {
  background: var(--accent);
  border-color: var(--accent);
  color: #ffffff;
}
.search-toggle:hover {
  background: var(--bg-hover);
}
.search-toggle.active:hover {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.search-close {
  margin-left: 2px;
}
</style>
