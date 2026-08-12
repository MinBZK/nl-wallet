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
  box-shadow: 0 1px 0 var(--color-border);
}

thead th {
  position: sticky;
  top: 0;
  z-index: 1;
  text-align: left;
  padding: 0.875rem 1rem;
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-text-secondary);
  background: var(--color-surface-tint);
  border-bottom: 1px solid var(--color-border);
}

tbody td {
  padding: 1.375rem 1rem;
  border-bottom: 1px solid var(--color-border);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
}

.details {
  color: var(--color-primary);
  font-weight: 700;
  text-decoration: none;
}

.details:hover {
  text-decoration: underline;
}
</style>
