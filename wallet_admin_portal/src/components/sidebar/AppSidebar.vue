<template>
  <aside class="sidebar">
    <SidebarBrand />

    <nav class="nav-section">
      <div class="section-label">ALGEMEEN</div>

      <RouterLink
        v-for="item in navItems"
        :key="item.name"
        :to="{ name: item.name }"
        class="nav-item"
        exact-active-class="active"
      >
        <img :src="item.icon" alt="" class="icon" />
        <span>{{ item.title }}</span>
      </RouterLink>
    </nav>

    <SidebarUserCard
      :name="loggedInUser?.displayName ?? ''"
      :role="loggedInUser?.role ?? Role.Unknown"
    />
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useAuth } from '@/composables/authentication.ts'
import { navItemsFor } from '@/router/access.ts'
import { Role } from '@/types/roles.ts'
import SidebarBrand from './SidebarBrand.vue'
import SidebarUserCard from './SidebarUserCard.vue'

const { loggedInUser } = useAuth()
const navItems = computed(() => (loggedInUser.value ? navItemsFor(loggedInUser.value) : []))
</script>

<style scoped>
.sidebar {
  border-right: 2px solid var(--color-border);
}

.nav-section {
  grid-row: 2;
  min-height: 0;
  padding: 1rem;
  overflow-y: auto;
}

.section-label {
  font-size: 0.75rem;
  font-weight: 700;
  color: var(--color-text-muted);
  margin: 0.5rem 0 0.625rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.625rem;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.875rem 0.5rem;
  border-radius: 8px;
  color: var(--color-text-primary);
  font-weight: 600;
  text-align: left;
  text-decoration: none;
  cursor: pointer;
}

.nav-item.active {
  border-color: var(--color-primary);
  background: var(--color-surface-tint);
  box-shadow: inset 0 0 0 1px rgba(56, 62, 222, 0.12);
}

.icon {
  width: 1.25em;
  text-align: center;
  color: var(--color-primary);
}
</style>
