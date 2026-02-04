<template>
  <div class="settings">
    <header class="header">
      <button @click="$router.push('/')">← Back</button>
      <h2>Settings</h2>
    </header>
    <div class="section" v-if="workspacePath">
      <p class="workspace-info">Config and sessions for: <strong>{{ workspacePath }}</strong></p>
    </div>
    <div class="section">
      <h3>Provider</h3>
      <p v-if="!providers.length && !showProviderForm">No providers configured for this workspace. Add a provider below; it will be saved in this project's <code>.opencode</code> folder.</p>
      <ul v-if="providers.length && !showProviderForm" class="provider-list">
        <li v-for="(p, index) in providers" :key="p.id" class="provider-row">
          <span class="provider-summary">
            <span v-if="index === 0" class="default-badge">Default</span>
            {{ p.id }} — {{ p.provider_type }} {{ p.model || '' }}
            <span v-if="p.base_url" class="base-url">({{ p.base_url }})</span>
          </span>
          <span class="provider-actions">
            <button type="button" class="btn-small" @click="editProvider(p)">Edit</button>
            <button v-if="index !== 0" type="button" class="btn-small" @click="setDefault(p.id)">Set as default</button>
          </span>
        </li>
      </ul>
      <div v-if="!showProviderForm" class="form-actions">
        <button type="button" class="btn-primary" @click="openAddForm">Add provider</button>
      </div>

      <div v-if="showProviderForm" class="provider-form">
        <h4>{{ editingProviderId !== null ? 'Edit provider' : 'Add provider' }}</h4>
        <form @submit.prevent="saveProvider">
          <div class="form-group">
            <label for="provider-id">Provider ID</label>
            <input id="provider-id" v-model="form.providerId" type="text" required placeholder="e.g. default" :readonly="editingProviderId !== null" />
            <span v-if="editingProviderId !== null" class="form-hint">ID cannot be changed when editing.</span>
          </div>
          <div class="form-group">
            <label for="provider-type">Provider type</label>
            <select id="provider-type" v-model="form.providerType" required>
              <option v-for="t in providerTypes" :key="t" :value="t">{{ t }}</option>
            </select>
          </div>
          <div class="form-group">
            <label for="api-key">API key</label>
            <input id="api-key" v-model="form.apiKey" type="password" autocomplete="off" :placeholder="editingProviderId ? 'Leave blank to keep current key' : ''" />
            <span v-if="editingProviderId === null && (form.providerType === 'openai' || form.providerType === 'anthropic')" class="form-hint">Required for OpenAI and Anthropic.</span>
            <span v-if="editingProviderId !== null" class="form-hint">Leave blank to keep current key.</span>
          </div>
          <div class="form-group">
            <label for="base-url">Base URL (optional)</label>
            <input id="base-url" v-model="form.baseUrl" type="text" placeholder="e.g. https://api.openai.com/v1" />
          </div>
          <div class="form-group">
            <label for="model">Model (optional)</label>
            <input id="model" v-model="form.model" type="text" placeholder="e.g. gpt-4o, claude-3-5-sonnet" />
          </div>
          <div class="form-actions">
            <button type="submit" class="btn-primary">Save</button>
            <button type="button" class="btn-secondary" @click="closeForm">Cancel</button>
          </div>
        </form>
      </div>
    </div>
    <div class="section">
      <h3>Agent</h3>
      <p>Current: <strong>{{ currentAgent }}</strong></p>
      <select v-model="selectedAgent" @change="setAgent">
        <option v-for="a in agents" :key="a" :value="a">{{ a }}</option>
      </select>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const PROVIDER_TYPES = ['openai', 'ollama', 'anthropic']

const workspacePath = ref('')
const providers = ref([])
const agents = ref([])
const currentAgent = ref('build')
const selectedAgent = ref('build')
const showProviderForm = ref(false)
const editingProviderId = ref(null)
const providerTypes = ref(PROVIDER_TYPES)

const form = ref({
  providerId: '',
  providerType: 'openai',
  apiKey: '',
  baseUrl: '',
  model: ''
})

function resetForm() {
  form.value = {
    providerId: '',
    providerType: 'openai',
    apiKey: '',
    baseUrl: '',
    model: ''
  }
  editingProviderId.value = null
}

function openAddForm() {
  resetForm()
  showProviderForm.value = true
}

function editProvider(p) {
  form.value = {
    providerId: p.id,
    providerType: p.provider_type,
    apiKey: '',
    baseUrl: p.base_url ?? '',
    model: p.model ?? ''
  }
  editingProviderId.value = p.id
  showProviderForm.value = true
}

function closeForm() {
  showProviderForm.value = false
  resetForm()
}

function validateForm() {
  if (editingProviderId.value === null) {
    if (form.value.providerType === 'openai' || form.value.providerType === 'anthropic') {
      if (!form.value.apiKey.trim()) {
        return 'API key is required for OpenAI and Anthropic.'
      }
    }
  }
  return null
}

async function saveProvider() {
  const err = validateForm()
  if (err) {
    alert(err)
    return
  }
  try {
    await invoke('set_provider_config', {
      providerId: form.value.providerId,
      providerType: form.value.providerType,
      apiKey: form.value.apiKey,
      baseUrl: form.value.baseUrl || null,
      model: form.value.model || null
    })
    await load()
    closeForm()
  } catch (e) {
    console.error(e)
    alert(String(e))
  }
}

async function setDefault(id) {
  try {
    await invoke('set_default_provider', { id })
    await load()
  } catch (e) {
    console.error(e)
    alert(String(e))
  }
}

async function load() {
  try {
    const config = await invoke('get_config')
    workspacePath.value = config?.workspace_path ?? ''
    providers.value = await invoke('get_providers')
    agents.value = await invoke('list_agents')
    currentAgent.value = await invoke('get_current_agent')
    selectedAgent.value = currentAgent.value
  } catch (e) {
    console.error(e)
  }
}

async function setAgent() {
  try {
    await invoke('set_agent', { name: selectedAgent.value })
    currentAgent.value = selectedAgent.value
  } catch (e) {
    console.error(e)
  }
}

onMounted(load)
</script>

<style scoped>
.settings { padding: var(--space-4); }
.header { margin-bottom: var(--space-4); }
.section { margin-bottom: var(--space-6); }
.section h3 { margin: 0 0 var(--space-2) 0; }
.workspace-info { font-size: var(--text-base); margin: 0 0 var(--space-2) 0; }
.workspace-info code { background: var(--color-bg-code); padding: var(--space-1) var(--space-2); border-radius: var(--radius-sm); }
select { padding: var(--space-1) var(--space-2); }

.provider-list { list-style: none; padding: 0; margin: 0 0 var(--space-2) 0; }
.provider-row { display: flex; align-items: center; justify-content: space-between; padding: var(--space-2) 0; border-bottom: 1px solid var(--color-border-input); }
.provider-summary { flex: 1; }
.default-badge { background: var(--color-badge-bg); color: var(--color-badge-text); padding: var(--space-1) var(--space-2); border-radius: var(--radius-md); font-size: var(--text-xs); margin-right: var(--space-2); }
.base-url { font-size: var(--text-base); color: var(--color-text-secondary); }
.provider-actions { display: flex; gap: var(--space-2); }
.btn-small { padding: var(--space-1) var(--space-2); font-size: var(--text-sm); cursor: pointer; }
.btn-primary { padding: var(--space-2) var(--space-3); cursor: pointer; margin-right: var(--space-2); }
.btn-secondary { padding: var(--space-2) var(--space-3); cursor: pointer; }
.form-actions { margin-top: var(--space-2); }

.provider-form { margin-top: var(--space-4); padding: var(--space-4); border: 1px solid var(--color-border-light); border-radius: var(--radius-lg); background: var(--color-main-bg); }
.provider-form h4 { margin: 0 0 var(--space-3) 0; }
.form-group { margin-bottom: var(--space-3); }
.form-group label { display: block; margin-bottom: var(--space-1); font-weight: var(--font-medium); font-size: var(--text-base); }
.form-group input, .form-group select { width: 100%; max-width: 24rem; padding: var(--space-2) var(--space-2); box-sizing: border-box; }
.form-hint { display: block; font-size: var(--text-sm); color: var(--color-text-secondary); margin-top: var(--space-1); }
</style>
