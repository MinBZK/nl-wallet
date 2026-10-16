<template>
  <div class="error-page">
    <ErrorState
      description="Inloggen mislukt. Dit kan verschillende redenen hebben. Probeer het opnieuw."
      @retry="restart"
    />
  </div>
</template>

<script setup lang="ts">
import ErrorState from '@/components/ui/ErrorState.vue'

function restart() {
  window.location.href = '/'
}
</script>

<style scoped>
.error-page {
  min-height: 100vh;
  box-sizing: border-box;
  padding: 48px;
  display: flex;
  flex-direction: column;
  align-items: center;
  background: var(--color-background);
}
</style>
