<template>
  <div class="home">
    <header class="header">
      <h1>OpenCode</h1>
      <nav>
        <button @click="$router.push('/settings')">Settings</button>
        <button @click="$router.push('/help')">Help</button>
      </nav>
    </header>
    <div class="toolbar">
      <button class="primary" @click="createAndOpen">New session</button>
    </div>
    <ul class="session-list" v-if="sessions.length">
      <li
        v-for="s in sessions"
        :key="s.id"
        class="session-item"
        @click="$router.push('/session/' + s.id)"
      >
        <span class="session-id">{{ s.id.slice(0, 8) }}…</span>
        <span class="session-date">{{ s.updated_at }}</span>
      </li>
    </ul>
    <p v-else class="empty">No sessions yet. Create one to start.</p>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const sessions = ref([])

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
</script>

<style scoped>
.home { display: flex; flex-direction: column; height: 100%; }
.header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 0.75rem 1rem; border-bottom: 1px solid #ddd;
}
.header h1 { margin: 0; font-size: 1.25rem; }
.toolbar { padding: 0.5rem 1rem; }
.primary { padding: 0.5rem 1rem; cursor: pointer; }
.session-list { list-style: none; margin: 0; padding: 0; flex: 1; overflow: auto; }
.session-item {
  padding: 0.75rem 1rem; border-bottom: 1px solid #eee; cursor: pointer;
}
.session-item:hover { background: #f5f5f5; }
.session-id { font-family: monospace; margin-right: 0.5rem; }
.session-date { color: #666; font-size: 0.9rem; }
.empty { padding: 1rem; color: #666; }
</style>
