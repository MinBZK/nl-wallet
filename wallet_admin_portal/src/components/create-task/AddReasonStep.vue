<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    title="Voeg een reden toe"
    description="Leg uit waarom deze taak nodig is."
  />
  <TaskCard class="reason-card">
    <ReasonInput
      :model-value="modelValue"
      placeholder="Leg uit waarom deze taak nodig is."
      required
      autofocus
      @update:model-value="emit('update:modelValue', $event)"
    />
  </TaskCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import ReasonInput from '@/components/ui/ReasonInput.vue'

defineProps<{
  stepNumber: number
  totalSteps: number
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<style scoped>
.reason-card {
  gap: 0.375rem;
}
</style>
