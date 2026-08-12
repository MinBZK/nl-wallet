<template>
  <thead>
    <tr>
      <th
        v-for="column in columns"
        :key="column.label"
        :style="column.width ? { width: column.width } : undefined"
      >
        {{ column.label }}
      </th>
    </tr>
  </thead>
</template>

<script setup lang="ts">
defineProps<{
  columns: { label: string; width?: string }[]
}>()
</script>

<style scoped>
thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  text-align: left;
  padding: 0.75rem 0.5rem;
  font-size: 0.875rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  line-height: 1.4286;
  color: var(--color-text-primary);
  background: var(--color-surface-tint);
  border-bottom: 2px solid var(--color-border);
  white-space: nowrap;
}

thead th:first-child {
  padding-left: 1rem;
}

thead th:last-child {
  padding-right: 1rem;
}
</style>
