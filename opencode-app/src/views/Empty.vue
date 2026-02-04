<template>
  <div class="empty">
    <div class="empty-content">
      <p class="empty-text">创建对话开始使用</p>
      <button class="btn-create" @click="createAndOpen">+ 新建对话</button>
    </div>
  </div>
</template>

<script setup>
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()

async function createAndOpen() {
  try {
    const id = await invoke('create_session')
    router.push('/session/' + id)
  } catch (e) {
    console.error(e)
  }
}
</script>

<style scoped>
.empty {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-main-bg);
}
.empty-content {
  text-align: center;
}
.empty-text {
  color: var(--color-text-secondary);
  font-size: var(--text-lg);
  margin: 0 0 var(--space-4) 0;
}
.btn-create {
  padding: 0.6rem 1.2rem;
  background: var(--color-primary);
  color: var(--color-primary-text);
  border: none;
  border-radius: var(--radius-lg);
  cursor: pointer;
  font-size: var(--text-body);
}
.btn-create:hover { background: var(--color-primary-hover); }
</style>
