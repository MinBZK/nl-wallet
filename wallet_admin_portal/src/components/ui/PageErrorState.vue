<template>
  <div class="page-error">
    <ErrorState v-bind="$attrs" />
  </div>
</template>

<script setup lang="ts">
import ErrorState from './ErrorState.vue'

defineOptions({ inheritAttrs: false })
</script>

<style scoped>
.page-error {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: safe center;
  box-sizing: border-box;
  padding: 1.5rem;
  overflow: auto;
}
</style>
