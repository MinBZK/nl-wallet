<template>
  <TopHeader
    title="Openstaande taken"
    description="Je ziet alleen taken die passen bij jouw rol en rechten. Dit zijn de taken waarvoor jij nu iets moet doen."
  />
  <TasksFilters />
  <TasksTable
    :columns="columns"
    :tasks="tasks"
    empty-title="Geen openstaande taken gevonden"
    empty-description="Controleer je zoekcriteria. Pas je filters aan en probeer het opnieuw."
  />
  <PaginationBar />
</template>

<script setup lang="ts">
import TopHeader from '../components/TopHeader.vue'
import TasksFilters from '../components/TasksFilters.vue'
import TasksTable from '../components/TasksTable.vue'
import PaginationBar from '../components/PaginationBar.vue'

const columns = [
  { label: 'TAAK-ID' },
  { label: 'ACTIE' },
  { label: 'DOEL' },
  { label: 'AANGEMAAKT OP' },
  { label: 'AANGEMAAKT DOOR' },
  { label: 'VOLGENDE STAP', width: '8.125rem' },
]

const tasks = [
  {
    id: 'UD-3774219',
    action: 'Gebruiker deblokkeren',
    target: 'Gebruiker: RC-1234-9930',
    createdAt: '07-09-2026',
    createdBy: 'Lisa Vermeer',
  },
  {
    id: 'UD-3774210',
    action: 'Gebruiker deblokkeren',
    target: 'Gebruiker: RC-1234-9930',
    createdAt: '07-09-2026',
    createdBy: 'John de Wit',
  },
  {
    id: 'UD-3774201',
    action: 'Gebruiker deblokkeren',
    target: 'Gebruiker: RC-1234-9930',
    createdAt: '08-09-2026',
    createdBy: 'Sarah Jansen',
  },
]
</script>
