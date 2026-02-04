<template>
  <div class="help">
    <header class="header">
      <button @click="$router.push('/')">← Back</button>
      <h2>Help</h2>
    </header>
    <div class="content">
      <p><strong>OpenCode</strong> — local AI coding assistant (desktop app).</p>
      <p>• <strong>Home</strong>: list sessions, create new.</p>
      <p>• <strong>Session</strong>: chat with the agent; messages and agent log appear in the same view.</p>
      <p>• <strong>Settings</strong>: view providers and switch agent (build / plan / general).</p>
      <p>Configure API key via environment (<code>OPENAI_API_KEY</code> or <code>OPENCODE_OPENAI_API_KEY</code>) or in the app when supported.</p>
    </div>
  </div>
</template>

<style scoped>
.help { padding: 1rem; }
.header { margin-bottom: 1rem; }
.content p { margin: 0.5rem 0; }
code { background: #eee; padding: 0.1rem 0.3rem; }
</style>
