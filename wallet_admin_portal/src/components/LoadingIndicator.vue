<template>
  <div class="loading-indicator">
    <div class="spinner" role="status" aria-label="Laden" />
  </div>
</template>

<style scoped>
.loading-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  box-sizing: border-box;
  width: 2.75rem;
  height: 2.75rem;
  margin: 1rem;
}

.spinner {
  width: 100%;
  height: 100%;
  border: 4px solid transparent;
  border-top-color: var(--color-primary);
  border-radius: 50%;
  animation: spin 0.75s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
