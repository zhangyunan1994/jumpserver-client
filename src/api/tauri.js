import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const tauriAPI = {
  // ==================== 认证 ====================
  validateCredentials: (jmsUrl, keyId, secret) =>
    invoke('validate_credentials', { jmsUrl, keyId, secret }),

  // ==================== 资产 ====================
  getAssets: (jmsUrl, keyId, secret) =>
    invoke('get_assets', { jmsUrl, keyId, secret }),

  // ==================== SSH 连接（多 tab 支持）====================
  connectToAsset: (jmsUrl, keyId, secret, username, assetId, tabId, cols, rows) =>
    invoke('connect_to_asset', { jmsUrl, keyId, secret, username, assetId, tabId, cols, rows }),

  disconnectSSH: (tabId) =>
    invoke('disconnect_ssh', { tabId }),

  disconnectAllSSH: () =>
    invoke('disconnect_all_ssh'),

  sendTerminalData: (tabId, data) =>
    invoke('terminal_input', { tabId, data }),

  sendTerminalResize: (tabId, cols, rows) =>
    invoke('terminal_resize', { tabId, cols, rows }),

  onTerminalData: async (callback) => {
    return listen('terminal-data', (event) => {
      const { tabId, data } = event.payload
      callback(tabId, data)
    })
  },

  onSSHStatus: async (callback) => {
    return listen('ssh-status', (event) => {
      const { tabId, data } = event.payload
      callback(tabId, data)
    })
  },

  // ==================== 设置 ====================
  getSettings: () =>
    invoke('get_settings'),

  saveSettings: (settings) =>
    invoke('save_settings', { settings }),

  // ==================== 图标 ====================
  setAppIcon: (iconName, iconData) =>
    invoke('set_app_icon', { iconName, iconData }),
}

window.electronAPI = tauriAPI
