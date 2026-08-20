<template>
  <div class="placeholder">
    <p>Deze pagina is nog in ontwikkeling.</p>
    <p class="action-type">Actie: {{ type }}</p>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()
const type = computed(() => route.params.type)
</script>

<style scoped>
.placeholder {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--color-text-secondary);
  font-size: 1rem;
  line-height: 1.375;
  text-align: center;
}

.action-type {
  font-weight: 700;
  color: var(--color-text-primary);
}
</style>
