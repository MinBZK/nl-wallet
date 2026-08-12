<template>
  <main v-if="isFullscreen">
    <RouterView />
  </main>
  <div v-else class="app-shell">
    <AppSidebar />
    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import AppSidebar from './components/sidebar/AppSidebar.vue'

const route = useRoute()
const isFullscreen = computed(() => !!route.meta.fullscreen)
</script>

<style scoped>
.app-shell {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background);
}

.main-content {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: var(--color-background);
  border-left: 1px solid var(--color-border);
}
</style>
