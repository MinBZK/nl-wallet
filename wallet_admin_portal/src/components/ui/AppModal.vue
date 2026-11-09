<template>
  <Teleport to="body">
    <!-- Native modal: the browser provides the backdrop, focus trap, inert background and Escape handling. -->
    <dialog
      ref="dialog"
      class="dialog"
      :class="size"
      :aria-labelledby="labelId"
      @cancel.prevent="dismiss"
      @close="onNativeClose"
      @click.self="dismiss"
    >
      <!-- Exposes labelId to the slot for the caller's heading. -->
      <slot :label-id="labelId" />
    </dialog>
  </Teleport>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, useId, useTemplateRef } from 'vue'

const props = withDefaults(
  defineProps<{
    // False while an action is in flight, to block a fake cancel.
    dismissible?: boolean
    size?: 'sm' | 'md'
  }>(),
  {
    dismissible: true,
    size: 'md',
  },
)

const emit = defineEmits<{
  close: []
}>()

const labelId = useId()
const dialog = useTemplateRef<HTMLDialogElement>('dialog')
// Captured during setup, before any child mounts and can move focus.
const previouslyFocused =
  document.activeElement instanceof HTMLElement ? document.activeElement : null

// Escape (cancel event) and backdrop clicks both land here; the parent decides whether to unmount.
function dismiss() {
  if (props.dismissible) {
    emit('close')
  }
}

// Set before our own close() on unmount, so the resulting close event is ignored.
let unmounting = false

// Chrome lets a repeated Escape close the dialog even when cancel was prevented.
// Keep the native state in sync with the parent: dismiss if allowed, otherwise reopen.
function onNativeClose() {
  if (unmounting) return
  if (props.dismissible) {
    emit('close')
  } else {
    dialog.value?.showModal()
  }
}

onMounted(() => {
  dialog.value?.showModal()
  document.body.style.overflow = 'hidden'
})

onBeforeUnmount(() => {
  unmounting = true
  dialog.value?.close()
  document.body.style.overflow = ''
  previouslyFocused?.focus()
})
</script>

<style scoped>
.dialog {
  /* Reset the browser <dialog> defaults. */
  padding: 0;
  border: 0;
  color: inherit;

  width: 28.8125rem;
  /* Keep a 1.5rem gutter around the dialog, replacing the UA max-size. */
  max-width: calc(100% - 3rem);
  max-height: calc(100% - 3rem);
  overflow-y: auto;
  background: var(--color-background);
  border-radius: 6px;
  box-shadow: 0 1px 15px rgba(0, 0, 0, 0.05);
}

.dialog::backdrop {
  background: rgba(21, 42, 98, 0.2);
}

.dialog.sm {
  width: 19.5rem;
  border-radius: 1rem;
}
</style>
