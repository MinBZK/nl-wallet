<template>
  <div class="error-page">
    <div class="error-content">
      <h1>Sorry, er gaat iets mis</h1>
      <p>Inloggen mislukt. Dit kan verschillende redenen hebben. Probeer het opnieuw.</p>

      <div class="illustration-box">
        <img src="@/assets/error_general.svg" alt="" class="illustration" />
      </div>
    </div>

    <button type="button" class="close-button" @click="restart">Probeer opnieuw</button>
  </div>
</template>

<script setup lang="ts">
function restart() {
  window.location.href = '/'
}
</script>

<style scoped>
.error-page {
  min-height: 100vh;
  box-sizing: border-box;
  padding: 48px;
  background: var(--color-background);
}

.error-content {
  max-width: 720px;
}

h1 {
  color: var(--color-text-primary);
  font-size: 1.25rem;
  line-height: 1.4;
  font-weight: 700;
  margin: 0 0 0.5rem 0;
}

p {
  color: var(--color-text-secondary);
  font-size: 1rem;
  line-height: 1.375;
  margin: 0 0 1.5rem 0;
}

.illustration-box {
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-surface-tint);
  border-radius: 16px;
  padding: 48px;
  margin-bottom: 24px;
}

.illustration {
  width: 300px;
  height: 214px;
}

.close-button {
  border-radius: 6px;
  background: var(--color-primary);
  color: #fff;
  font-weight: 700;
  border: none;
  font-size: 1rem;
  line-height: 1.375;
  padding: 0.75rem 1.5rem;
  cursor: pointer;
}

.close-button:hover {
  background: var(--color-primary-hover);
}
</style>
