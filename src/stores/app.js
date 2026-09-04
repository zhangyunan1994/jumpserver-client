import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

let tabIdCounter = 0
function generateTabId() {
  return `tab-${Date.now()}-${++tabIdCounter}`
}

export const useAppStore = defineStore('app', () => {
  // ==================== JumpServer 配置 ====================
  const jmsUrl = ref('')
  const keyId = ref('')
  const secret = ref('')

  // ==================== 用户信息 ====================
  const userInfo = ref(null)

  // ==================== 资产列表 ====================
  const assets = ref([])
  const assetTree = ref([])

  // ==================== 快捷指令 ====================
  // quickCommands: [{ id, name, command, assetId }]
  const quickCommands = ref([])

  // 资产标签 { assetId: string[] }
  const assetTags = ref({})

  // 资产排序 [assetId, ...]
  const assetOrder = ref([])

  // ==================== 标签页管理 ====================
  // tabs: [{ id, title, assetId, assetTitle, assetAddress, connected }]
  const tabs = ref([])
  const activeTabId = ref(null)

  const activeTab = computed(() => {
    return tabs.value.find(t => t.id === activeTabId.value) || null
  })

  function createTab(asset) {
    // 计算同名资产的数量，用于显示序号
    const sameAssetCount = tabs.value.filter(t => t.assetId === asset.id).length
    const baseTitle = asset.title || asset.address || '未命名'
    const title = sameAssetCount > 0
      ? `${baseTitle} (${sameAssetCount + 1})`
      : baseTitle

    const tab = {
      id: generateTabId(),
      title,
      assetId: asset.id,
      assetTitle: asset.title,
      assetAddress: asset.address,
      connected: false
    }
    tabs.value.push(tab)
    activeTabId.value = tab.id
    return tab
  }

  function closeTab(tabId) {
    const idx = tabs.value.findIndex(t => t.id === tabId)
    if (idx === -1) return

    tabs.value.splice(idx, 1)

    // 如果关闭的是当前激活的 tab，切换到相邻 tab
    if (activeTabId.value === tabId) {
      if (tabs.value.length > 0) {
        const newIdx = Math.min(idx, tabs.value.length - 1)
        activeTabId.value = tabs.value[newIdx].id
      } else {
        activeTabId.value = null
      }
    }
  }

  function setActiveTab(tabId) {
    activeTabId.value = tabId
  }

  function setTabConnected(tabId, connected) {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.connected = connected
    }
  }

  // ==================== 资产布局 ====================
  const assetLayout = ref('flat') // 'flat' 或 'tree'

  // ==================== 主题设置 ====================
  const theme = ref('dark') // 'dark' 或 'light'
  const terminalColorScheme = ref('default') // 终端配色方案

  // ==================== 设置持久化 ====================
  function setSettings(settings) {
    if (settings.jms_url) jmsUrl.value = settings.jms_url
    if (settings.key_id) keyId.value = settings.key_id
    if (settings.secret) secret.value = settings.secret
    if (settings.user_info) userInfo.value = settings.user_info
    if (settings.asset_tags) assetTags.value = settings.asset_tags
    if (settings.asset_order) assetOrder.value = settings.asset_order
    if (settings.asset_layout) assetLayout.value = settings.asset_layout
    if (settings.theme) theme.value = settings.theme
    if (settings.terminal_color_scheme) terminalColorScheme.value = settings.terminal_color_scheme
    if (settings.quick_commands) quickCommands.value = settings.quick_commands
  }

  function clearSettings() {
    jmsUrl.value = ''
    keyId.value = ''
    secret.value = ''
    userInfo.value = null
    assets.value = []
    assetTree.value = []
    assetTags.value = {}
    assetOrder.value = []
    assetLayout.value = 'flat'
    tabs.value = []
    activeTabId.value = null
    theme.value = 'dark'
    terminalColorScheme.value = 'default'
    quickCommands.value = []
  }

  function setUserInfo(user) {
    userInfo.value = user
  }

  function setAssets(list, tree = []) {
    assets.value = list
    assetTree.value = Array.isArray(tree) ? tree : []
    if (assetOrder.value.length === 0) {
      assetOrder.value = list.map(a => a.id)
    }
  }

  function updateAssetOrder(newOrder) {
    assetOrder.value = newOrder
    saveAssetOrder()
  }

  function getOrderedAssets() {
    if (assetOrder.value.length === 0) return assets.value
    const orderMap = new Map(assetOrder.value.map((id, idx) => [id, idx]))
    return [...assets.value].sort((a, b) => {
      const aIdx = orderMap.get(a.id)
      const bIdx = orderMap.get(b.id)
      if (aIdx === undefined && bIdx === undefined) return 0
      if (aIdx === undefined) return 1
      if (bIdx === undefined) return -1
      return aIdx - bIdx
    })
  }

  // ==================== 标签管理 ====================
  function addTag(assetId, tag) {
    if (!assetTags.value[assetId]) {
      assetTags.value[assetId] = []
    }
    if (!assetTags.value[assetId].includes(tag)) {
      assetTags.value[assetId].push(tag)
      saveTags()
    }
  }

  function removeTag(assetId, tag) {
    if (assetTags.value[assetId]) {
      assetTags.value[assetId] = assetTags.value[assetId].filter(t => t !== tag)
      saveTags()
    }
  }

  function getAssetTags(assetId) {
    return assetTags.value[assetId] || []
  }

  async function saveTags() {
    await window.electronAPI.saveSettings({ asset_tags: assetTags.value })
  }

  async function saveAssetOrder() {
    await window.electronAPI.saveSettings({ asset_order: assetOrder.value })
  }

  async function setAssetLayout(layout) {
    assetLayout.value = layout
    await window.electronAPI.saveSettings({ asset_layout: layout })
  }

  // ==================== 快捷指令管理 ====================
  function addQuickCommand(name, command, assetId = null) {
    const newCommand = {
      id: `cmd-${Date.now()}`,
      name,
      command,
      assetId
    }
    quickCommands.value.push(newCommand)
    saveQuickCommands()
    return newCommand
  }

  function updateQuickCommand(id, updates) {
    const cmd = quickCommands.value.find(c => c.id === id)
    if (cmd) {
      Object.assign(cmd, updates)
      saveQuickCommands()
    }
  }

  function removeQuickCommand(id) {
    quickCommands.value = quickCommands.value.filter(c => c.id !== id)
    saveQuickCommands()
  }

  async function saveQuickCommands() {
    await window.electronAPI.saveSettings({ quick_commands: quickCommands.value })
  }

  async function setTheme(newTheme) {
    theme.value = newTheme
    await window.electronAPI.saveSettings({ theme: newTheme })
  }

  async function setTerminalColorScheme(scheme) {
    terminalColorScheme.value = scheme
    await window.electronAPI.saveSettings({ terminal_color_scheme: scheme })
  }

  return {
    // 配置
    jmsUrl, keyId, secret,
    // 用户 & 资产
    userInfo, assets, assetTree, assetTags, assetOrder, assetLayout,
    // Tab 管理
    tabs, activeTabId, activeTab,
    createTab, closeTab, setActiveTab,
    setTabConnected,
    // 设置
    setSettings, clearSettings,
    setUserInfo, setAssets,
    updateAssetOrder, getOrderedAssets, setAssetLayout,
    addTag, removeTag, getAssetTags,
    // 主题
    theme, setTheme,
    terminalColorScheme, setTerminalColorScheme,
    // 快捷指令
    quickCommands,
    addQuickCommand, updateQuickCommand, removeQuickCommand,
    saveQuickCommands
  }
})
