<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    title="Controleer en maak taak aan"
    description="Controleer de taak voordat je die aanmaakt."
  />
  <TaskWizardCard>
    <h3>Taakdetails</h3>
    <TaskDetails :details="details" />
  </TaskWizardCard>
  <TaskWizardCard>
    <h3>{{ consequences.length > 1 ? 'Belangrijkste gevolgen' : 'Gevolg' }}</h3>
    <TaskItemList :value="consequences.length > 1 ? consequences : (consequences[0] ?? '')" />
  </TaskWizardCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskWizardCard from './TaskWizardCard.vue'
import TaskDetails from './TaskDetails.vue'
import TaskItemList from './TaskItemList.vue'
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
