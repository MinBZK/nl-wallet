<template>
  <div class="user-card">
    <img v-if="avatarUrl" class="avatar avatar-image" :src="avatarUrl" alt="" />
    <div v-else class="avatar avatar-initials" aria-hidden="true">{{ initials }}</div>
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
import { computed } from 'vue'

const props = defineProps<{
  name: string
  role: string
  avatarUrl?: string
}>()

const initials = computed(() => {
  const parts = props.name.split(' ').filter(Boolean)
  const relevant = parts.length > 1 ? parts.slice(0, 1).concat(parts.slice(-1)) : parts
  return relevant
    .map((part) => part.charAt(0))
    .join('')
    .toUpperCase()
})
</script>

<style scoped>
.user-card {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: auto;
  box-sizing: border-box;
  height: var(--footer-height);
  border-top: 2px solid var(--color-border);
  padding: 16px;
}

.avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  flex-shrink: 0;
}

.avatar-image {
  object-fit: cover;
}

.avatar-initials {
  display: flex;
  align-items: center;
  justify-content: center;
  background: #e8eaf9;
  color: #383ede;
  font-size: 13px;
  font-weight: 700;
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
