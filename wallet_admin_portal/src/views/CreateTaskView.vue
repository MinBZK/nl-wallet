<template>
  <template v-if="action">
    <div class="wizard-content">
      <SelectWalletsStep
        v-if="step === 'intro' && type === TaskActionType.RevokeWallet"
        v-model="selectedWallets"
        :step-number="stepNumber"
        :total-steps="totalSteps"
        :title="action.header"
        :description="action.description"
        :error="introError"
        @search="introError = null"
      />

      <template v-else-if="step === 'intro'">
        <TaskStepHeader
          :step-number="stepNumber"
          :total-steps="totalSteps"
          :title="action.header"
          :description="action.description"
        />
        <TaskCard>
          <p class="placeholder-message">Deze stap is nog in ontwikkeling.</p>
        </TaskCard>
      </template>

      <AddReasonStep
        v-else-if="step === 'reason'"
        v-model="reason"
        :step-number="stepNumber"
        :total-steps="totalSteps"
      />

      <ErrorState v-else-if="step === 'check' && taskCreationFailed" @retry="retryTaskCreation" />

      <CheckTaskStep
        v-else-if="step === 'check'"
        :step-number="stepNumber"
        :total-steps="totalSteps"
        :details="taskDetails"
        :consequences="action.consequences"
      />

      <TaskCreatedStep v-else :details="createdDetails" />
    </div>

    <Teleport v-if="!(step === 'check' && taskCreationFailed)" defer to="#page-footer-target">
      <TaskWizardFooter
        :show-cancel="step !== 'done'"
        :show-back="step === 'reason' || step === 'check'"
        :next-label="nextLabel"
        :next-disabled="(step === 'reason' && !reason.trim()) || isCreatingTask"
        @cancel="goBackOrHome"
        @back="goToPreviousStep"
        @next="handleNext"
      />
    </Teleport>
  </template>
  <div v-else class="wizard-content">
    <ErrorState @retry="goHome" />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch, watchEffect } from 'vue'
import { useRoute } from 'vue-router'
import { usePageTitle } from '@/composables/pageTitle.ts'
import { useNavigation } from '@/composables/navigation.ts'
import TaskStepHeader from '@/components/create-task/TaskStepHeader.vue'
import TaskWizardFooter from '@/components/create-task/TaskWizardFooter.vue'
import AddReasonStep from '@/components/create-task/AddReasonStep.vue'
import CheckTaskStep from '@/components/create-task/CheckTaskStep.vue'
import SelectWalletsStep from '@/components/create-task/SelectWalletsStep.vue'
import TaskCreatedStep from '@/components/create-task/TaskCreatedStep.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import TaskCard from '@/components/tasks/TaskCard.vue'
import { TASK_ACTION_INFO, TaskActionType } from '@/types/task-action.ts'
import type { TaskDetailRow } from '@/types/task-detail-row.ts'
import type { TaskPayload } from '@/types/task-payload.ts'
import type { Wallet } from '@/types/wallet.ts'
import { createTask } from '@/api/tasks.ts'
import { taskPayloadRows } from '@/utils/taskPayloadRows.ts'

type WizardStep = 'intro' | 'reason' | 'check' | 'done'

interface TaskDraft {
  /** What the task will do. */
  payload: TaskPayload
  /** The reason the first step can't continue yet, or null when it has everything it needs. */
  introStepError: string | null
}

/** Order of all wizard steps. */
const STEP_ORDER: WizardStep[] = ['intro', 'reason', 'check', 'done']
/** The steps shown as "Stap x van y"; `done` is a confirmation screen, not a numbered step. */
const NUMBERED_STEPS: WizardStep[] = ['intro', 'reason', 'check']

const route = useRoute()
const { setPageTitle, resetPageTitle } = usePageTitle()
const { goBackOrHome, goHome } = useNavigation()

const step = ref<WizardStep>('intro')
const reason = ref('')
const selectedWallets = ref<Wallet[]>([])
const introError = ref<string | null>(null)
const taskId = ref<string | null>(null)
const isCreatingTask = ref(false)
const taskCreationFailed = ref(false)

const stepNumber = computed(() => NUMBERED_STEPS.indexOf(step.value) + 1)
const totalSteps = NUMBERED_STEPS.length

const type = computed(() => route.params.type as TaskActionType)
const action = computed(() => TASK_ACTION_INFO[type.value] ?? null)
const pageTitle = computed(() => (action.value ? `Maak taak aan: ${action.value.title}` : ''))

/** The part of the wizard's data that depends on the task type. */
const taskDraft = computed((): TaskDraft => {
  if (type.value === TaskActionType.RevokeWallet) {
    const walletIds = selectedWallets.value.map((wallet) => wallet.id)
    return {
      payload: { taskType: TaskActionType.RevokeWallet, taskParameters: { walletIds } },
      introStepError:
        walletIds.length === 0
          ? 'Om verder te gaan, moet je minimaal één wallet toevoegen aan de lijst.'
          : null,
    }
  }
  return {
    payload: { taskType: type.value, taskParameters: {} },
    introStepError: null,
  }
})

const payloadRows = computed(() => taskPayloadRows(taskDraft.value.payload))

const taskDetails = computed<TaskDetailRow[]>(() => [
  { label: 'Actie', value: action.value?.title ?? '' },
  ...payloadRows.value.summaryRows,
  ...payloadRows.value.selectionRows,
  { label: 'Reden', value: reason.value },
])

const createdDetails = computed<TaskDetailRow[]>(() => [
  { label: 'Taak ID', value: taskId.value ?? '' },
  { label: 'Actie', value: action.value?.title ?? '' },
  ...payloadRows.value.summaryRows,
])

const nextLabel = computed(() => {
  if (step.value === 'check') return 'Maak taak aan'
  if (step.value === 'done') return 'Sluiten'
  return 'Volgende'
})

watchEffect(() => {
  if (!action.value) return
  setPageTitle(pageTitle.value)
})
watch(selectedWallets, () => {
  introError.value = null
})
onBeforeUnmount(resetPageTitle)

function goToPreviousStep() {
  taskCreationFailed.value = false
  step.value = STEP_ORDER[STEP_ORDER.indexOf(step.value) - 1]!
}

async function handleNext() {
  if (step.value === 'check') {
    if (isCreatingTask.value) return
    isCreatingTask.value = true
    try {
      const task = await createTask(taskDraft.value.payload, reason.value)
      taskId.value = task.id
      taskCreationFailed.value = false
      step.value = 'done'
    } catch {
      taskCreationFailed.value = true
    } finally {
      isCreatingTask.value = false
    }
    return
  }
  if (step.value === 'done') {
    goBackOrHome()
    return
  }
  if (step.value === 'intro') {
    introError.value = taskDraft.value.introStepError
    if (introError.value) return
  }
  step.value = STEP_ORDER[STEP_ORDER.indexOf(step.value) + 1]!
}

function retryTaskCreation() {
  taskCreationFailed.value = false
  return handleNext()
}
</script>

<style scoped>
.wizard-content {
  display: flex;
  flex-direction: column;
  align-items: safe center;
  gap: 1.5rem;
  box-sizing: border-box;
  height: 100%;
  padding: 1.5rem;
  overflow: auto;
}

.placeholder-message {
  margin: 0;
  color: var(--color-error);
  text-align: center;
}
</style>
