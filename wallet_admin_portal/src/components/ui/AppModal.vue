<template>
  <Teleport to="body">
    <div class="overlay" @click.self="emit('close')">
      <div class="dialog" role="dialog" aria-modal="true">
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'

const emit = defineEmits<{
  close: []
}>()

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.body.style.overflow = 'hidden'
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.body.style.overflow = ''
})
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(21, 42, 98, 0.2);
  padding: 1.5rem;
  z-index: 1000;
}

.dialog {
  width: 28.8125rem;
  max-width: 100%;
  max-height: 100%;
  overflow-y: auto;
  background: var(--color-background);
  border-radius: 6px;
  box-shadow: 0 1px 15px rgba(0, 0, 0, 0.05);
}
</style>
