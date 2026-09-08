<template>
  <div class="setup-container">
    <div class="setup-card">
      <div class="setup-header">
        <div class="setup-logo">
          <img :src="getIconSrc(appStore.appIcon)" alt="JumpServer Client" width="48" height="48" style="border-radius: 8px;" />
        </div>
        <h1>JumpServer Client</h1>
        <p>请输入 JumpServer 的连接信息以开始使用</p>
      </div>

      <n-form ref="formRef" :model="formData" :rules="rules" label-placement="top" size="large">
        <n-form-item label="JMS URL" path="jmsUrl">
          <n-input
            v-model:value="formData.jmsUrl"
            placeholder="https://jumpserver.example.com"
            clearable
          />
        </n-form-item>

        <n-form-item label="Key ID" path="keyId">
          <n-input
            v-model:value="formData.keyId"
            placeholder="请输入 Key ID"
            clearable
          />
        </n-form-item>

        <n-form-item label="Secret" path="secret">
          <n-input
            v-model:value="formData.secret"
            type="password"
            show-password-on="click"
            placeholder="请输入 Secret"
            clearable
          />
        </n-form-item>

        <n-button
          type="primary"
          block
          size="large"
          :loading="loading"
          :disabled="loading"
          @click="handleSubmit"
        >
          {{ loading ? '验证中...' : '连接 JumpServer' }}
        </n-button>
      </n-form>

      <div v-if="errorMsg" class="error-msg">
        <n-alert type="error" :title="errorMsg" closable @close="errorMsg = ''" />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { NForm, NFormItem, NInput, NButton, NAlert, useMessage } from 'naive-ui'
import { useAppStore } from '../stores/app'
import { getIconSrc } from '../utils/icons'

const emit = defineEmits(['configured'])
const message = useMessage()
const appStore = useAppStore()

const formRef = ref(null)
const loading = ref(false)
const errorMsg = ref('')

const formData = reactive({
  jmsUrl: '',
  keyId: '',
  secret: ''
})

const rules = {
  jmsUrl: [
    { required: true, message: '请输入 JMS URL', trigger: 'blur' },
    {
      validator: (rule, value) => {
        try {
          new URL(value)
          return true
        } catch {
          return new Error('请输入有效的 URL')
        }
      },
      trigger: 'blur'
    }
  ],
  keyId: [
    { required: true, message: '请输入 Key ID', trigger: 'blur' }
  ],
  secret: [
    { required: true, message: '请输入 Secret', trigger: 'blur' }
  ]
}

async function handleSubmit() {
  errorMsg.value = ''

  try {
    await formRef.value?.validate()
  } catch {
    return
  }

  loading.value = true

  try {
    const result = await window.electronAPI.validateCredentials(
      formData.jmsUrl,
      formData.keyId,
      formData.secret
    )

    if (!result.success) {
      errorMsg.value = result.error || '验证失败，请确认连接信息是否正确'
      return
    }

    await window.electronAPI.saveSettings({
      jms_url: formData.jmsUrl,
      key_id: formData.keyId,
      secret: formData.secret,
      user_info: result.user
    })

    // 读取完整 settings（含保留的 asset_tags / asset_order），恢复用户自定义数据
    const fullSettings = await window.electronAPI.getSettings()
    appStore.setSettings(fullSettings)

    message.success(`欢迎，${result.user.name || result.user.username}`)
    emit('configured')
  } catch (err) {
    errorMsg.value = `连接失败: ${err.message}`
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.setup-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  padding: 20px;
}

.setup-card {
  width: 420px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 36px 32px;
}

.setup-header {
  text-align: center;
  margin-bottom: 28px;
}

.setup-logo {
  margin-bottom: 16px;
}

.setup-header h1 {
  font-size: 22px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.setup-header p {
  font-size: 13px;
  color: var(--text-muted);
}

.error-msg {
  margin-top: 16px;
}

/* Naive UI 暗色覆盖 */
:deep(.n-form-item-label) {
  --n-label-text-color: var(--text-secondary);
}

:deep(.n-input) {
  --n-color: var(--bg-primary) !important;
  --n-color-focus: var(--bg-primary) !important;
  --n-text-color: var(--text-primary) !important;
  --n-placeholder-color: var(--text-muted) !important;
  --n-border: 1px solid var(--border-color) !important;
  --n-border-focus: 1px solid var(--accent) !important;
  --n-border-hover: 1px solid var(--border-color) !important;
  --n-border-radius: 4px;
}

:deep(.n-button--primary-type) {
  --n-color: var(--accent);
  --n-color-hover: var(--accent-hover);
  --n-color-pressed: var(--accent);
  --n-text-color: #fff;
}
</style>
