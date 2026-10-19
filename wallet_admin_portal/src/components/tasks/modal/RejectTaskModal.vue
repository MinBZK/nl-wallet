<template>
  <ConfirmTaskModal
    :task-id="taskId"
    :busy="busy"
    :error="error"
    title-suffix="afwijzen"
    question="Weet je zeker dat je deze taak wilt afwijzen?"
    warning="Je kunt deze actie niet ongedaan maken."
    confirm-label="Ja, afwijzen"
    with-reason
    reason-placeholder="Schrijf hier waarom je deze taak afwijst."
    @close="emit('close')"
    @confirm="emit('confirm', $event)"
  />
</template>

<script setup lang="ts">
import ConfirmTaskModal from './ConfirmTaskModal.vue'

defineProps<{
  taskId: string
  busy?: boolean
  error?: boolean
}>()

const emit = defineEmits<{
  close: []
  confirm: [reason: string]
}>()
</script>
