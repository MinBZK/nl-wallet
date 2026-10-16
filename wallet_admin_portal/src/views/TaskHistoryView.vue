<template>
  <LoadingState v-if="loading" />
  <ErrorState v-else-if="error" @retry="load" />
  <TasksTable
    v-else
    variant="history"
    :tasks="tasks ?? []"
    empty-title="Geen taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />
</template>

<script setup lang="ts">
import { fetchTaskHistory } from '@/api/tasks.ts'
import { useAsyncData } from '@/composables/asyncData.ts'
import TasksTable from '@/components/tasks/TasksTable.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'

const { data: tasks, loading, error, load } = useAsyncData(fetchTaskHistory)
</script>
