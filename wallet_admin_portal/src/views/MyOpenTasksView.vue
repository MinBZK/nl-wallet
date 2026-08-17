<template>
  <TopHeader title="Mijn open taken" description="Dit zijn de taken die aan jou zijn toegewezen." />
  <TasksTable
    :columns="columns"
    :tasks="tasks"
    empty-title="Geen open taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />
</template>

<script setup lang="ts">
import TopHeader from '../components/TopHeader.vue'
import TasksTable, { type Task } from '../components/TasksTable.vue'

const columns = [
  { label: 'TAAK-ID' },
  { label: 'ACTIE' },
  { label: 'DOEL' },
  { label: 'AANGEMAAKT OP' },
  { label: 'AANGEMAAKT DOOR' },
  { label: 'VOLGENDE STAP', width: '8.125rem' },
]

const tasks: Task[] = []
</script>
