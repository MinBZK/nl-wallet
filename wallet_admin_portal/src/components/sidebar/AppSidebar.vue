<template>
  <aside class="sidebar">
    <SidebarBrand />

    <div class="nav-section">
      <div class="section-label">ALGEMEEN</div>

      <RouterLink to="/tasks" class="nav-item" exact-active-class="active">
        <img src="@/assets/icons/checklist.svg" alt="" class="icon" />
        <span>Openstaande taken</span>
      </RouterLink>

      <RouterLink to="/history" class="nav-item" exact-active-class="active">
        <img src="@/assets/icons/history.svg" alt="" class="icon" />
        <span>Taakgeschiedenis</span>
      </RouterLink>
    </div>

    <SidebarUserCard name="Joss van Leiden" role="Teamleider" />
  </aside>
</template>

<script setup lang="ts">
import SidebarBrand from './SidebarBrand.vue'
import SidebarUserCard from './SidebarUserCard.vue'
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  border-right: 2px solid var(--color-border);
}

.nav-section {
  padding: 16px;
}

.section-label {
  font-size: 12px;
  font-weight: 700;
  color: #6b7280;
  margin: 8px 0 10px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  border: 1px solid transparent;
  background: transparent;
  padding: 14px 8px;
  border-radius: 8px;
  color: #23314d;
  font-weight: 600;
  text-align: left;
  text-decoration: none;
  cursor: pointer;
}

.nav-item.active {
  border-color: #5f6bf3;
  background: #f5f7ff;
  box-shadow: inset 0 0 0 1px rgba(95, 107, 243, 0.12);
}

.icon {
  width: 24px;
  height: 24px;
}
</style>
