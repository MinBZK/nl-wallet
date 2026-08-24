<template>
  <div class="shell">
    <AppSidebar class="area-sidebar" />
    <main class="area-main">
      <TopHeader
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
  grid-template-columns: var(--sidebar-width) 1fr;
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

.area-main > footer {
  grid-row: 3;
}
</style>
