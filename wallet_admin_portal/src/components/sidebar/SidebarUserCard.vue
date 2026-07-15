<template>
  <div class="user-card">
    <img class="avatar" :src="avatarUrl" alt="User avatar" />
    <div class="user-meta">
      <div class="user-name">{{ name }}</div>
      <div class="user-role">{{ role }}</div>
    </div>
    <div class="chevron">›</div>
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
  gap: 12px;
  padding: 14px 6px 4px;
  border-top: 1px solid #ececf3;
  margin-top: auto;
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
  font-weight: 700;
  font-size: 14px;
}

.user-role {
  color: #6c7285;
  font-size: 12px;
  margin-top: 2px;
}

.chevron {
  color: #4b55d6;
  font-size: 26px;
  line-height: 1;
}
</style>
