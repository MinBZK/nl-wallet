<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    title="Controleer en maak taak aan"
    description="Controleer de taak voordat je die aanmaakt."
  />
  <TaskCard>
    <h3>Taakdetails</h3>
    <TaskDetails :details="details" />
  </TaskCard>
  <TaskCard v-if="consequences.length > 0">
    <h3>{{ consequences.length > 1 ? 'Belangrijkste gevolgen' : 'Gevolg' }}</h3>
    <TaskItemList :value="consequences.length > 1 ? consequences : consequences[0]!" />
  </TaskCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskDetails from './TaskDetails.vue'
import TaskItemList from './TaskItemList.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'

defineProps<{
  stepNumber: number
  totalSteps: number
  details: TaskDetailRow[]
  consequences: string[]
}>()
</script>

<style scoped>
h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
}
</style>
