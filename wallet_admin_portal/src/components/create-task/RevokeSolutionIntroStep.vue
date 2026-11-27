<template>
  <TaskStepHeader
    :step-number="stepNumber"
    :total-steps="totalSteps"
    :title="title"
    :description="description"
  />

  <TaskCard>
    <h3>Wanneer gebruik je dit?</h3>
    <TaskItemList :value="WHEN_TO_USE" />
  </TaskCard>

  <TaskCard>
    <h3>Belangrijkste gevolgen</h3>
    <TaskItemList :value="consequences" />
  </TaskCard>

  <TaskCard>
    <h3>Regels</h3>
    <TaskItemList :value="RULES" />
  </TaskCard>

  <TaskCard :class="{ invalid: error }">
    <AppCheckbox
      v-model="acknowledged"
      :error="error"
      label="Ik heb dit gelezen en begrijp dat deze actie alle wallets stopt en niet ongedaan gemaakt kan worden."
    />
  </TaskCard>
</template>

<script setup lang="ts">
import TaskStepHeader from './TaskStepHeader.vue'
import TaskItemList from './TaskItemList.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'

const WHEN_TO_USE = [
  'Gebruik dit alleen in een uitzonderlijke situatie.',
  'Gebruik dit alleen bij een ernstig probleem dat alle wallets raakt.',
  'Gebruik dit alleen als een kleinere maatregel niet genoeg is.',
]

const RULES = [
  'Je moet een duidelijke reden opgeven.',
  'Deze taak volgt het vier-ogenprincipe.',
  'De actie wordt vastgelegd in de auditlog.',
]

const acknowledged = defineModel<boolean>({ required: true })

defineProps<{
  stepNumber: number
  totalSteps: number
  title: string
  description: string
  consequences: string[]
  /** Shown when the wizard cannot continue because the acknowledgement is missing. */
  error: string | null
}>()
</script>

<style scoped>
h3 {
  margin: 0;
  font-size: 1.125rem;
  font-weight: 700;
  line-height: 1.3333;
}

.invalid {
  border-color: var(--color-error);
}
</style>
