<template>
  <div class="footer">
    <AppButton variant="secondary" @click="emit('close')">Sluiten</AppButton>
    <div class="actions">
      <AppButton v-if="rejectLabel" variant="danger" @click="emit('reject')">{{
        rejectLabel
      }}</AppButton>
      <AppButton v-if="approveLabel" variant="primary" @click="emit('approve')">{{
        approveLabel
      }}</AppButton>
      <AppButton v-if="actionLabel" variant="danger" @click="emit('action')">{{
        actionLabel
      }}</AppButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'

defineProps<{
  actionLabel?: string
  approveLabel?: string
  rejectLabel?: string
}>()

const emit = defineEmits<{
  close: []
  action: []
  approve: []
  reject: []
}>()
</script>

<style scoped>
.footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  box-sizing: border-box;
  width: 100%;
  padding: 1rem 1.5rem;
  border-top: 2px solid var(--color-border);
}

.actions {
  display: flex;
  align-items: center;
  gap: 16px;
}
</style>
