<template>
  <aside class="log">
    <h3>Controleoverzicht</h3>
    <template v-for="(entry, index) in entries" :key="index">
      <hr v-if="index > 0" />
      <div class="entry">
        <h4>{{ entry.title }}</h4>
        <div v-for="row in entry.rows" :key="row.label" class="row">
          <span class="row-label">{{ row.label }}</span>
          <span class="row-value">{{ row.value }}</span>
        </div>
      </div>
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { TaskHistoryEntry } from '@/types/task-history-entry.ts'
import { dateTimeFormatter } from '@/utils/dateFormatters.ts'

/** A labeled value displayed within a log entry. */
interface TaskLogRow {
  label: string
  value: string
}

/** A titled group of rows. */
interface TaskLogEntry {
  title: string
  rows: TaskLogRow[]
}

const props = defineProps<{
  task: TaskHistoryEntry
}>()

function buildTaskLog(task: TaskHistoryEntry): TaskLogEntry[] {
  return [
    {
      title: 'Aangemaakt:',
      rows: [
        { label: 'Op:', value: dateTimeFormatter.format(task.createdAt) },
        { label: 'Door:', value: task.createdBy },
      ],
    },
    {
      title: 'Beoordeeld:',
      rows: [
        { label: 'Op:', value: dateTimeFormatter.format(task.completedAt) },
        { label: 'Oordeel:', value: task.verdict },
        { label: 'Door:', value: task.completedBy },
        ...(task.completionReason ? [{ label: 'Reden:', value: task.completionReason }] : []),
      ],
    },
    {
      title: 'Resultaat:',
      rows: [
        { label: 'Op:', value: dateTimeFormatter.format(task.completedAt) },
        { label: 'Resultaat:', value: task.result },
      ],
    },
  ]
}

const entries = computed(() => buildTaskLog(props.task))
</script>

<style scoped>
.log {
  box-sizing: border-box;
  flex-shrink: 0;
  width: var(--width-task-log);
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.5rem;
  border-left: 1px solid var(--color-border);
  overflow: auto;
}

h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
}

hr {
  margin: 0;
  border: none;
  border-top: 1px solid var(--color-border);
}

.entry {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

h4 {
  margin: 0;
  font-size: 0.875rem;
  font-weight: 700;
  line-height: 1.4286;
}

.row {
  display: flex;
  gap: 0.25rem;
  font-size: 1rem;
  line-height: 1.375;
}

.row-label {
  flex-shrink: 0;
}

.row-value {
  font-weight: 700;
}
</style>
