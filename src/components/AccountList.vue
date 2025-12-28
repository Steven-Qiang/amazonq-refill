<template>
  <div class="account-container">
    <div class="url-bar">
      <el-input v-model="loginUrl" placeholder="输入登录URL" size="large" :disabled="isBrowserOpen">
        <template #prefix>
          <el-icon><link-icon /></el-icon>
        </template>
      </el-input>
      <div>
        <el-button v-if="!isBrowserOpen" type="primary" size="large" :disabled="!selectedAccount" @click="handleLogin">
          开始登录
        </el-button>
        <el-button v-if="isBrowserOpen" type="success" size="large" @click="markLoginSuccess">
          完成登录
        </el-button>
        <el-button v-if="isBrowserOpen" type="danger" size="large" @click="closeBrowser">
          取消
        </el-button>
      </div>
    </div>

    <div class="account-grid">
      <el-card
        v-for="account in accounts"
        :key="account.id"
        shadow="hover"
        :class="{ selected: selectedAccount?.id === account.id }"
        @click="selectedAccount = selectedAccount?.id === account.id ? null : account"
      >
        <div class="account-content">
          <div class="email-title">
            {{ account.email }}
          </div>
          <div class="account-meta">
            <span class="last-login">{{ formatTime(account.lastLoginTime) }}</span>
            <el-tag v-if="getDaysLeft(account) !== null" :type="getExpiryTagType(account)" size="small" round>
              {{ getExpiryText(account) }}
            </el-tag>
          </div>
        </div>
        <div class="account-actions">
          <el-button link size="small" @click.stop="editAccount(account)">
            编辑
          </el-button>
          <el-button link type="danger" size="small" @click.stop="deleteAccount(account.id)">
            删除
          </el-button>
        </div>
      </el-card>

      <el-card shadow="hover" class="add-card" @click="showAddForm = true">
        <el-icon :size="48" color="#409eff">
          <plus />
        </el-icon>
        <el-text type="primary">
          添加账号
        </el-text>
      </el-card>
    </div>

    <account-form
      v-if="showAddForm || editingAccount"
      :account="editingAccount"
      @save="handleSave"
      @cancel="closeForm"
    />

    <el-dialog v-model="showLoginInfo" title="登录信息" width="450px" align-center>
      <div class="login-info">
        <div class="info-item">
          <div class="info-label">
            <el-icon><message /></el-icon>
            <span>邮箱地址</span>
          </div>
          <div class="info-content">
            <span class="value">{{ currentAccount?.email }}</span>
            <el-button type="primary" size="small" @click="copyEmail">
              <el-icon><copy-document /></el-icon>
              复制
            </el-button>
          </div>
        </div>
        <div class="info-item">
          <div class="info-label">
            <el-icon><lock /></el-icon>
            <span>登录密码</span>
          </div>
          <div class="info-content">
            <span class="value">{{ currentAccount?.password }}</span>
            <el-button type="primary" size="small" @click="copyPassword">
              <el-icon><copy-document /></el-icon>
              复制
            </el-button>
          </div>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import type { Account } from '../types';
import { CopyDocument, Link as LinkIcon, Lock, Message, Plus } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useAccountStore } from '../stores/accounts';
import AccountForm from './AccountForm.vue';

const accountStore = useAccountStore();
const { accounts, selectedAccount } = storeToRefs(accountStore);

const showAddForm = ref(false);
const editingAccount = ref<Account | null>(null);
const loginUrl = ref('https://profile.aws.amazon.com/#/profile/details');
const isBrowserOpen = ref(false);
const currentAccount = ref<Account | null>(null);
const showLoginInfo = ref(false);

onMounted(() => {
  accountStore.loadAccounts();
});

function handleLogin() {
  if (!selectedAccount.value) {
    ElMessage.warning('请先选择账号');
    return;
  }
  if (!loginUrl.value) {
    ElMessage.warning('请先输入登录URL');
    return;
  }
  openBrowser(selectedAccount.value);
}

function editAccount(account: Account) {
  editingAccount.value = { ...account };
}

async function deleteAccount(id: string) {
  await ElMessageBox.confirm('确定删除此账号？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning',
  });
  await accountStore.deleteAccount(id);
}

async function handleSave(account: Account) {
  await accountStore.saveAccount(account);
  closeForm();
}

function closeForm() {
  showAddForm.value = false;
  editingAccount.value = null;
}

async function openBrowser(account: Account) {
  currentAccount.value = account;
  isBrowserOpen.value = true;
  showLoginInfo.value = true;

  window.dispatchEvent(new CustomEvent('start-email-receiver', {
    detail: {
      email: account.email,
      password: account.emailPassword,
      server: account.smtpServer,
      port: account.smtpPort
    }
  }));

  try {
    await invoke('open_browser_window', {
      url: loginUrl.value,
      accountId: account.id
    });
  } catch (error) {
    console.error('Failed to open browser window:', error);
    isBrowserOpen.value = false;
  }
}

function copyEmail() {
  if (currentAccount.value) {
    navigator.clipboard.writeText(currentAccount.value.email);
    ElMessage.success('邮箱已复制');
  }
}

function copyPassword() {
  if (currentAccount.value) {
    navigator.clipboard.writeText(currentAccount.value.password);
    ElMessage.success('密码已复制');
  }
}

async function closeBrowser() {
  isBrowserOpen.value = false;
  currentAccount.value = null;

  try {
    await invoke('close_browser_window');
  } catch (error) {
    console.error('Failed to close browser window:', error);
  }
}

async function markLoginSuccess() {
  if (currentAccount.value) {
    try {
      await invoke('save_browser_session', {
        accountId: currentAccount.value.id
      });

      await accountStore.updateLastLogin(currentAccount.value.id);

      closeBrowser();
    } catch (error) {
      console.error('Failed to save session:', error);
    }
  }
}

function getDaysLeft(account: Account): number | null {
  return accountStore.getDaysUntilExpiry(account);
}

function getExpiryText(account: Account): string {
  if (!account.lastLoginTime) return '';
  const expiryDate = dayjs(account.lastLoginTime).add(1, 'month');
  const days = expiryDate.diff(dayjs(), 'day');
  return `还有${days}天刷新`;
}

function formatTime(time?: string): string {
  if (!time) return '从未登录';
  return dayjs(time).format('YYYY-MM-DD HH:mm:ss');
}

function getExpiryTagType(account: Account): 'primary' | 'success' | 'warning' | 'info' | 'danger' {
  const days = getDaysLeft(account);
  if (days === null) return 'info';
  if (days <= 0) return 'danger';
  if (days <= 7) return 'warning';
  return 'success';
}
</script>

<style scoped lang="scss">
.account-container {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

.url-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
}

.url-bar .el-input {
  flex: 1;
}

.account-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.el-card {
  cursor: pointer;
  transition: all 0.3s;
  height: 100%;
}

.el-card:hover {
  transform: translateY(-4px);
}

.el-card.selected {
  border: 2px solid var(--el-color-primary);
  box-shadow: 0 0 0 2px var(--el-color-primary-light-9);
}

.account-content {
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-height: 80px;
  gap: 12px;
}

.email-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.account-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.last-login {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.account-actions {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--el-border-color-lighter);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.add-card {
  border: 2px dashed var(--el-border-color);
  background: var(--el-fill-color-blank);
  :deep(.el-card__body) {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 12px;
  }
}

.add-card:hover {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}

.login-info {
  display: flex;
  flex-direction: column;
  gap: 20px;
  padding: 8px 0;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  background: var(--el-fill-color-light);
  border-radius: 8px;
  transition: all 0.3s;

  &:hover {
    background: var(--el-fill-color);
  }

  .info-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--el-text-color-primary);

    .el-icon {
      font-size: 16px;
      color: var(--el-color-primary);
    }
  }

  .info-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;

    .value {
      flex: 1;
      padding: 8px 12px;
      background: var(--el-bg-color);
      border-radius: 4px;
      font-family: 'Consolas', 'Monaco', monospace;
      font-size: 14px;
      color: var(--el-text-color-regular);
      word-break: break-all;
    }

    .el-button {
      flex-shrink: 0;
    }
  }
}
</style>
