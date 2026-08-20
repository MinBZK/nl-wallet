<template>
  <button
    type="button"
    class="chevron-toggle"
    :aria-haspopup="ariaHaspopup"
    :aria-expanded="isOpen"
    :aria-label="ariaLabel"
    @click="isOpen = !isOpen"
  >
    <img src="@/assets/icons/chevron_forward.svg" alt="" :class="{ 'chevron-icon-open': isOpen }" />
  </button>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    ariaLabel: string
    ariaHaspopup?: boolean
  }>(),
  {
    ariaHaspopup: true,
  },
)

const isOpen = defineModel<boolean>('isOpen', { required: true })
</script>

<style scoped>
.chevron-toggle {
  display: flex;
  width: 1.5rem;
  height: 1.5rem;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
}

.chevron-toggle img {
  transition: transform 0.15s ease-in-out;
}

.chevron-toggle img.chevron-icon-open {
  transform: rotate(90deg);
}
</style>
