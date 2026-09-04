<template>
  <div class="server-list">
    <!-- 搜索栏 -->
    <div class="list-header">
      <div class="search-box">
        <n-input
          v-model:value="searchQuery"
          placeholder="搜索名称 / IP / 节点..."
          clearable
          size="small"
          :input-props="{ autocomplete: 'off' }"
        >
          <template #prefix>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <circle cx="6" cy="6" r="4.5" stroke="currentColor" stroke-width="1.2"/>
              <path d="M9.5 9.5L12.5 12.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            </svg>
          </template>
        </n-input>
      </div>

      <div class="header-actions">
        <span class="server-count">{{ filteredAssets.length }} / {{ assets.length }}</span>
        <div class="header-buttons">
          <n-button text size="tiny" @click="toggleLayout" :title="layoutTitle">
            <template #icon>
              <svg v-if="appStore.assetLayout === 'flat'" width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M2 3h10M2 7h10M2 11h10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              </svg>
              <svg v-else width="14" height="14" viewBox="0 0 14 14" fill="none">
                <path d="M2 3h10M4 7h8M6 11h6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              </svg>
            </template>
          </n-button>
          <n-popover trigger="click" placement="bottom-end">
          <template #trigger>
            <n-button text size="tiny" class="color-scheme-btn" :title="terminalColorSchemeTitle">
              <template #icon>
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <path d="M2 4l3 3-3 3M7 11h5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </template>
            </n-button>
          </template>
          <div class="color-scheme-popover">
            <div class="color-scheme-title">终端配色方案</div>
            <n-select
              :value="appStore.terminalColorScheme"
              :options="terminalColorSchemeOptions"
              size="small"
              @update:value="appStore.setTerminalColorScheme"
            />
            <div class="color-scheme-preview">
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.background }"></span>
                <span class="preview-label">背景</span>
              </div>
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.foreground }"></span>
                <span class="preview-label">前景</span>
              </div>
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.red }"></span>
                <span class="preview-label">红色</span>
              </div>
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.green }"></span>
                <span class="preview-label">绿色</span>
              </div>
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.blue }"></span>
                <span class="preview-label">蓝色</span>
              </div>
              <div class="preview-row">
                <span class="preview-color" :style="{ background: previewColors.yellow }"></span>
                <span class="preview-label">黄色</span>
              </div>
            </div>
          </div>
        </n-popover>
        <n-button text size="tiny" @click="refreshAssets" title="刷新">
          <template #icon>
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none" :class="{ spinning: loading }">
              <path d="M2 7a5 5 0 0 1 5-5c1.8 0 3.4.9 4.3 2.4M12 7a5 5 0 0 1-5 5c-1.8 0-3.4-.9-4.3-2.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              <path d="M11 2v2.5H8.5M3 12v-2.5h2.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
          </template>
        </n-button>
        </div>
      </div>
    </div>

    <!-- 用户信息栏 -->
    <div class="user-bar">
      <span class="user-avatar">{{ userInitial }}</span>
      <span class="user-name">{{ appStore.userInfo?.username || '未登录' }}</span>
      <n-button text size="tiny" @click="toggleTheme" class="theme-btn" :title="themeTitle">
        <template #icon>
          <svg v-if="appStore.theme === 'dark'" width="14" height="14" viewBox="0 0 14 14" fill="none">
            <circle cx="7" cy="7" r="3" stroke="currentColor" stroke-width="1.2"/>
            <path d="M7 1v1.5M7 11.5V13M1 7h1.5M11.5 7H13M2.8 2.8l1 1M10.2 10.2l1 1M2.8 11.2l1-1M10.2 3.8l1-1" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
          <svg v-else width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M12.5 8.5a5.5 5.5 0 01-7-7 5.5 5.5 0 107 7z" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </template>
      </n-button>
      <n-button text size="tiny" type="error" @click="$emit('logout')" class="logout-btn">
        退出
      </n-button>
    </div>

    <!-- 快捷指令栏 -->
    <div class="quick-commands-bar">
      <div class="quick-commands-header">
        <span class="quick-commands-title">快捷指令</span>
        <n-button text size="tiny" @click="showAddCommandModal" title="添加快捷指令">
          <template #icon>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M6 1v10M1 6h10" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            </svg>
          </template>
        </n-button>
      </div>
      <div v-if="appStore.quickCommands.length === 0" class="quick-commands-empty">
        暂无快捷指令
      </div>
      <div v-else class="quick-commands-list">
        <div
          v-for="cmd in appStore.quickCommands"
          :key="cmd.id"
          class="quick-command-item"
          @click="executeQuickCommand(cmd)"
          :title="`执行: ${cmd.command}`"
        >
          <span class="quick-command-icon">⚡</span>
          <span class="quick-command-name">{{ cmd.name }}</span>
          <n-button
            text
            size="tiny"
            class="quick-command-edit"
            @click.stop="editQuickCommand(cmd)"
            title="编辑"
          >
            <template #icon>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M7.5 1.5l1 1-5 5H2.5v-1l5-5z" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </template>
          </n-button>
          <n-button
            text
            size="tiny"
            class="quick-command-delete"
            @click.stop="deleteQuickCommand(cmd.id)"
            title="删除"
          >
            <template #icon>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M2 2l6 6M8 2l-6 6" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
              </svg>
            </template>
          </n-button>
          <n-button
            text
            size="tiny"
            class="quick-command-broadcast"
            @click.stop="executeQuickCommandToAll(cmd)"
            title="发送到所有终端"
          >
            <template #icon>
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M1 2l4 3-4 3M5 2l4 3-4 3" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </template>
          </n-button>
        </div>
      </div>
    </div>

    <!-- 加载状态 -->
    <div v-if="loading" class="loading-area">
      <n-spin size="small" />
      <span>加载服务器列表...</span>
    </div>

    <!-- 错误信息 -->
    <div v-else-if="errorMsg" class="error-area">
      <span class="error-icon">!</span>
      <span>{{ errorMsg }}</span>
      <n-button size="tiny" @click="refreshAssets">重试</n-button>
    </div>

    <!-- 服务器列表 -->
    <div v-else class="list-body" ref="listBodyRef">
      <div v-if="appStore.assetLayout === 'flat'">
        <div v-if="filteredAssets.length === 0 && !loading" class="empty-state">
          <p>暂无 Linux 服务器</p>
        </div>

        <div
          v-for="(asset, index) in filteredAssets"
          :key="asset.id"
          class="asset-item"
          :class="{ 'drag-over': dragOverIndex === index }"
          draggable="true"
          @click="openAsset(asset)"
          @dragstart="onDragStart($event, index)"
          @dragover.prevent="onDragOver($event, index)"
          @dragleave="onDragLeave"
          @drop="onDrop($event, index)"
          @dragend="onDragEnd"
        >
          <!-- 连接状态指示 -->
          <span class="asset-status" :class="{ connected: isAssetConnected(asset.id) }"></span>

          <div class="asset-info">
            <div class="asset-title">
              <span>{{ asset.title }}</span>
            </div>
          </div>

          <div class="asset-action">
            <n-button
              text
              size="tiny"
              @click.stop="showTagInput(asset)"
              title="添加标签"
            >
              <template #icon>
                <svg width="13" height="13" viewBox="0 0 13 13" fill="none">
                  <path d="M2 2h4l3 3-5 5L2 8V2z" stroke="currentColor" stroke-width="1" fill="none"/>
                  <circle cx="8.5" cy="4.5" r="0.8" fill="currentColor"/>
                </svg>
              </template>
            </n-button>
          </div>

          <!-- 标签 -->
          <div v-if="appStore.getAssetTags(asset.id).length > 0" class="asset-tags">
            <span
              v-for="tag in appStore.getAssetTags(asset.id)"
              :key="tag"
              class="tag-chip"
            >
              {{ tag }}
              <span class="close-btn" @click.stop="appStore.removeTag(asset.id, tag)">&times;</span>
            </span>
          </div>
        </div>
      </div>

      <div v-else>
        <div v-if="visibleTree.length === 0 && !loading" class="empty-state">
          <p>暂无 Linux 服务器</p>
        </div>

        <AssetTree
          v-else
          :nodes="visibleTree"
          :expanded-ids="expandedIds"
          :force-expand="isSearching"
          @toggle="toggleNode"
          @open-asset="openTreeAsset"
          @add-tag="showTagInput"
        />
      </div>
    </div>

    <!-- 添加标签弹窗 -->
    <n-modal
      v-model:show="showTagModal"
      preset="dialog"
      title="添加标签"
      positive-text="确定"
      negative-text="取消"
      @positive-click="confirmAddTag"
      @negative-click="cancelAddTag"
    >
      <n-input
        v-model:value="newTagValue"
        placeholder="输入标签名称"
        @keyup.enter="confirmAddTag"
      />
    </n-modal>

    <!-- 添加/编辑快捷指令弹窗 -->
    <n-modal
      v-model:show="showCommandModal"
      preset="dialog"
      :title="commandModalMode === 'add' ? '添加快捷指令' : '编辑快捷指令'"
      positive-text="确定"
      negative-text="取消"
      @positive-click="confirmCommand"
      @negative-click="cancelCommand"
    >
      <n-form label-placement="top">
        <n-form-item label="指令名称">
          <n-input
            v-model:value="newCommand.name"
            placeholder="例如: 查看磁盘空间"
          />
        </n-form-item>
        <n-form-item label="执行命令">
          <n-input
            v-model:value="newCommand.command"
            placeholder="例如: df -h"
          />
        </n-form-item>
        <n-form-item label="目标资产 (可选)">
          <n-select
            v-model:value="newCommand.assetId"
            :options="assetOptions"
            placeholder="留空则手动选择资产"
            clearable
          />
        </n-form-item>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import AssetTree from './AssetTree.vue'
import { NInput, NButton, NSpin, NModal, NForm, NFormItem, NSelect, NPopover, useMessage } from 'naive-ui'
import { useAppStore } from '../stores/app'
import { getAllColorSchemes, getColorScheme } from '../styles/terminal-color-schemes'

const emit = defineEmits(['logout', 'openAsset'])
const message = useMessage()
const appStore = useAppStore()

const searchQuery = ref('')
const loading = ref(true)
const errorMsg = ref('')
const listBodyRef = ref(null)
const expandedIds = ref({})

// 拖拽状态
const dragIndex = ref(-1)
const dragOverIndex = ref(-1)

// 标签弹窗
const showTagModal = ref(false)
const newTagValue = ref('')
const currentTagAsset = ref(null)

// 快捷指令弹窗
const showCommandModal = ref(false)
const commandModalMode = ref('add') // 'add' or 'edit'
const editingCommandId = ref(null)
const newCommand = ref({
  name: '',
  command: '',
  assetId: null
})

const assets = computed(() => appStore.assets)

const assetOptions = computed(() => {
  return appStore.assets.map(a => ({
    label: a.title || a.address || '未命名',
    value: a.id
  }))
})

const userInitial = computed(() => {
  const name = appStore.userInfo?.username || 'U'
  return name.charAt(0).toUpperCase()
})

const themeTitle = computed(() => appStore.theme === 'dark' ? '切换到亮色主题' : '切换到暗色主题')

const terminalColorSchemeOptions = computed(() => {
  return getAllColorSchemes().map(scheme => ({
    label: scheme.name,
    value: scheme.id
  }))
})

const terminalColorSchemeTitle = computed(() => {
  const schemes = getAllColorSchemes()
  const current = schemes.find(s => s.id === appStore.terminalColorScheme)
  return `终端配色: ${current?.name || '默认'}`
})

const layoutTitle = computed(() => appStore.assetLayout === 'flat' ? '切换到树状布局' : '切换到扁平布局')

const previewColors = computed(() => {
  const scheme = getColorScheme(appStore.terminalColorScheme, appStore.theme)
  return {
    background: scheme.background,
    foreground: scheme.foreground,
    red: scheme.red,
    green: scheme.green,
    blue: scheme.blue,
    yellow: scheme.yellow
  }
})

function toggleTheme() {
  appStore.setTheme(appStore.theme === 'dark' ? 'light' : 'dark')
}

function toggleLayout() {
  appStore.setAssetLayout(appStore.assetLayout === 'flat' ? 'tree' : 'flat')
}

function isAssetConnected(assetId) {
  return appStore.tabs.some(t => t.assetId === assetId && t.connected)
}

function countAssets(nodes) {
  let count = 0
  for (const node of nodes) {
    if (node.type === 'asset') count += 1
    else count += countAssets(node.children || [])
  }
  return count
}

function pruneEmpty(nodes) {
  const result = []
  for (const node of nodes) {
    if (node.type === 'asset') {
      result.push(node)
      continue
    }
    const children = pruneEmpty(node.children || [])
    if (children.length === 0) continue
    result.push({ ...node, children, count: countAssets(children) })
  }
  return result
}

function nestTree(flat) {
  const items = (flat || []).map((node, index) => ({
    id: node.id,
    parentId: node.parentId || '',
    name: node.name || node.title || '',
    type: node.type || 'asset',
    address: node.address || '',
    assetId: node.assetId || node.id,
    uid: `${node.type || 'asset'}:${node.parentId || 'root'}:${node.id}:${index}`,
    children: []
  }))

  const nodeMap = new Map()
  for (const item of items) {
    if (item.type === 'node') nodeMap.set(item.id, item)
  }

  const roots = []
  for (const item of items) {
    const parent = item.parentId ? nodeMap.get(item.parentId) : null
    if (parent) parent.children.push(item)
    else roots.push(item)
  }
  return pruneEmpty(roots)
}

function nodeMatches(node, query) {
  const name = (node.name || '').toLowerCase()
  const address = (node.address || '').toLowerCase()
  const tags = node.type === 'asset'
    ? appStore.getAssetTags(node.assetId).join(' ').toLowerCase()
    : ''
  return name.includes(query) || address.includes(query) || tags.includes(query)
}

function filterTree(nodes, query) {
  const result = []
  for (const node of nodes) {
    if (node.type === 'asset') {
      if (nodeMatches(node, query)) result.push(node)
      continue
    }
    const children = filterTree(node.children || [], query)
    if (nodeMatches(node, query)) {
      result.push({ ...node, children: children.length ? children : node.children, count: countAssets(children.length ? children : node.children) })
    } else if (children.length) {
      result.push({ ...node, children, count: countAssets(children) })
    }
  }
  return result
}

const nestedTree = computed(() => {
  const nested = nestTree(appStore.assetTree)
  if (nested.length) return nested
  return (appStore.assets || []).map(asset => ({
    id: asset.id,
    parentId: '',
    name: asset.title || asset.address || '未命名',
    type: 'asset',
    address: asset.address || '',
    assetId: asset.id,
    uid: `asset:root:${asset.id}`,
    children: []
  }))
})
const isSearching = computed(() => !!searchQuery.value.trim())
const visibleTree = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return nestedTree.value
  return filterTree(nestedTree.value, query)
})

const filteredAssets = computed(() => {
  if (appStore.assetLayout === 'flat') {
    // 扁平布局：使用排序后的资产列表
    let list = appStore.getOrderedAssets()
    if (!searchQuery.value.trim()) return list

    const query = searchQuery.value.trim().toLowerCase()
    return list.filter(a => {
      const title = (a.title || '').toLowerCase()
      const address = (a.address || '').toLowerCase()
      const tags = appStore.getAssetTags(a.id).join(' ').toLowerCase()
      return title.includes(query) || address.includes(query) || tags.includes(query)
    })
  } else {
    // 树状布局：从树中提取资产 ID
    const ids = []
    const walk = (nodes) => {
      for (const node of nodes) {
        if (node.type === 'asset') ids.push(node.assetId)
        else walk(node.children || [])
      }
    }
    walk(visibleTree.value)
    return [...new Set(ids)]
  }
})

function toggleNode(id) {
  expandedIds.value = {
    ...expandedIds.value,
    [id]: !expandedIds.value[id]
  }
}

function expandRootNodes(flat) {
  const nodeIds = new Set((flat || []).filter(n => n.type === 'node').map(n => n.id))
  const next = {}
  for (const node of flat || []) {
    if (node.type === 'node' && (!node.parentId || !nodeIds.has(node.parentId))) {
      next[node.id] = true
    }
  }
  expandedIds.value = next
}

// 防止双击/快速连点同一资产时重复创建标签：
// 树节点同时绑定 click 与 dblclick，若不加保护会开出多个 Tab 并建立多条 SSH 连接
let lastOpenedAssetId = null
let lastOpenedAt = 0
const REOPEN_GUARD_MS = 400

function openTreeAsset(node) {
  const asset = appStore.assets.find(a => a.id === node.assetId) || {
    id: node.assetId,
    title: node.name,
    address: node.address,
    platform_type: 'linux'
  }
  const now = Date.now()
  if (asset.id === lastOpenedAssetId && now - lastOpenedAt < REOPEN_GUARD_MS) {
    lastOpenedAt = now
    return
  }
  lastOpenedAssetId = asset.id
  lastOpenedAt = now
  emit('openAsset', asset)
}

async function refreshAssets() {
  loading.value = true
  errorMsg.value = ''
  try {
    const result = await window.electronAPI.getAssets(
      appStore.jmsUrl,
      appStore.keyId,
      appStore.secret
    )
    if (!result.success) {
      errorMsg.value = result.error || '获取服务器列表失败'
      return
    }
    appStore.setAssets(result.assets || [], result.tree || [])
    expandRootNodes(result.tree || [])
  } catch (err) {
    errorMsg.value = `获取列表失败: ${err.message}`
  } finally {
    loading.value = false
  }
}

// ==================== 扁平布局拖拽排序 ====================
function onDragStart(event, index) {
  dragIndex.value = index
  event.dataTransfer.effectAllowed = 'move'
  event.target.classList.add('dragging')
}

function onDragOver(event, index) {
  event.preventDefault()
  dragOverIndex.value = index
}

function onDragLeave() {
  dragOverIndex.value = -1
}

function onDrop(event, index) {
  event.preventDefault()
  if (dragIndex.value === -1 || dragIndex.value === index) return

  const currentList = [...filteredAssets.value]
  const [movedItem] = currentList.splice(dragIndex.value, 1)
  currentList.splice(index, 0, movedItem)

  // 更新完整排序
  const allAssets = appStore.getOrderedAssets()
  const filteredIds = new Set(filteredAssets.value.map(a => a.id))
  const nonFiltered = allAssets.filter(a => !filteredIds.has(a.id))

  const newFullOrder = []
  let filteredIdx = 0
  let nonFilteredIdx = 0

  for (const a of allAssets) {
    if (filteredIds.has(a.id)) {
      newFullOrder.push(currentList[filteredIdx]?.id)
      filteredIdx++
    } else {
      newFullOrder.push(nonFiltered[nonFilteredIdx]?.id)
      nonFilteredIdx++
    }
  }

  appStore.updateAssetOrder(newFullOrder.filter(Boolean))
  dragOverIndex.value = -1
}

function onDragEnd(event) {
  event.target.classList.remove('dragging')
  dragIndex.value = -1
  dragOverIndex.value = -1
}

// ==================== 标签管理 ====================
function showTagInput(asset) {
  if (appStore.assetLayout === 'flat') {
    currentTagAsset.value = asset
  } else {
    currentTagAsset.value = {
      id: asset.assetId || asset.id,
      title: asset.name || asset.title,
      address: asset.address
    }
  }
  newTagValue.value = ''
  showTagModal.value = true
}

function confirmAddTag() {
  if (currentTagAsset.value && newTagValue.value.trim()) {
    appStore.addTag(currentTagAsset.value.id, newTagValue.value.trim())
  }
  showTagModal.value = false
}

function cancelAddTag() {
  showTagModal.value = false
}

// ==================== 快捷指令管理 ====================
function showAddCommandModal() {
  commandModalMode.value = 'add'
  editingCommandId.value = null
  newCommand.value = { name: '', command: '', assetId: null }
  showCommandModal.value = true
}

function editQuickCommand(cmd) {
  commandModalMode.value = 'edit'
  editingCommandId.value = cmd.id
  newCommand.value = {
    name: cmd.name,
    command: cmd.command,
    assetId: cmd.assetId
  }
  showCommandModal.value = true
}

function confirmCommand() {
  if (newCommand.value.name.trim() && newCommand.value.command.trim()) {
    if (commandModalMode.value === 'add') {
      appStore.addQuickCommand(
        newCommand.value.name.trim(),
        newCommand.value.command.trim(),
        newCommand.value.assetId
      )
      message.success('快捷指令已添加')
    } else {
      appStore.updateQuickCommand(editingCommandId.value, {
        name: newCommand.value.name.trim(),
        command: newCommand.value.command.trim(),
        assetId: newCommand.value.assetId
      })
      message.success('快捷指令已更新')
    }
  }
  cancelCommand()
}

function cancelCommand() {
  showCommandModal.value = false
  newCommand.value = { name: '', command: '', assetId: null }
  editingCommandId.value = null
}

function deleteQuickCommand(id) {
  appStore.removeQuickCommand(id)
  message.success('快捷指令已删除')
}

function executeQuickCommand(cmd) {
  if (cmd.assetId) {
    // 如果指定了资产，打开对应 Tab；命令由 TerminalPanel 在连接成功后发送
    // （不再用固定延时，避免连接慢时命令丢失）
    const asset = appStore.assets.find(a => a.id === cmd.assetId)
    if (asset) {
      emit('openAsset', asset, cmd.command)
    } else {
      message.warning('未找到指定的资产')
    }
  } else {
    // 未指定资产，检查是否有活动的终端
    const activeTabId = appStore.activeTabId
    if (activeTabId) {
      // 直接发送命令到当前活动的终端
      window.electronAPI?.sendTerminalData(activeTabId, cmd.command + '\n')
      message.success(`已发送命令: ${cmd.command}`)
    } else {
      // 没有活动终端，提示用户
      message.info('请先连接到一台服务器，然后再次点击快捷指令')
    }
  }
}

function executeQuickCommandToAll(cmd) {
  const connectedTabs = appStore.tabs.filter(t => t.connected)
  if (connectedTabs.length === 0) {
    message.info('没有已连接的终端')
    return
  }
  connectedTabs.forEach(tab => {
    window.electronAPI?.sendTerminalData(tab.id, cmd.command + '\n')
  })
  message.success(`已发送命令到 ${connectedTabs.length} 个终端: ${cmd.command}`)
}

// ==================== 监听外部打开 ====================
// 暴露 refresh 供外部调用
defineExpose({ refreshAssets })

onMounted(() => {
  refreshAssets()
})
</script>

<style scoped>
.server-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--sidebar-bg);
}

/* 搜索栏 */
.list-header {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-bottom: 1px solid var(--border-color);
}

.search-box :deep(.n-input) {
  --n-color: var(--bg-tertiary);
  --n-color-focus: var(--bg-tertiary);
  --n-text-color: var(--text-primary);
  --n-placeholder-color: var(--text-muted);
  --n-border: 1px solid var(--border-color);
  --n-border-focus: 1px solid var(--accent);
  --n-border-hover: 1px solid var(--border-color);
  --n-border-radius: 4px;
}

.header-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
}

.header-buttons {
  display: flex;
  align-items: center;
  gap: 6px;
}

.server-count {
  font-size: 11px;
  color: var(--text-muted);
}

/* 用户栏 */
.user-bar {
  display: flex;
  align-items: center;
  padding: 6px 10px;
  gap: 8px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-primary);
}

.user-avatar {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.user-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.logout-btn {
  flex-shrink: 0;
  font-size: 11px;
}

.theme-btn {
  flex-shrink: 0;
  font-size: 11px;
}

.color-scheme-btn {
  flex-shrink: 0;
  font-size: 11px;
}

.color-scheme-popover {
  padding: 8px;
  min-width: 180px;
}

.color-scheme-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.color-scheme-preview {
  margin-top: 12px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.preview-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.preview-color {
  width: 16px;
  height: 16px;
  border-radius: 3px;
  border: 1px solid var(--border-color);
  flex-shrink: 0;
}

.preview-label {
  font-size: 11px;
  color: var(--text-muted);
}

/* 快捷指令栏 */
.quick-commands-bar {
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-primary);
}

.quick-commands-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
}

.quick-commands-title {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.quick-commands-empty {
  padding: 8px 10px;
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
}

.quick-commands-list {
  max-height: 120px;
  overflow-y: auto;
  padding: 0 6px 6px;
}

.quick-command-item {
  display: flex;
  align-items: center;
  padding: 4px 6px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.1s;
  gap: 6px;
}

.quick-command-item:hover {
  background: var(--bg-hover);
}

.quick-command-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.quick-command-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.quick-command-edit,
.quick-command-delete,
.quick-command-broadcast {
  opacity: 0;
  transition: opacity 0.1s;
  flex-shrink: 0;
}

.quick-command-item:hover .quick-command-edit,
.quick-command-item:hover .quick-command-delete,
.quick-command-item:hover .quick-command-broadcast {
  opacity: 1;
}

.quick-command-broadcast {
  color: var(--accent);
}

/* 加载 & 错误 */
.loading-area {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 0;
  color: var(--text-muted);
  font-size: 13px;
}

.error-area {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px 12px;
  color: var(--error);
  font-size: 12px;
  flex-wrap: wrap;
}

.error-icon {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--error);
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  flex-shrink: 0;
}

/* 列表 */
.list-body {
  flex: 1 1 0;
  min-height: 0;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 2px 0;
  height: 0;
}

.empty-state {
  text-align: center;
  padding: 40px 0;
  color: var(--text-muted);
  font-size: 13px;
}

/* 资产项 */
.asset-item {
  padding: 8px 10px 8px 8px;
  cursor: pointer;
  transition: background 0.1s;
  user-select: none;
  border-left: 2px solid transparent;
  position: relative;
}

.asset-item:hover {
  background: var(--bg-hover);
}

.asset-item.drag-over {
  border-top: 2px solid var(--accent);
}

.asset-status {
  position: absolute;
  left: 10px;
  top: 12px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  flex-shrink: 0;
  transition: background 0.2s;
}
.asset-status.connected {
  background: var(--success);
}

.asset-info {
  margin-left: 14px;
  min-width: 0;
  overflow: hidden;
  padding-right: 28px;
}

.asset-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.asset-icon {
  flex-shrink: 0;
  color: var(--text-muted);
}

.asset-address {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 3px;
  margin-left: 20px;
}

.asset-action {
  position: absolute;
  right: 6px;
  top: 6px;
  opacity: 0;
  transition: opacity 0.1s;
}

.asset-item:hover .asset-action {
  opacity: 1;
}

/* 标签 */
.asset-tags {
  display: flex;
  flex-wrap: wrap;
  margin-top: 4px;
  margin-left: 20px;
}

/* 加载旋转动画 */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
