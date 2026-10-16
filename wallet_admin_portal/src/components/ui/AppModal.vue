<template>
  <Teleport to="body">
    <div class="overlay" @click.self="dismiss">
      <div
        ref="dialog"
        class="dialog"
        role="dialog"
        aria-modal="true"
        :aria-labelledby="labelId"
        tabindex="-1"
      >
        <!-- Exposes labelId to the slot for the caller's heading. -->
        <slot :label-id="labelId" />
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, useId, useTemplateRef } from 'vue'

const FOCUSABLE_SELECTOR =
  'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"])'

const props = withDefaults(
  defineProps<{
    // False while an action is in flight, to block a fake cancel.
    dismissible?: boolean
  }>(),
  {
    dismissible: true,
  },
)

const emit = defineEmits<{
  close: []
}>()

const labelId = useId()
const dialog = useTemplateRef<HTMLElement>('dialog')
// Captured during setup, before any child mounts and can move focus.
const previouslyFocused =
  document.activeElement instanceof HTMLElement ? document.activeElement : null

function dismiss() {
  if (props.dismissible) {
    emit('close')
  }
}

function focusableElements(): HTMLElement[] {
  return Array.from(dialog.value?.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR) ?? [])
}

function regainFocus() {
  ;(focusableElements()[0] ?? dialog.value)?.focus()
}

// Pulls focus back in if it ends up outside, e.g. a focused button being disabled blurs to <body>.
function reclaimFocus() {
  if (dialog.value && !dialog.value.contains(document.activeElement)) {
    regainFocus()
  }
}

function trapTab(event: KeyboardEvent) {
  const elements = focusableElements()
  const first = elements[0]
  const last = elements[elements.length - 1]
  if (!first || !last) {
    // All controls disabled: hold focus in place.
    event.preventDefault()
    return
  }

  const active = document.activeElement
  if (!event.shiftKey && active === last) {
    event.preventDefault()
    first.focus()
  } else if (event.shiftKey && (active === first || active === dialog.value)) {
    // dialog.value: the tabindex="-1" container, reachable by clicking non-interactive content.
    event.preventDefault()
    last.focus()
  }
}

function handleKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    dismiss()
  } else if (event.key === 'Tab') {
    trapTab(event)
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
  document.addEventListener('focusin', reclaimFocus)
  document.body.style.overflow = 'hidden'
  regainFocus()
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', handleKeydown)
  document.removeEventListener('focusin', reclaimFocus)
  document.body.style.overflow = ''
  previouslyFocused?.focus()
})
</script>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(21, 42, 98, 0.2);
  padding: 1.5rem;
  z-index: 1000;
}

.dialog {
  width: 28.8125rem;
  max-width: 100%;
  max-height: 100%;
  overflow-y: auto;
  background: var(--color-background);
  border-radius: 6px;
  box-shadow: 0 1px 15px rgba(0, 0, 0, 0.05);
}
</style>
