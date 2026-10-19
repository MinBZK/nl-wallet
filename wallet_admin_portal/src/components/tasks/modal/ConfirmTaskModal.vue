<template>
  <AppModal v-slot="{ labelId }" :dismissible="!busy" @close="emit('close')">
    <div class="content">
      <h2 :id="labelId">Taak {{ taskId }} {{ titleSuffix }}</h2>
      <p class="question">{{ question }}</p>

      <ReasonInput
        v-if="withReason"
        v-model="reason"
        :placeholder="reasonPlaceholder"
        :disabled="busy"
        required
      />

      <p class="warning">{{ warning }}</p>

      <p v-if="error" class="error" role="alert">
        Er ging iets mis bij het {{ titleSuffix }} van de taak. Probeer het opnieuw.
      </p>
    </div>

    <div class="footer">
      <AppButton class="cancel" variant="secondary" :disabled="busy" @click="emit('close')"
        >Nee, annuleren</AppButton
      >
      <AppButton
        class="confirm"
        :class="{ danger: dangerConfirm }"
        variant="primary"
        :disabled="busy || (withReason && !reason.trim())"
        @click="emit('confirm', reason)"
      >
        {{ confirmLabel }}
      </AppButton>
    </div>
  </AppModal>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import AppModal from '@/components/ui/AppModal.vue'
import AppButton from '@/components/ui/AppButton.vue'
import ReasonInput from '@/components/ui/ReasonInput.vue'

withDefaults(
  defineProps<{
    taskId: string
    titleSuffix: string
    question: string
    warning: string
    confirmLabel: string
    dangerConfirm?: boolean
    withReason?: boolean
    reasonPlaceholder?: string
    // While true the confirmed action is in flight: block confirming twice and block dismissal,
    // which would otherwise suggest the action was cancelled while it still completes server side.
    busy?: boolean
    // Set when the confirmed action failed; the modal stays open so the entered reason is kept.
    error?: boolean
  }>(),
  {
    dangerConfirm: false,
    withReason: false,
    reasonPlaceholder: '',
    busy: false,
    error: false,
  },
)

const emit = defineEmits<{
  close: []
  confirm: [reason: string]
}>()

const reason = ref('')
</script>

<style scoped>
.content {
  display: flex;
  flex-direction: column;
  padding: 1.5rem;
  gap: 1rem;
}

h2 {
  color: var(--color-text-primary);
  font-size: 1.5rem;
  font-weight: 700;
  line-height: 1.4167;
  margin: 0;
}

.question,
.warning {
  color: var(--color-text-primary);
  font-size: 1rem;
  line-height: 1.375;
  margin: 0;
}

.error {
  color: var(--color-error);
  font-size: 1rem;
  font-weight: 700;
  line-height: 1.375;
  margin: 0;
}

.footer {
  display: flex;
  justify-content: flex-end;
  gap: 1rem;
  padding: 1rem;
}

.footer :deep(.cancel) {
  border: 0px solid transparent;
  background: transparent;
}

.footer :deep(.confirm.danger) {
  background: var(--color-error);
  color: #fff;
}

.footer :deep(.confirm.danger:hover) {
  background: #910056;
  color: #fff;
}

.footer :deep(.confirm.danger:disabled:hover) {
  background: var(--color-error);
  color: #fff;
}
</style>
