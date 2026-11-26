<template>
  <div class="item-row">
    <div class="item-info">
      <p class="item-title">{{ title }}</p>
      <p v-if="warning" class="item-line warning">
        <img src="@/assets/icons/warning.svg" alt="" class="warning-icon" />
        {{ warning }}
      </p>
      <p v-for="line in lines" :key="line" class="item-line">{{ line }}</p>
    </div>
    <button
      v-if="actionLabel"
      type="button"
      class="action"
      :aria-label="`${actionLabel} ${title}`"
      @click="emit('action')"
    >
      {{ actionLabel }}
    </button>
  </div>
</template>

<script setup lang="ts">
// A row in the create-task wizard describing one item (e.g. a wallet), with an optional single action.
defineProps<{
  title: string
  /** Lines describing the item, shown under the title. */
  lines: string[]
  /** Shown above [lines], marked as a warning. */
  warning?: string
  /** Label of the action button; the button is hidden when omitted. */
  actionLabel?: string
}>()

const emit = defineEmits<{
  action: []
}>()
</script>

<style scoped>
.item-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: #fff;
}

.item-info {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 0.125rem;
  /* Allow the column to shrink below its content width so long ids wrap. */
  min-width: 0;
}

p {
  margin: 0;
}

.item-title {
  font-weight: 700;
  line-height: 1.375;
  overflow-wrap: anywhere;
}

.item-line {
  font-size: 0.875rem;
  line-height: 1.4286;
}

.warning {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  color: var(--color-warning);
}

.warning-icon {
  width: 1rem;
  height: 1rem;
}

.action {
  /* Size to the label and never shrink, so the button always fits. */
  flex: none;
  white-space: nowrap;
  padding: 0;
  border: none;
  background: none;
  color: var(--color-primary);
  font: inherit;
  font-size: 0.875rem;
  font-weight: 700;
  letter-spacing: 0.03125rem;
  text-transform: uppercase;
  cursor: pointer;
}

.action:hover {
  color: var(--color-primary-hover);
  text-decoration: underline;
}
</style>
