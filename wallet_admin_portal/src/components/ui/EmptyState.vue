<template>
  <div class="empty-state">
    <h2>{{ title }}</h2>
    <p>{{ description }}</p>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2.5rem 1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  text-align: center;
}

h2 {
  margin: 0;
  color: var(--color-text-primary);
  font-size: 1.125rem;
  line-height: 1.3333;
  font-weight: 700;
}

p {
  margin: 0;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4286;
  font-weight: 400;
}
</style>
