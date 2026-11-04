<template>
  <AppModal v-slot="{ labelId }" size="sm" @close="emit('close')">
    <div class="content">
      <h2 :id="labelId">Deze wallet is niet actief</h2>
      <p>Deze wallet is ingetrokken.</p>
      <p>Kies een andere wallet.</p>
    </div>

    <div class="footer">
      <button type="button" class="action" @click="emit('close')">Begrepen</button>
    </div>
  </AppModal>
</template>

<script setup lang="ts">
import AppModal from '@/components/ui/AppModal.vue'

const emit = defineEmits<{
  close: []
}>()
</script>

<style scoped>
.content {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.5rem;
}

h2 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.5;
  letter-spacing: 0.01125rem;
}

p {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1rem;
  line-height: 1.375;
}

.footer {
  display: flex;
  justify-content: flex-end;
  padding: 0.5rem;
}

.action {
  min-width: 3.25rem;
  min-height: 3.25rem;
  padding: 0 0.5rem;
  border: none;
  border-radius: 12px;
  background: none;
  color: var(--color-primary);
  font: inherit;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
  letter-spacing: 0.03125rem;
  text-transform: uppercase;
  cursor: pointer;
}

.action:hover {
  background: var(--color-surface-tint);
}
</style>
