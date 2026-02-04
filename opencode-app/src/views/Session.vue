<template>
  <div class="session">
    <header class="chat-header">
      <h1 class="chat-title">{{ sessionTitle }}</h1>
      <p class="chat-meta">{{ messages.length + (streamingContent ? 1 : 0) }} 条消息</p>
    </header>
    <div class="messages" ref="messagesRef">
      <!-- Welcome block when no messages -->
      <div v-if="!messages.length && !streamingContent" class="welcome">
        <p class="welcome-title">👋 欢迎使用编码智能体</p>
        <p class="welcome-desc">我可以帮助你编写代码、调试问题、优化性能等。</p>
        <div class="agent-pills">
          <button
            v-for="a in agents"
            :key="a"
            type="button"
            :class="['agent-pill', { active: currentAgent === a }]"
            @click="selectAgent(a)"
          >
            <span class="agent-pill-icon">{{ agentIcon(a) }}</span>
            {{ agentDisplayName(a) }}
          </button>
        </div>
      </div>
      <div
        v-for="(m, i) in messages"
        :key="i"
        :class="['msg-row', m.role === 'user' ? 'user' : 'assistant']"
      >
        <div class="avatar" :class="m.role">
          <span v-if="m.role === 'user'">👤</span>
          <span v-else>🖥</span>
        </div>
        <div class="msg-body">
          <span class="msg-meta">{{ m.role === 'user' ? '你' : '编码智能体' }} · {{ msgTime(i, m) }}</span>
          <div class="msg-content">{{ m.content }}</div>
        </div>
      </div>
      <div v-if="streamingContent" class="msg-row assistant">
        <div class="avatar assistant">🖥</div>
        <div class="msg-body">
          <span class="msg-meta">编码智能体 · {{ msgTime(-1) }}</span>
          <div class="msg-content">{{ streamingContent }}</div>
        </div>
      </div>
    </div>
    <div class="log-panel" v-if="logs.length">
      <div v-for="(l, i) in logs" :key="i" :class="['log', l.level]">{{ l.message }}</div>
    </div>
    <div class="input-area" ref="inputAreaRef">
      <div class="quick-actions">
        <button type="button" class="quick-btn" @click="setPrompt('写一个 React 组件')">&lt;/&gt; 写一个 React 组件</button>
        <button type="button" class="quick-btn" @click="setPrompt('优化这段代码')">优化这段代码</button>
        <button type="button" class="quick-btn" @click="setPrompt('解释这个函数')">&lt;&gt; 解释这个函数</button>
      </div>
      <!-- Current agent bar -->
      <div class="current-agent-bar">
        <span class="current-agent-name">
          <span class="current-agent-icon">{{ agentIcon(currentAgent) }}</span>
          {{ agentDisplayName(currentAgent) }}
        </span>
        <span class="current-agent-desc">{{ agentDescription(currentAgent) }}</span>
      </div>
      <div class="input-row">
        <textarea
          ref="inputRef"
          v-model="input"
          placeholder="输入你的问题或代码需求... (Shift+Enter 换行, Tab 切换智能体)"
          rows="2"
          @keydown="onKeydown"
          @input="onInput"
          @blur="onInputBlur"
          :disabled="loading"
        />
        <button
          type="button"
          class="btn-send"
          @click="send"
          :disabled="loading || !input.trim()"
          title="发送"
        >
          <span class="send-icon">✈</span>
        </button>
      </div>
      <p class="input-hint">按 Tab 切换 · 输入/选择</p>
      <!-- Slash command palette -->
      <div
        v-if="showCommandPalette"
        class="command-palette"
        role="listbox"
        :aria-activedescendant="filteredCommands[selectedCommandIndex]?.id"
      >
        <button
          v-for="(cmd, i) in filteredCommands"
          :key="cmd.id"
          type="button"
          :id="cmd.id"
          :class="['command-item', { active: i === selectedCommandIndex }]"
          role="option"
          @mousedown.prevent="applyCommand(cmd)"
        >
          {{ cmd.label }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, watch, nextTick, computed } from 'vue'
import { useRoute } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const route = useRoute()
const sessionId = ref(route.params.id)
const messages = ref([])
const streamingContent = ref('')
const logs = ref([])
const input = ref('')
const loading = ref(false)
const messagesRef = ref(null)
const inputRef = ref(null)
const inputAreaRef = ref(null)

const agents = ref([])
const currentAgent = ref('build')

const showCommandPalette = ref(false)
const commandFilter = ref('')
const selectedCommandIndex = ref(0)
const lastSelectionStart = ref(0)
const lastSelectionEnd = ref(0)
const pendingCommandId = ref(null)

const DEFAULT_SLASH_COMMANDS = [
  { id: 'react', label: '写一个 React 组件', insertText: '写一个 React 组件' },
  { id: 'optimize', label: '优化这段代码', insertText: '优化这段代码' },
  { id: 'explain', label: '解释这个函数', insertText: '解释这个函数' },
]
const slashCommands = ref([...DEFAULT_SLASH_COMMANDS])

const AGENT_DISPLAY_NAMES = {
  build: '通用助手',
  plan: '规划',
  general: '通用',
}
const AGENT_DESCRIPTIONS = {
  build: '解答各类编程问题',
  plan: '规划任务与拆解',
  general: '通用问答',
}
const AGENT_ICONS = {
  build: '📄',
  plan: '</>',
  general: '>_',
}

const sessionTitle = computed(() => {
  const id = sessionId.value
  return id ? id.slice(0, 8) + '…' : '对话'
})

const filteredCommands = computed(() => {
  const list = slashCommands.value
  const q = commandFilter.value.toLowerCase().trim()
  if (!q) return list
  return list.filter(
    (c) =>
      c.label.toLowerCase().includes(q) ||
      c.id.toLowerCase().includes(q)
  )
})

function agentDisplayName(id) {
  return AGENT_DISPLAY_NAMES[id] || id
}
function agentDescription(id) {
  return AGENT_DESCRIPTIONS[id] || ''
}
function agentIcon(id) {
  return AGENT_ICONS[id] || '•'
}

async function loadAgents() {
  try {
    const list = await invoke('list_agents')
    agents.value = Array.isArray(list) ? [...list].sort() : []
    const current = await invoke('get_current_agent')
    currentAgent.value = current || 'build'
  } catch (e) {
    console.error(e)
  }
}

async function selectAgent(name) {
  try {
    await invoke('set_agent', { name })
    currentAgent.value = name
  } catch (e) {
    console.error(e)
  }
}

function cycleAgent() {
  if (!agents.value.length) return
  const idx = agents.value.indexOf(currentAgent.value)
  const nextIdx = (idx + 1) % agents.value.length
  const next = agents.value[nextIdx]
  selectAgent(next)
}

function msgTime(index, m) {
  return '16:30'
}

function setPrompt(text) {
  input.value = input.value ? input.value + '\n' + text : text
  nextTick(() => inputRef.value?.focus())
}

function normalizeMessage(m) {
  return {
    role: m.role?.toLowerCase() === 'assistant' ? 'assistant' : 'user',
    content: m.content || '',
  }
}

async function loadSession() {
  try {
    const s = await invoke('get_session', { sessionId: sessionId.value })
    messages.value = (s.messages || []).map(normalizeMessage)
  } catch (e) {
    console.error(e)
  }
}

function getLineStart(value, pos) {
  const idx = value.lastIndexOf('\n', pos - 1)
  return idx === -1 ? 0 : idx + 1
}

function onInput() {
  nextTick(() => {
    const el = inputRef.value
    if (!el) return
    lastSelectionStart.value = el.selectionStart
    lastSelectionEnd.value = el.selectionEnd
    const val = input.value
    const lineStart = getLineStart(val, el.selectionStart)
    const line = val.slice(lineStart, el.selectionStart)
    if (line.startsWith('/')) {
      showCommandPalette.value = true
      commandFilter.value = line.slice(1)
      selectedCommandIndex.value = 0
    } else {
      showCommandPalette.value = false
    }
  })
}

function onInputBlur() {
  setTimeout(() => {
    showCommandPalette.value = false
  }, 150)
}

function applyCommand(cmd) {
  const el = inputRef.value
  if (!el) return
  const val = input.value
  const lineStart = getLineStart(val, lastSelectionStart.value)
  const end = lastSelectionEnd.value
  const insertText = cmd.insertText ?? cmd.label
  input.value = val.slice(0, lineStart) + insertText + val.slice(end)
  pendingCommandId.value = cmd.id
  showCommandPalette.value = false
  nextTick(() => {
    if (inputRef.value) {
      const pos = lineStart + insertText.length
      inputRef.value.setSelectionRange(pos, pos)
      inputRef.value.focus()
    }
  })
}

function onKeydown(e) {
  if (showCommandPalette.value) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedCommandIndex.value = Math.min(
        selectedCommandIndex.value + 1,
        filteredCommands.value.length - 1
      )
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedCommandIndex.value = Math.max(selectedCommandIndex.value - 1, 0)
      return
    }
    if (e.key === 'Enter') {
      e.preventDefault()
      const cmd = filteredCommands.value[selectedCommandIndex.value]
      if (cmd) applyCommand(cmd)
      return
    }
    if (e.key === 'Escape') {
      e.preventDefault()
      showCommandPalette.value = false
      return
    }
  }
  if (e.key === 'Tab') {
    e.preventDefault()
    cycleAgent()
    return
  }
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    send()
  }
}

async function send() {
  const text = input.value.trim()
  if (!text || loading.value) return
  input.value = ''
  messages.value.push({ role: 'user', content: text })
  streamingContent.value = ''
  loading.value = true
  const unlistenChunk = await listen('session-reply-chunk', (e) => {
    if (e.payload?.session_id === sessionId.value && e.payload?.content)
      streamingContent.value += e.payload.content
  })
  const unlistenDone = await listen('session-reply-done', (e) => {
    if (e.payload?.session_id === sessionId.value) {
      if (streamingContent.value) {
        messages.value.push({ role: 'assistant', content: streamingContent.value })
        streamingContent.value = ''
      }
      loading.value = false
      unlistenChunk()
      unlistenDone()
    }
  })
  const unlistenLog = await listen('session-log', (e) => {
    if (e.payload?.session_id === sessionId.value)
      logs.value.push({ level: e.payload.level || 'info', message: e.payload.message })
  })
  const commandToSend = pendingCommandId.value
  pendingCommandId.value = null
  try {
    await invoke('send_message', {
      sessionId: sessionId.value,
      input: text,
      command: commandToSend ?? undefined,
    })
  } catch (e) {
    messages.value.push({ role: 'assistant', content: 'Error: ' + e })
    loading.value = false
    unlistenChunk()
    unlistenDone()
  }
  nextTick(() => messagesRef.value?.scrollTo(0, messagesRef.value.scrollHeight))
}

async function loadCommands() {
  try {
    const list = await invoke('list_commands')
    if (Array.isArray(list) && list.length) {
      slashCommands.value = list.map((c) => ({
        id: c.id,
        label: c.label,
        insertText: c.label,
      }))
    }
  } catch (e) {
    console.error('list_commands failed', e)
  }
}

onMounted(() => {
  loadAgents()
  loadSession()
  loadCommands()
})
watch(() => route.params.id, (id) => {
  sessionId.value = id
  messages.value = []
  logs.value = []
  loadSession()
})
</script>

<style scoped>
.session { display: flex; flex-direction: column; height: 100%; background: var(--color-main-bg); }
.chat-header { padding: var(--space-4) var(--space-6); border-bottom: 1px solid var(--color-border); background: var(--color-surface); }
.chat-title { margin: 0; font-size: var(--text-2xl); font-weight: var(--font-semibold); color: var(--color-text-primary); }
.chat-meta { margin: var(--space-1) 0 0 0; font-size: var(--text-base); color: var(--color-text-secondary); }
.messages { flex: 1; overflow: auto; padding: var(--space-4) var(--space-6); }
.msg-row { display: flex; gap: var(--space-3); margin-bottom: var(--space-4); align-items: flex-start; }
.avatar {
  width: 36px; height: 36px; border-radius: 50%; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center; font-size: var(--text-lg);
}
.avatar.user { background: var(--color-border-input); color: var(--color-text-secondary); }
.avatar.assistant { background: var(--color-primary); color: var(--color-primary-text); }
.msg-body { min-width: 0; flex: 1; }
.msg-meta { font-size: var(--text-sm); color: var(--color-text-muted); display: block; margin-bottom: var(--space-1); }
.msg-content {
  white-space: pre-wrap; word-break: break-word; font-size: var(--text-body); line-height: 1.5;
  color: var(--color-text-primary);
}
.log-panel { max-height: 100px; overflow: auto; padding: var(--space-1) var(--space-4); font-size: var(--text-xs); background: var(--color-bg-log); border-top: 1px solid var(--color-border); }
.log { margin: var(--space-1) 0; }
.log.error { color: var(--color-error); }

.welcome { padding: var(--space-6) 0; text-align: center; }
.welcome-title { margin: 0 0 var(--space-2); font-size: var(--text-xl); color: var(--color-text-primary); }
.welcome-desc { margin: 0 0 var(--space-4); font-size: var(--text-body); color: var(--color-text-secondary); }
.agent-pills { display: flex; gap: var(--space-2); justify-content: center; flex-wrap: wrap; }
.agent-pill {
  display: inline-flex; align-items: center; gap: var(--space-2);
  padding: var(--space-2) var(--space-4); font-size: var(--text-base); background: var(--color-bg-muted);
  border: 1px solid var(--color-border-input); border-radius: var(--radius-xl); cursor: pointer; color: var(--color-text-primary);
}
.agent-pill:hover { background: var(--color-border-input); }
.agent-pill.active { background: var(--color-primary); color: var(--color-primary-text); border-color: var(--color-primary); }
.agent-pill-icon { font-size: var(--text-lg); }

.input-area { padding: var(--space-3) var(--space-6) var(--space-4); background: var(--color-surface); border-top: 1px solid var(--color-border); position: relative; }
.quick-actions { display: flex; gap: var(--space-2); margin-bottom: var(--space-2); flex-wrap: wrap; }
.quick-btn {
  padding: var(--space-2) var(--space-3); font-size: var(--text-base); background: var(--color-bg-muted); border: 1px solid var(--color-border-input);
  border-radius: var(--radius-lg); cursor: pointer; color: var(--color-text-primary);
}
.quick-btn:hover { background: var(--color-border-input); }
.current-agent-bar {
  display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-2);
  font-size: var(--text-sm); color: var(--color-text-secondary);
}
.current-agent-name { display: inline-flex; align-items: center; gap: var(--space-1); font-weight: var(--font-medium); color: var(--color-text-primary); }
.current-agent-icon { font-size: var(--text-base); }
.current-agent-desc { color: var(--color-text-muted); }
.input-row { display: flex; gap: var(--space-2); align-items: flex-end; }
.input-row textarea {
  flex: 1; min-height: 44px; max-height: 120px; padding: var(--space-2) var(--space-3);
  border: 1px solid var(--color-border-light); border-radius: var(--radius-xl); font-family: inherit; font-size: var(--text-body);
  resize: none;
}
.input-row textarea:focus { outline: none; border-color: var(--color-primary); }
.input-hint { margin: var(--space-1) 0 0 0; font-size: var(--text-xs); color: var(--color-text-muted); }
.btn-send {
  width: 44px; height: 44px; flex-shrink: 0; border: none; border-radius: var(--radius-xl);
  background: var(--color-primary); color: var(--color-primary-text); cursor: pointer; display: flex; align-items: center; justify-content: center;
}
.btn-send:hover:not(:disabled) { background: var(--color-primary-hover); }
.btn-send:disabled { opacity: 0.5; cursor: not-allowed; }
.send-icon { font-size: var(--text-xl); }

.command-palette {
  position: absolute; left: var(--space-6); right: var(--space-6); bottom: 100%;
  margin-bottom: var(--space-1); padding: var(--space-1); max-height: 200px; overflow: auto;
  background: var(--color-surface); border: 1px solid var(--color-border); border-radius: var(--radius-lg);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1); z-index: var(--z-dropdown);
}
.command-item {
  display: block; width: 100%; padding: var(--space-2) var(--space-3); text-align: left; font-size: var(--text-body);
  background: none; border: none; border-radius: var(--radius-md); cursor: pointer; color: var(--color-text-primary);
}
.command-item:hover { background: var(--color-bg-muted); }
.command-item.active { background: var(--color-primary); color: var(--color-primary-text); }
</style>
