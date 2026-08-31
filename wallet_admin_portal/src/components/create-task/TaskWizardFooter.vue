<template>
  <div class="footer" :class="{ 'flex-end': !showCancel }">
    <AppButton v-if="showCancel" variant="secondary" @click="emit('cancel')">Annuleren</AppButton>
    <div class="right">
      <AppButton v-if="showBack" variant="secondary" @click="emit('back')">Vorige</AppButton>
      <AppButton data-testid="wizard-next-button" :disabled="nextDisabled" @click="emit('next')">
        {{ nextLabel }}
      </AppButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'

withDefaults(
  defineProps<{
    showCancel?: boolean
    showBack?: boolean
    nextLabel: string
    nextDisabled?: boolean
  }>(),
  {
    showCancel: false,
    showBack: false,
    nextDisabled: false,
  },
)

const emit = defineEmits<{
  cancel: []
  back: []
  next: []
}>()
</script>

<style scoped>
.footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  box-sizing: border-box;
  width: 100%;
  padding: 1rem 1.5rem;
  border-top: 2px solid var(--color-border);
}

.footer.flex-end {
  justify-content: flex-end;
}

.right {
  display: flex;
  align-items: center;
  gap: 1rem;
}
</style>
