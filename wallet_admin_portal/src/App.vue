<template>
  <div class="app-shell">
    <AppSidebar />
    <main class="main-content">
      <TopHeader />
      <TasksFilters />
      <TasksTable />
      <PaginationBar />
    </main>
  </div>
</template>

<script setup lang="ts">
import AppSidebar from './components/AppSidebar.vue'
import TopHeader from './components/TopHeader.vue'
import TasksFilters from './components/TasksFilters.vue'
import TasksTable from './components/TasksTable.vue'
import PaginationBar from './components/PaginationBar.vue'
</script>

<style scoped>
.app-shell {
  display: grid;
  grid-template-columns: 260px 1fr;
  height: 100vh;
  overflow: hidden;
  background: #fafafa;
  color: #1f2a44;
  font-family: 'RO Sans', sans-serif;
}

.main-content {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  background: #fff;
  border-left: 1px solid #e7e7ef;
}
</style>
