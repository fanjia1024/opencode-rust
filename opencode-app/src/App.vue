<template>
  <div class="app">
    <AppSidebar />
    <main class="main">
      <router-view />
    </main>
  </div>
</template>

<script setup>
import AppSidebar from './components/AppSidebar.vue'
</script>

<style>
* { box-sizing: border-box; }
body { margin: 0; font-family: var(--font-sans); }
.app {
  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}
.main {
  flex: 1;
  min-width: 0;
  height: 100vh;
  background: var(--color-main-bg);
  overflow: auto;
}
</style>
