import type { Account, EmailReceiverStatus } from '../types';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import { EmailStatus } from '../types';

export const useAccountStore = defineStore('accounts', () => {
  const accounts = ref<Account[]>([]);
  const selectedAccount = ref<Account | null>(null);
  const emailStatus = ref<EmailReceiverStatus>({
    status: EmailStatus.Idle,
    errorMessage: undefined,
    lastCheckTime: undefined,
    codesCount: 0,
  });
  const isLoadingAccounts = ref(false);
  const accountError = ref<string | null>(null);

  const loadAccounts = async () => {
    isLoadingAccounts.value = true;
    accountError.value = null;
    try {
      const data = await invoke<Account[]>('get_accounts');
      accounts.value = data;
      // accounts.value = new Array(100).fill(data[0]); // 测试
    } catch (error) {
      accountError.value = error as string;
      console.error('Failed to load accounts:', error);
      throw error;
    } finally {
      isLoadingAccounts.value = false;
    }
  };

  const saveAccount = async (account: Account) => {
    accountError.value = null;
    try {
      await invoke('save_account', { account });
      await loadAccounts();
    } catch (error) {
      accountError.value = error as string;
      console.error('Failed to save account:', error);
      throw error;
    }
  };

  const deleteAccount = async (id: string) => {
    accountError.value = null;
    try {
      await invoke('delete_account', { id });
      await loadAccounts();
    } catch (error) {
      accountError.value = error as string;
      console.error('Failed to delete account:', error);
      throw error;
    }
  };

  const updateLastLogin = async (id: string) => {
    accountError.value = null;
    try {
      await invoke('update_last_login', { id });
      await loadAccounts();
    } catch (error) {
      accountError.value = error as string;
      console.error('Failed to update last login:', error);
      throw error;
    }
  };

  const getEmailStatus = async () => {
    try {
      const status = await invoke<EmailReceiverStatus>('get_email_receiver_status');
      emailStatus.value = status;
      return status;
    } catch (error) {
      console.error('Failed to get email status:', error);
      throw error;
    }
  };

  const testEmailConnection = async (
    email: string,
    password: string,
    server: string,
    port: number
  ) => {
    try {
      const result = await invoke<string>('test_email_connection', {
        email,
        password,
        server,
        port,
      });
      return result;
    } catch (error) {
      console.error('Email connection test failed:', error);
      throw error;
    }
  };

  const getDaysUntilExpiry = (account: Account): number | null => {
    if (!account.lastLoginTime) return null;
    const expiryDate = dayjs(account.lastLoginTime).add(1, 'month');
    return expiryDate.diff(dayjs(), 'day');
  };

  return {
    accounts,
    selectedAccount,
    emailStatus,
    isLoadingAccounts,
    accountError,
    loadAccounts,
    saveAccount,
    deleteAccount,
    updateLastLogin,
    getEmailStatus,
    testEmailConnection,
    getDaysUntilExpiry,
  };
});
