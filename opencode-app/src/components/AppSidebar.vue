<template>
  <aside class="sidebar">
    <div class="logo">
      <span class="logo-icon">&lt;/&gt;</span>
      <span class="logo-text">编码智能体</span>
    </div>
    <button class="btn-new" @click="createAndOpen">+ 新建对话</button>
    <ul class="session-list">
      <li
        v-for="s in sessions"
        :key="s.id"
        :class="['session-item', { active: $route.params.id === s.id }]"
        @click="$router.push('/session/' + s.id)"
      >
        <span class="session-icon">💬</span>
        <span class="session-title">{{ sessionTitle(s) }}</span>
        <span class="session-date">{{ formatDate(s.updated_at) }}</span>
      </li>
    </ul>
    <div class="sidebar-footer">
      <button class="btn-settings" @click="$router.push('/settings')">
        <span class="icon">⚙</span>
        设置
      </button>
    </div>
  </aside>
</template>

<script setup>
import { ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const route = useRoute()
const sessions = ref([])

function sessionTitle(s) {
  return s.id.slice(0, 8) + '…'
}

function formatDate(updatedAt) {
  if (!updatedAt) return ''
  const str = String(updatedAt)
  const today = new Date().toISOString().slice(0, 10)
  if (str.startsWith(today.slice(0, 4))) {
    const d = str.slice(0, 10)
    if (d === today) return '今天'
    const yesterday = new Date(Date.now() - 864e5).toISOString().slice(0, 10)
    if (d === yesterday) return '昨天'
  }
  return str.length >= 10 ? str.slice(0, 10) : str
}

async function loadSessions() {
  try {
    sessions.value = await invoke('list_sessions')
  } catch (e) {
    console.error(e)
  }
}

async function createAndOpen() {
  try {
    const id = await invoke('create_session')
    await loadSessions()
    router.push('/session/' + id)
  } catch (e) {
    console.error(e)
  }
}

onMounted(loadSessions)
watch(() => route.path, () => loadSessions())
</script>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  height: 100vh;
  background: var(--color-sidebar-bg);
  color: var(--color-sidebar-text);
  display: flex;
  flex-direction: column;
  font-size: var(--text-body);
}
.logo {
  padding: var(--space-4) var(--space-4) var(--space-3);
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.logo-icon { font-family: var(--font-mono); color: var(--color-logo-accent); font-size: var(--text-xl); }
.logo-text { font-weight: var(--font-semibold); }
.btn-new {
  margin: 0 var(--space-4) var(--space-4);
  padding: 0.6rem var(--space-4);
  background: var(--color-primary);
  color: var(--color-primary-text);
  border: none;
  border-radius: var(--radius-lg);
  cursor: pointer;
  font-size: var(--text-body);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
}
.btn-new:hover { background: var(--color-primary-hover); }
.session-list {
  list-style: none;
  margin: 0;
  padding: 0;
  flex: 1;
  overflow: auto;
}
.session-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0.6rem var(--space-4);
  cursor: pointer;
  border-radius: var(--radius-md);
  margin: 0 var(--space-2);
}
.session-item:hover { background: var(--color-sidebar-hover); }
.session-item.active { background: var(--color-primary); color: var(--color-primary-text); }
.session-icon { font-size: var(--text-base); opacity: 0.9; }
.session-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.session-date { font-size: var(--text-sm); opacity: 0.85; }
.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--color-sidebar-border);
}
.btn-settings {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  width: 100%;
  padding: var(--space-2) var(--space-3);
  background: transparent;
  border: none;
  color: var(--color-sidebar-text);
  cursor: pointer;
  border-radius: var(--radius-md);
  font-size: var(--text-body);
}
.btn-settings:hover { background: var(--color-sidebar-hover); }
.icon { font-size: var(--text-lg); }
</style>
