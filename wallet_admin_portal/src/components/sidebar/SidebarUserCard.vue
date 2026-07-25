<template>
  <div class="user-card">
    <img class="avatar" :src="avatarUrl" alt="User avatar" />
    <div class="user-meta">
      <div class="user-name">{{ name }}</div>
      <div class="user-role">{{ role }}</div>
    </div>
    <div class="chevron">
      <img src="@/assets/icons/chevron_forward.svg" alt="Open Profile" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  name: string
  role: string
  avatarUrl: string
}>()
</script>

<style scoped>
.user-card {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: auto;
  box-sizing: border-box;
  height: var(--sidebar-footer-height);
  border-top: 2px solid var(--color-border);
  padding: 16px;
}

.avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  object-fit: cover;
}

.user-meta {
  flex: 1;
}

.user-name {
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 16px;
  line-height: 22px;
}

.user-role {
  font-weight: 400;
  color: var(--color-text-secondary);
  font-size: 14px;
  line-height: 20px;
  margin-top: 2px;
}

.chevron {
  display: flex;
  width: 24px;
  height: 24px;
}
</style>
