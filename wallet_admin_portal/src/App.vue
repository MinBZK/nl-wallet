<template>
  <div class="app-shell">
    <AppSidebar />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import AppSidebar from './components/sidebar/AppSidebar.vue'
</script>

<style scoped>
.app-shell {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100vh;
  overflow: hidden;
  background: --var(--color-background);
  font-family: 'RO Sans', sans-serif;
}

.main-content {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: --var(--color-background);
  border-left: 1px solid #e7e7ef;
}
</style>
