<template>
  <TasksFilters />
  <LoadingState v-if="loading" />
  <ErrorState v-else-if="error" @retry="load" />
  <TasksTable
    v-else
    variant="open"
    :tasks="tasks ?? []"
    empty-title="Geen openstaande taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />

  <Teleport defer to="#page-footer-target">
    <PaginationBar />
  </Teleport>
</template>

<script setup lang="ts">
import { fetchOpenTasks } from '@/api/tasks.ts'
import { useAsyncData } from '@/composables/asyncData.ts'
import TasksTable from '@/components/tasks/TasksTable.vue'
import TasksFilters from '@/components/tasks/TasksFilters.vue'
import LoadingState from '@/components/LoadingState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import PaginationBar from '@/components/ui/PaginationBar.vue'

const { data: tasks, loading, error, load } = useAsyncData(fetchOpenTasks)
</script>
