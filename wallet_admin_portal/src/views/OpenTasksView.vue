<template>
  <TopHeader
    title="Openstaande taken"
    description="Je ziet alleen taken die passen bij jouw rol en rechten. Dit zijn de taken waarvoor jij nu iets moet doen."
  />
  <TasksFilters />
  <TasksTable />
  <PaginationBar />
</template>

<script setup lang="ts">
import TopHeader from '../components/TopHeader.vue'
import TasksFilters from '../components/TasksFilters.vue'
import TasksTable from '../components/TasksTable.vue'
import PaginationBar from '../components/PaginationBar.vue'
</script>
