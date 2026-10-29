<template>
  <div class="table-wrap" role="region" aria-label="Taken" tabindex="0">
    <div class="table-inner">
      <table class="tasks-table">
        <TasksTableHeader :columns="variant === 'history' ? HISTORY_COLUMNS : OPEN_COLUMNS" />
        <tbody v-if="tasks.length">
          <tr v-for="task in tasks" :key="task.id">
            <td>{{ task.id }}</td>
            <td>{{ TASK_ACTION_INFO[task.action].title }}</td>
            <td>{{ task.purpose }}</td>
            <template v-if="isTaskHistoryEntry(task)">
              <td>{{ task.verdict }}</td>
              <td>{{ task.result }}</td>
              <td>{{ dateFormatter.format(task.completedAt) }}</td>
              <td>{{ task.completedBy }}</td>
              <td>{{ task.createdBy }}</td>
            </template>
            <template v-else>
              <td>{{ dateFormatter.format(task.createdAt) }}</td>
              <td>{{ task.createdBy }}</td>
            </template>
            <td>
              <RouterLink :to="{ name: 'task-details', params: { id: task.id } }" class="details">
                BEKIJK DETAILS
              </RouterLink>
            </td>
          </tr>
        </tbody>
      </table>
      <EmptyState v-if="!tasks.length" :title="emptyTitle" :description="emptyDescription" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Task } from '@/types/task.ts'
import { isTaskHistoryEntry, type TaskHistoryEntry } from '@/types/task-history-entry.ts'
import { TASK_ACTION_INFO } from '@/types/task-action.ts'
import { dateFormatter } from '@/utils/dateFormatters.ts'
import EmptyState from '../ui/EmptyState.vue'
import TasksTableHeader from './TasksTableHeader.vue'

defineProps<{
  variant: 'open' | 'history'
  tasks: (Task | TaskHistoryEntry)[]
  emptyTitle: string
  emptyDescription: string
}>()

const OPEN_COLUMNS = [
  { label: 'TAAK-ID' },
  { label: 'ACTIE' },
  { label: 'DOEL' },
  { label: 'AANGEMAAKT OP' },
  { label: 'AANGEMAAKT DOOR' },
  { label: 'VOLGENDE STAP', width: '8.125rem' },
]

const HISTORY_COLUMNS = [
  { label: 'TAAK-ID' },
  { label: 'ACTIE' },
  { label: 'DOEL' },
  { label: 'OORDEEL' },
  { label: 'RESULTAAT' },
  { label: 'AFGEROND OP' },
  { label: 'AFGEROND DOOR' },
  { label: 'AANGEMAAKT DOOR' },
  { label: 'VOLGENDE STAP', width: '8.125rem' },
]
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
