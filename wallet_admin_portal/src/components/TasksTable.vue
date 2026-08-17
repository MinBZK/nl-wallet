<template>
  <div class="table-wrap">
    <div class="table-inner">
      <table class="tasks-table">
        <TasksTableHeader :columns="columns" />
        <tbody v-if="tasks.length">
          <tr v-for="task in tasks" :key="task.id">
            <td>{{ task.id }}</td>
            <td>{{ task.action }}</td>
            <td>{{ task.target }}</td>
            <td>{{ task.createdAt }}</td>
            <td>{{ task.createdBy }}</td>
            <td><a href="#" class="details">BEKIJK DETAILS</a></td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-if="!tasks.length" :title="emptyTitle" :description="emptyDescription" />
    </div>
  </div>
</template>

<script setup lang="ts">
import EmptyState from './EmptyState.vue'
import TasksTableHeader from './TasksTableHeader.vue'

export interface Task {
  id: string
  action: string
  target: string
  createdAt: string
  createdBy: string
}

defineProps<{
  columns: { label: string; width?: string }[]
  tasks: Task[]
  emptyTitle: string
  emptyDescription: string
}>()
</script>

<style scoped>
.table-wrap {
  padding: 1.5rem;
  overflow: auto;
  flex: 1;
}

.table-inner {
  width: max-content;
  min-width: 100%;
}

.tasks-table {
  width: 100%;
  border-collapse: collapse;
  box-shadow: 0 1px 0 var(--color-border);
}

tbody td {
  padding: 1.375rem 1rem;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.details {
  color: var(--color-primary);
  font-weight: 700;
  text-decoration: none;
}

.details:hover {
  text-decoration: underline;
}
</style>
