<template>
  <div>
    <el-button
      v-if="isConnected && !isVisible"
      type="primary"
      circle
      size="large"
      class="email-fab"
      @click="isVisible = true"
    >
      <el-icon :size="24">
        <message />
      </el-icon>
    </el-button>

    <el-drawer
      v-model="isVisible"
      title="邮箱验证码接收"
      direction="rtl"
      size="300px"
    >
      <div class="receiver-content">
        <div class="status-section">
          <div class="status-header">
            <span class="status-label">连接状态</span>
            <el-tag :type="statusTagType" size="small" effect="plain">
              {{ statusText }}
            </el-tag>
          </div>
          <div class="email-info">
            {{ currentEmail }}
          </div>
          <div v-if="accountStore.emailStatus.lastCheckTime" class="last-check">
            最后检查: {{ formatTime(accountStore.emailStatus.lastCheckTime) }}
          </div>
        </div>

        <el-alert
          v-if="connectionError"
          type="error"
          :title="connectionError"
          :closable="false"
          show-icon
        />

        <div v-if="latestCode" class="code-card latest">
          <div class="code-header">
            最新验证码
          </div>
          <div class="code-body">
            <div class="code-number">
              {{ latestCode.code }}
            </div>
            <el-button type="primary" size="small" @click="copyCode(latestCode.code)">
              复制
            </el-button>
          </div>
          <div class="code-meta">
            <div>来自: {{ latestCode.from }}</div>
            <div>{{ formatTime(latestCode.timestamp) }}</div>
          </div>
        </div>

        <div v-if="codeHistory.length > 0" class="history-section">
          <div class="history-header">
            历史验证码
          </div>
          <div class="history-list">
            <div v-for="code in codeHistory" :key="code.timestamp" class="history-item">
              <div class="history-code">
                {{ code.code }}
              </div>
              <div class="history-time">
                {{ formatTime(code.timestamp) }}
              </div>
            </div>
          </div>
        </div>
        <el-button type="danger" plain style="width: 100%; margin-top: 16px" @click="stopReceiver">
          停止接收
        </el-button>
      </div>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import type { VerificationCode } from '../types';
import { Message } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useAccountStore } from '../stores/accounts';
import { EmailStatus } from '../types';

const accountStore = useAccountStore();
const isVisible = ref(false);
const currentEmail = ref('');
const latestCode = ref<VerificationCode | null>(null);
const codeHistory = ref<VerificationCode[]>([]);
const connectionError = ref<string | null>(null);
const startTime = ref<Date | null>(null);
const previousCodesCount = ref(0);

let emailCheckInterval: number | null = null;
let statusCheckInterval: number | null = null;

const isConnected = computed(() => {
  const status = accountStore.emailStatus.status;
  return status === EmailStatus.Connected || status === EmailStatus.Receiving;
});

const statusText = computed(() => {
  switch (accountStore.emailStatus.status) {
    case EmailStatus.Idle:
      return '未连接';
    case EmailStatus.Connecting:
      return '连接中';
    case EmailStatus.Connected:
      return '已连接';
    case EmailStatus.Receiving:
      return '接收中';
    case EmailStatus.Error:
      return '错误';
    case EmailStatus.Stopped:
      return '已停止';
    default:
      return '未知';
  }
});

const statusTagType = computed(() => {
  const status = accountStore.emailStatus.status;
  if (status === EmailStatus.Connected || status === EmailStatus.Receiving) return 'success';
  if (status === EmailStatus.Error) return 'danger';
  if (status === EmailStatus.Connecting) return 'warning';
  return 'info';
});

async function startEmailReceiver(email: string, password: string, server: string, port: number) {
  try {
    connectionError.value = null;

    if (isConnected.value) {
      await invoke('stop_email_receiver');
    }

    currentEmail.value = email;
    startTime.value = new Date();
    previousCodesCount.value = 0;
    latestCode.value = null;
    codeHistory.value = [];

    try {
      await accountStore.testEmailConnection(email, password, server, port);
    } catch (error) {
      connectionError.value = error as string;
      throw error;
    }

    await invoke('start_email_receiver', {
      email,
      password,
      server,
      port
    });

    await checkStatus();

    emailCheckInterval = setInterval(checkNewEmails, 5000);
    statusCheckInterval = setInterval(checkStatus, 3000);
  } catch (error) {
    console.error('Failed to start email receiver:', error);
    connectionError.value = error as string;
  }
}

async function checkStatus() {
  try {
    await accountStore.getEmailStatus();

    if (accountStore.emailStatus.status === EmailStatus.Error) {
      connectionError.value = accountStore.emailStatus.errorMessage || '未知错误';
    } else {
      connectionError.value = null;
    }
  } catch (error) {
    console.error('Failed to check status:', error);
  }
}

async function checkNewEmails() {
  try {
    const codes = await invoke<VerificationCode[]>('get_verification_codes');

    if (codes.length > 0) {
      const newCodes = codes.filter((code) => {
        return startTime.value && code.timestamp > startTime.value.getTime();
      });

      if (newCodes.length > 0) {
        latestCode.value = newCodes[0];
        codeHistory.value = newCodes.slice(1, 6);

        if (newCodes.length > previousCodesCount.value) {
          isVisible.value = true;
          previousCodesCount.value = newCodes.length;
        }
      }
    }
  } catch (error) {
    console.error('Failed to check emails:', error);
  }
}

function stopReceiver() {
  isVisible.value = false;
  connectionError.value = null;
  startTime.value = null;
  previousCodesCount.value = 0;
  latestCode.value = null;
  codeHistory.value = [];
  if (emailCheckInterval) {
    clearInterval(emailCheckInterval);
    emailCheckInterval = null;
  }
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval);
    statusCheckInterval = null;
  }
  invoke('stop_email_receiver');
}

function copyCode(code: string) {
  navigator.clipboard.writeText(code);
  ElMessage.success('验证码已复制');
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString();
}

onMounted(() => {
  window.addEventListener('start-email-receiver', (event: any) => {
    const { email, password, server, port } = event.detail;
    startEmailReceiver(email, password, server, port);
  });
});

onUnmounted(() => {
  if (emailCheckInterval) {
    clearInterval(emailCheckInterval);
  }
  if (statusCheckInterval) {
    clearInterval(statusCheckInterval);
  }
  window.removeEventListener('start-email-receiver', () => {});
});
</script>

<style scoped lang="scss">
.receiver-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-section {
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.status-label {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.email-info {
  font-size: 14px;
  color: var(--el-text-color-regular);
  margin-bottom: 8px;
}

.last-check {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.code-card {
  padding: 16px;
  background: var(--el-bg-color);
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
}

.code-card.latest {
  background: var(--el-color-primary-light-9);
  border-color: var(--el-color-primary-light-5);
}

.code-header {
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--el-text-color-primary);
}

.code-body {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.code-number {
  font-size: 28px;
  font-weight: bold;
  color: var(--el-color-primary);
  letter-spacing: 2px;
}

.code-meta {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
}

.history-section {
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
}

.history-header {
  font-weight: 600;
  margin-bottom: 12px;
  color: var(--el-text-color-primary);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--el-bg-color);
  border-radius: 6px;
}

.history-code {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.history-time {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}

.email-fab {
  position: fixed;
  bottom: 42px;
  right: 12px;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}
</style>
