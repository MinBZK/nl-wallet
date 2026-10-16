<template>
  <div class="card">
    <slot />
  </div>
</template>

<style scoped>
.card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  box-sizing: border-box;
  width: var(--width-task-wizard);
  max-width: 100%;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text-primary);
  text-align: start;
}
</style>
