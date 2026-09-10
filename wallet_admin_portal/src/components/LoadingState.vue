<template>
  <div class="loading-state">
    <p>{{ text }}</p>
    <LoadingIndicator />
  </div>
</template>

<script setup lang="ts">
import LoadingIndicator from './LoadingIndicator.vue'

withDefaults(
  defineProps<{
    text?: string
  }>(),
  {
    text: 'Even geduld. De gegevens worden nu opgehaald',
  },
)
</script>

<style scoped>
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1.5rem;
  padding: 1rem;
  margin: 1.5rem;
}

p {
  margin: 0;
  color: var(--color-text-primary);
  text-align: center;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
}
</style>
