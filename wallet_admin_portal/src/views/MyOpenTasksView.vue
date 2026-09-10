<template>
  <LoadingState v-if="loading" />
  <TasksTable
    v-else
    :columns="columns"
    :tasks="tasks"
    empty-title="Geen open taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { fetchMyOpenTasks } from '@/api/tasks.ts'
import type { Task } from '@/types/task.ts'
import TasksTable from '@/components/tasks/TasksTable.vue'
import LoadingState from '@/components/LoadingState.vue'

const columns = [
  { label: 'TAAK-ID' },
  { label: 'ACTIE' },
  { label: 'DOEL' },
  { label: 'AANGEMAAKT OP' },
  { label: 'AANGEMAAKT DOOR' },
  { label: 'VOLGENDE STAP', width: '8.125rem' },
]

const tasks = ref<Task[]>([])
const loading = ref(true)

// TODO: handle a rejected fetch once the ErrorState component lands together with [PVW-6183].
onMounted(async () => {
  try {
    tasks.value = await fetchMyOpenTasks()
  } finally {
    loading.value = false
  }
})
</script>
