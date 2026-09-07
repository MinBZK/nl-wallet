<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    title="Voeg een reden toe"
    description="Leg uit waarom deze taak nodig is."
  />
  <TaskWizardCard class="reason-card">
    <label for="reason" class="label">
      <span>Reden</span>
      <span class="required" aria-hidden="true">*</span>
    </label>
    <textarea
      id="reason"
      class="reason-input"
      :value="modelValue"
      maxlength="500"
      placeholder="Leg uit waarom deze taak nodig is."
      required
      aria-describedby="reason-counter"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
    <p id="reason-counter" class="counter" aria-live="polite">{{ modelValue.length }}/500</p>
  </TaskWizardCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskWizardCard from './TaskWizardCard.vue'

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

.label {
  display: flex;
  align-items: center;
  gap: 0.125rem;
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.4286;
}

.required {
  color: var(--color-error);
}

.reason-input {
  box-sizing: border-box;
  width: 100%;
  height: 6.25rem;
  padding: 1rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #fff;
  color: var(--color-text-primary);
  font-family: inherit;
  font-size: 1rem;
  line-height: 1.375;
  resize: vertical;
}

.counter {
  margin: 0.125rem 0 0 0;
  color: var(--color-text-primary);
  font-size: 0.75rem;
  line-height: 1.3333;
  text-align: right;
}
</style>
