<template>
  <section class="table-wrap">
    <table class="tasks-table">
      <thead>
        <tr>
          <th>TAAK-ID</th>
          <th>ACTIE</th>
          <th>DOEL</th>
          <th>AANGEMAAKT OP ↓</th>
          <th>AANGEMAAKT DOOR</th>
          <th>VOLGENDE STAP</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="task in tasks" :key="task.id">
          <td>{{ task.id }}</td>
          <td>{{ task.action }}</td>
          <td>{{ task.target }}</td>
          <td>{{ task.createdAt }}</td>
          <td>{{ task.createdBy }}</td>
          <td><a href="#" class="details">BEKIJK DETAILS</a></td>
        </tr>
      </tbody>
    </table>
  </section>
</template>

<script setup lang="ts">
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

<style scoped>
.table-wrap {
  padding: 0 24px;
  flex: 1;
  overflow-y: auto;
}

.tasks-table {
  width: 100%;
  border-collapse: collapse;
  background: #fff;
  box-shadow: 0 1px 0 #e9e9f1;
}

thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  text-align: left;
  padding: 14px 16px;
  font-size: 12px;
  letter-spacing: 0.02em;
  color: #4a5674;
  background: #f3f4fb;
  border-bottom: 1px solid #e5e7f0;
}

tbody td {
  padding: 22px 16px;
  border-bottom: 1px solid #ececf3;
  color: #39445f;
  font-size: 14px;
}

.details {
  color: #4b55d6;
  font-weight: 700;
  text-decoration: none;
}

.details:hover {
  text-decoration: underline;
}
</style>
