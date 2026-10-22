<template>
  <div class="error-state">
    <div class="error-content">
      <h1>{{ title }}</h1>
      <p>{{ description }}</p>

      <div class="illustration-box">
        <img src="@/assets/error_general.svg" alt="" class="illustration" />
      </div>
    </div>

    <AppButton @click="emit('retry')">Probeer opnieuw</AppButton>
  </div>
</template>

<script setup lang="ts">
import AppButton from './AppButton.vue'

withDefaults(
  defineProps<{
    title?: string
    description?: string
  }>(),
  {
    title: 'Sorry, er gaat iets mis',
    description: 'Dit kan verschillende redenen hebben. Probeer het opnieuw.',
  },
)

const emit = defineEmits<{
  retry: []
}>()
</script>

<style scoped>
.error-content {
  display: flex;
  flex-direction: column;
  align-items: start;
  width: min(var(--width-content-default), 100%);
}

h1 {
  margin: 0 0 0.5rem 0;
  color: var(--color-text-primary);
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1.4;
}

p {
  margin: 0 0 1.5rem 0;
  color: var(--color-text-secondary);
  font-size: 1rem;
  line-height: 1.375;
}

.illustration-box {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
  width: 100%;
  background: var(--color-surface-tint);
  border-radius: 16px;
}

.illustration {
  margin: 48px;
  max-width: 300px;
  max-height: 214px;
}
</style>
