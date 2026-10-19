<template>
  <ConfirmTaskModal
    :task-id="taskId"
    :busy="busy"
    :error="error"
    title-suffix="intrekken"
    question="Weet je zeker dat je deze taak wilt intrekken?"
    warning="Je kunt dit niet ongedaan maken."
    confirm-label="Ja, intrekken"
    danger-confirm
    with-reason
    reason-placeholder="Schrijf hier waarom je deze taak intrekt."
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
