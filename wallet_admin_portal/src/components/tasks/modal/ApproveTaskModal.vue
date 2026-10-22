<template>
  <!-- Approving is irreversible (unlike rejecting), so it gets the danger styling here even
  though TaskDetailsFooter treats reject as the "danger" action. -->
  <ConfirmTaskModal
    :task-id="taskId"
    :busy="busy"
    :error="error"
    title-suffix="goedkeuren"
    question="Weet je zeker dat je deze taak wilt uitvoeren?"
    warning="Je kunt deze actie niet ongedaan maken."
    confirm-label="Ja, goedkeuren"
    danger-confirm
    @close="emit('close')"
    @confirm="emit('confirm')"
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
  confirm: []
}>()
</script>
