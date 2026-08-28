<template>
  <button type="button" class="app-button" :class="[variant, size]">
    <slot />
  </button>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary'
    size?: 'md' | 'lg'
  }>(),
  {
    variant: 'primary',
    size: 'md',
  },
)
</script>

<style scoped>
.app-button {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
  border: none;
  border-radius: 6px;
  font-weight: 700;
  white-space: nowrap;
  cursor: pointer;
}

.app-button.md {
  min-height: 2.75rem;
  padding: 0 0.75rem;
  font-size: 1rem;
  line-height: 1.25;
  letter-spacing: 0.03125rem;
}

.app-button.lg {
  min-height: 3rem;
  padding: 0 1.5rem;
  font-size: 1.125rem;
  line-height: 1.25;
}

.app-button.primary {
  background: var(--color-primary);
  color: #fff;
}

.app-button.primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.app-button.secondary {
  background: #fff;
  border: 1px solid var(--color-primary);
  color: var(--color-primary);
}

.app-button.secondary:hover:not(:disabled) {
  background: #f5f6ff;
}

.app-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
