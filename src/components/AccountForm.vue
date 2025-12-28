<template>
  <el-dialog
    v-model="dialogVisible"
    :title="account ? '编辑账号' : '添加账号'"
    width="500px"
    @close="emit('cancel')"
  >
    <el-form :model="form" label-width="120px" label-position="left">
      <el-form-item label="邮箱地址" required>
        <el-input v-model="form.email" type="email" placeholder="请输入邮箱地址" />
      </el-form-item>

      <el-form-item label="密码" required>
        <el-input v-model="form.password" type="password" placeholder="请输入密码" show-password />
      </el-form-item>

      <el-form-item label="邮箱密码" required>
        <el-input v-model="form.emailPassword" type="password" placeholder="请输入邮箱密码" show-password />
      </el-form-item>

      <el-form-item label="POP3服务器" required>
        <el-input v-model="form.smtpServer" placeholder="pop.ym.163.com" />
      </el-form-item>

      <el-form-item label="POP3端口" required>
        <el-input-number v-model="form.smtpPort" :min="1" :max="65535" placeholder="995" />
        <el-text size="small" type="info" style="margin-left: 10px">
          必须使用995 SSL端口
        </el-text>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="emit('cancel')">
        取消
      </el-button>
      <el-button type="primary" @click="handleSubmit">
        保存
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import type { Account } from '../types';

interface Props {
  account?: Account | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  save: [account: Account];
  cancel: [];
}>();

const dialogVisible = ref(true);

const form = ref({
  id: '',
  email: '',
  password: '',
  emailPassword: '',
  smtpServer: 'pop.ym.163.com',
  smtpPort: 995,
});

watch(() => props.account, (account) => {
  if (account) {
    form.value = { ...account };
  } else {
    form.value = {
      id: Date.now().toString(),
      email: '',
      password: '',
      emailPassword: '',
      smtpServer: 'pop.ym.163.com',
      smtpPort: 995,
    };
  }
}, { immediate: true });

function handleSubmit() {
  emit('save', form.value as Account);
}
</script>
