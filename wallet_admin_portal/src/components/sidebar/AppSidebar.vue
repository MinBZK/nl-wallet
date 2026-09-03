<template>
  <aside class="sidebar">
    <SidebarBrand />

    <nav class="nav-section">
      <div class="section-label">ALGEMEEN</div>

      <RouterLink
        v-if="
          loggedInUser?.canCreateTask || loggedInUser?.privileges?.includes(Privilege.ShowAllTasks)
        "
        to="/tasks"
        class="nav-item"
        exact-active-class="active"
      >
        <img src="@/assets/icons/checklist.svg" alt="" class="icon" />
        <span>Openstaande taken</span>
      </RouterLink>

      <RouterLink
        v-if="loggedInUser?.canCreateTask"
        to="/my-tasks"
        class="nav-item"
        exact-active-class="active"
      >
        <img src="@/assets/icons/account_box.svg" alt="" class="icon" />
        <span>Mijn open taken</span>
      </RouterLink>

      <RouterLink to="/history" class="nav-item" exact-active-class="active">
        <img src="@/assets/icons/history.svg" alt="" class="icon" />
        <span>Taakgeschiedenis</span>
      </RouterLink>
    </nav>

    <SidebarUserCard
      :name="loggedInUser?.displayName ?? ''"
      :role="loggedInUser?.role ?? Role.Unknown"
    />
  </aside>
</template>

<script setup lang="ts">
import { useAuth } from '@/composables/authentication.ts'
import { Role } from '@/types/roles.ts'
import { Privilege } from '@/types/privilege.ts'
import SidebarBrand from './SidebarBrand.vue'
import SidebarUserCard from './SidebarUserCard.vue'

const { loggedInUser } = useAuth()
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
