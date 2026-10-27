<template>
  <div class="user-card">
    <img v-if="avatarUrl" class="avatar avatar-image" :src="avatarUrl" alt="" />
    <div v-else class="avatar avatar-initials" aria-hidden="true">{{ initials }}</div>
    <div class="user-meta">
      <div class="user-name">{{ name }}</div>
      <div class="user-role">{{ role }}</div>
    </div>
    <button
      ref="toggle"
      type="button"
      class="profile-toggle"
      :popovertarget="popoverId"
      aria-label="Open profiel"
    >
      <img src="@/assets/icons/chevron_forward.svg" alt="" />
    </button>

    <div ref="popover" :id="popoverId" popover class="popover" @focusout="onFocusOut">
      <div class="popover-title">Profiel</div>
      <button type="button" class="popover-action">
        <img src="@/assets/icons/account_box.svg" alt="" />
        <span>Foto aanpassen</span>
      </button>
      <AppButton variant="secondary" class="logout-button" @click="logout">Uitloggen</AppButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, useId, useTemplateRef } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'

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

const popoverId = useId()
const toggle = useTemplateRef<HTMLButtonElement>('toggle')
const popover = useTemplateRef<HTMLElement>('popover')

// Close the popover when keyboard focus leaves it
function onFocusOut(event: FocusEvent) {
  const next = event.relatedTarget
  if (!(next instanceof Node) || popover.value?.contains(next)) return
  popover.value?.hidePopover()
}

function logout() {
  window.location.href = '/admin-portal/auth/logout'
}
</script>

<style scoped>
.user-card {
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

.profile-toggle {
  anchor-name: --profile-toggle;
  display: flex;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
}

/* Rendered in the top layer, so it is not clipped by the sidebar's overflow */
.popover {
  position-anchor: --profile-toggle;
  inset: auto;
  left: calc(anchor(right) + 0.75rem);
  bottom: calc(anchor(bottom) - 1rem);
  width: 14.0625rem;
  margin: 0;
  border: none;
  gap: 0.5rem;
  background: var(--color-background);
  border-radius: 4px;
  padding: 1.5rem;
  box-shadow: 0 4px 20px 5px rgba(0, 0, 0, 0.08);
}

/* Only set display when open, so the closed popover stays hidden */
.popover:popover-open {
  display: flex;
  flex-direction: column;
}

.popover-title {
  margin-bottom: 0.5rem;
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 1rem;
  line-height: 1.375;
}

.popover-action {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-height: 3rem;
  padding: 0 1rem 0 0.5rem;
  border: none;
  border-radius: 6px;
  background: none;
  color: var(--color-text-primary);
  font-weight: 700;
  font-size: 0.875rem;
  line-height: 1.4286;
  letter-spacing: 0.03125rem;
  cursor: pointer;
  text-align: left;
}

.logout-button {
  min-height: 3rem;
}
</style>
