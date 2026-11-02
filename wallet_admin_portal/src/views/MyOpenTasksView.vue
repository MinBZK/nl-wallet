<template>
  <LoadingState v-if="loading" />
  <PageErrorState v-else-if="error" @retry="load" />
  <TasksTable
    v-else
    variant="open"
    :tasks="tasks ?? []"
    empty-title="Geen open taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />
</template>

<script setup lang="ts">
import { fetchMyOpenTasks } from '@/api/tasks.ts'
import { useAsyncData } from '@/composables/asyncData.ts'
import TasksTable from '@/components/tasks/TasksTable.vue'
import LoadingState from '@/components/LoadingState.vue'
import PageErrorState from '@/components/ui/PageErrorState.vue'

const { data: tasks, loading, error, load } = useAsyncData(fetchMyOpenTasks)
</script>
