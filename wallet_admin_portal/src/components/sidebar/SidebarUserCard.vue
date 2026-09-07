<template>
  <div ref="cardRef" class="user-card">
    <img v-if="avatarUrl" class="avatar avatar-image" :src="avatarUrl" alt="" />
    <div v-else class="avatar avatar-initials" aria-hidden="true">{{ initials }}</div>
    <div class="user-meta">
      <div class="user-name">{{ name }}</div>
      <div class="user-role">{{ role }}</div>
    </div>
    <ChevronToggleButton v-model:isOpen="isOpen" ariaLabel="Open profiel" />

    <div v-if="isOpen" class="popover" role="menu">
      <div class="popover-title">Profiel</div>
      <button type="button" class="popover-action">
        <img src="@/assets/icons/account_box.svg" alt="" />
        <span>Foto aanpassen</span>
      </button>
      <AppButton variant="secondary" @click="logout">Uitloggen</AppButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import ChevronToggleButton from '@/components/ui/ChevronToggleButton.vue'

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

const isOpen = ref(false)
const cardRef = ref<HTMLElement | null>(null)

function handleClickOutside(event: MouseEvent) {
  if (isOpen.value && cardRef.value && !cardRef.value.contains(event.target as Node)) {
    isOpen.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickOutside))
onBeforeUnmount(() => document.removeEventListener('click', handleClickOutside))

function logout() {
  window.location.href = '/auth/logout'
}
</script>

<style scoped>
.user-card {
  position: relative;
  display: flex;
  align-items: center;
  gap: 1rem;
  box-sizing: border-box;
  grid-row: 3;
  border-top: 2px solid var(--color-border);
  padding: 1rem;
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
  color: var(--color-primary);
  font-size: 0.8125rem;
  font-weight: 700;
}

.user-meta {
  flex: 1;
}

.user-name {
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 1rem;
  line-height: 1.375;
}

.user-role {
  font-weight: 400;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4286;
  margin-top: 0.125rem;
}

.popover {
  position: absolute;
  left: 16px;
  right: 16px;
  bottom: calc(100% + 16px);
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
  background: #fff;
  border-radius: 10px;
  padding: 1rem;
  box-shadow: 0 8px 24px rgba(21, 42, 98, 0.14);
  z-index: 10;
}

.popover-title {
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 1rem;
}

.popover-action {
  display: flex;
  color: var(--color-text-primary);
  align-items: center;
  gap: 0.625rem;
  border: none;
  background: none;
  padding: 0;
  font-weight: 700;
  font-size: 0.875rem;
  cursor: pointer;
  text-align: left;
}
</style>
