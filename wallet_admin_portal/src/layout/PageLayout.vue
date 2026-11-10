<template>
  <div class="shell">
    <AppSidebar class="area-sidebar" />
    <main class="area-main">
      <TopHeader
        v-if="!route.meta.hideHeader"
        :show-create-task-button="!!loggedInUser?.canCreateTask && !route.meta.hideCreateTaskButton"
      />
      <div class="area-content">
        <slot />
      </div>
      <footer id="page-footer-target" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { useAuth } from '@/composables/authentication.ts'
import AppSidebar from '@/components/sidebar/AppSidebar.vue'
import TopHeader from '@/components/TopHeader.vue'

const route = useRoute()
const { loggedInUser } = useAuth()
</script>

<style scoped>
.shell {
  display: grid;
  grid-template-columns: var(--width-sidebar) 1fr;
  grid-template-rows: auto 1fr auto;
  height: 100dvh;
  overflow: hidden;
  background: var(--color-background);
}

.area-sidebar,
.area-main {
  grid-row: 1 / 4;
  display: grid;
  grid-template-rows: subgrid;
  overflow: hidden;
}

.area-main {
  grid-template-columns: minmax(0, 1fr);
  border-left: 1px solid var(--color-border);
}

.area-main > header {
  grid-row: 1;
}

.area-content {
  grid-row: 2;
  min-height: 0;
  overflow: hidden;
}

/* No page footer (e.g. pagination) teleported in: let the content use the footer row too. */
.area-content:has(+ footer:empty) {
  grid-row: 2 / 4;
}

.area-main > footer {
  grid-row: 3;
}
</style>
