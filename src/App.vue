<template>
  <div id="app">
    <account-list />
    <email-receiver />
    <footer class="app-footer">
      <span>© Steven-Qiang amazonq-refill v{{ version }}</span>
      <a href="https://github.com/Steven-Qiang/amazonq-refill" target="_blank">GitHub</a>
    </footer>
  </div>
</template>

<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app';
import { onMounted, ref } from 'vue';
import AccountList from './components/AccountList.vue';
import EmailReceiver from './components/EmailReceiver.vue';

const version = ref('');

onMounted(async () => {
  version.value = await getVersion();
});
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
}

body {
  margin: 0;
  padding: 0;
}

::-webkit-scrollbar {
  display: none;
}

.app-footer {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 8px 16px;
  background: var(--el-bg-color);
  border-top: 1px solid var(--el-border-color-lighter);
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
  z-index: 100;
}

.app-footer a {
  color: var(--el-color-primary);
  text-decoration: none;
}

.app-footer a:hover {
  text-decoration: underline;
}
</style>
