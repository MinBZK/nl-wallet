<template>
  <div class="step-header">
    <p class="step-count">Stap {{ stepNumber }} van {{ totalSteps }}</p>
    <h2 ref="heading" tabindex="-1">{{ title }}</h2>
    <p class="description">{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'

defineProps<{
  stepNumber: number
  totalSteps: number
  title: string
  description: string
}>()

const heading = ref<HTMLHeadingElement | null>(null)

// Each step mounts a fresh TaskStepHeader, so focusing here on mount moves focus (and screen
// reader announcement) to the new step's heading every time the wizard advances or goes back.
onMounted(() => {
  heading.value?.focus()
})
</script>

<style scoped>
.step-header {
  display: flex;
  flex-direction: column;
  align-items: start;
  gap: 0.5rem;
  width: var(--width-task-wizard);
  max-width: 100%;
  text-align: start;
}

.step-count {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.75rem;
  line-height: 1.3333;
}

h2 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1.4;
}

.description {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1rem;
  line-height: 1.375;
}
</style>
