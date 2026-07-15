<template>
  <aside class="sidebar">
    <SidebarBrand />

    <div class="nav-section">
      <div class="section-label">ALGEMEEN</div>

      <button class="nav-item active">
        <span class="icon">▦</span>
        <span>Openstaande taken</span>
      </button>

      <button class="nav-item">
        <span class="icon">⏱</span>
        <span>Taakgeschiedenis</span>
      </button>
    </div>

    <SidebarUserCard
      name="Joss van Leiden"
      role="Teamleider"
      avatar-url="https://i.pravatar.cc/80?img=12"
    />
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
  padding: 16px;
  background: #fff;
  border-right: 1px solid #e7e7ef;
}

.section-label {
  font-size: 12px;
  font-weight: 700;
  color: #6b7280;
  margin: 8px 0 10px;
}

.nav-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  border: 1px solid transparent;
  background: transparent;
  padding: 14px 12px;
  border-radius: 8px;
  color: #23314d;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}

.nav-item.active {
  border-color: #5f6bf3;
  background: #f5f7ff;
  box-shadow: inset 0 0 0 1px rgba(95, 107, 243, 0.12);
}

.icon {
  width: 20px;
  text-align: center;
  color: #4b55d6;
}
</style>
