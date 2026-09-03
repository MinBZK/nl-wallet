<template>
  <div v-for="row in details" :key="row.label" class="row">
    <p class="row-label">{{ row.label }}</p>
    <TaskItemList :value="row.value" />
  </div>
</template>

<script setup lang="ts">
import TaskItemList from './TaskItemList.vue'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'

defineProps<{
  details: TaskDetailRow[]
}>()
</script>

<style scoped>
.row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.row-label {
  margin: 0;
  font-size: 0.875rem;
  line-height: 1.4286;
}
</style>
