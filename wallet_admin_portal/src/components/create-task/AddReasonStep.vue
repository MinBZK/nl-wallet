<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    title="Voeg een reden toe"
    description="Leg uit waarom deze taak nodig is."
  />
  <TaskCard class="reason-card">
    <ReasonInput
      v-model="reason"
      placeholder="Leg uit waarom deze taak nodig is."
      required
      autofocus
    />
  </TaskCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import ReasonInput from '@/components/ui/ReasonInput.vue'

const reason = defineModel<string>({ required: true })

defineProps<{
  stepNumber: number
  totalSteps: number
}>()
</script>

<style scoped>
.reason-card {
  gap: 0.375rem;
}
</style>
