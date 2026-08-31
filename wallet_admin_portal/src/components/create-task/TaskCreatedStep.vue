<template>
  <TaskWizardCard>
    <div class="intro">
      <h2>Taak aangemaakt</h2>
      <p class="message">
        Je taak is aangemaakt. Een andere collega met de juiste rechten moet deze taak beoordelen.
      </p>
    </div>
    <TaskDetails :details="details" />
  </TaskWizardCard>
</template>

<script setup lang="ts">
import TaskWizardCard from './TaskWizardCard.vue'
import TaskDetails from './TaskDetails.vue'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'

defineProps<{
  details: TaskDetailRow[]
}>()
</script>

<style scoped>
.intro {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

h2 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1.4;
}

.message {
  margin: 0;
  font-size: 1rem;
  line-height: 1.375;
}
</style>
