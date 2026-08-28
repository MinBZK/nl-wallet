<template>
  <ul v-if="Array.isArray(value)" class="row-list">
    <li v-for="item in value" :key="item">{{ item }}</li>
  </ul>
  <p v-else class="row-value">{{ value }}</p>
</template>

<script setup lang="ts">
// Renders a single value as plain text, or an array as a bulleted list.
defineProps<{
  value: string | string[]
}>()
</script>

<style scoped>
.row-value,
.row-list {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.375;
  max-width: 100%;
  overflow-wrap: break-word;
}

.row-list {
  padding-left: 1.5rem;
}
</style>
