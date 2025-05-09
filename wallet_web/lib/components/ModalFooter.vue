<script setup lang="ts">
import { type ModalState } from "@/models/state"
import { injectStrict, translationsKey } from "@/util/translations"

defineProps<{
  modalState: ModalState
}>()

const t = injectStrict(translationsKey)

const emit = defineEmits<{
  close: []
  stop: []
  confirm: []
  retry: []
  back: []
}>()
</script>

<template>
  <footer>
    <a
      v-if="['creating', 'loading', 'in-progress'].includes(modalState.kind)"
      href="/help"
      class="button link"
      data-testid="help"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M6.4 18.5 5 17.1l9.6-9.6H6v-2h12v12h-2V8.9z" />
      </svg>
      <span>{{ t("need_help") }}</span>
    </a>

    <button
      v-if="['creating', 'loading', 'in-progress', 'confirm-stop'].includes(modalState.kind)"
      type="button"
      class="button"
      :class="{
        secondary: ['creating', 'loading', 'in-progress'].includes(modalState.kind),
        error: modalState.kind === 'confirm-stop',
      }"
      data-testid="cancel_button"
      @click="modalState.kind === 'confirm-stop' ? emit('stop') : emit('confirm')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2m0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31A7.9 7.9 0 0 1 12 20m6.31-3.1L7.1 5.69A7.9 7.9 0 0 1 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9"
        />
      </svg>
      <span>{{ modalState.kind === "confirm-stop" ? t("yes_stop") : t("stop") }}</span>
    </button>

    <button
      v-if="modalState.kind === 'error'"
      type="button"
      class="button primary"
      data-testid="retry_button"
      @click="emit('retry')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M12 22.5q-1.874 0-3.512-.712a9.1 9.1 0 0 1-2.85-1.926 9.1 9.1 0 0 1-1.926-2.85A8.7 8.7 0 0 1 3 13.5h2q0 2.925 2.038 4.962T12 20.5q2.925 0 4.962-2.038T19 13.5q0-2.925-2.038-4.963Q14.925 6.5 12 6.5h-.15l1.55 1.55L12 9.5l-4-4 4-4 1.4 1.45-1.55 1.55H12q1.875 0 3.513.713a9.2 9.2 0 0 1 2.85 1.924 9.2 9.2 0 0 1 1.925 2.85A8.7 8.7 0 0 1 21 13.5q0 1.874-.712 3.512a9.2 9.2 0 0 1-1.925 2.85 9.2 9.2 0 0 1-2.85 1.926A8.7 8.7 0 0 1 12 22.5"
        />
      </svg>
      <span>{{ t("retry") }}</span>
    </button>

    <button
      v-if="['created', 'error', 'success'].includes(modalState.kind)"
      type="button"
      class="button"
      :class="{
        link:
          ['created', 'error'].includes(modalState.kind) ||
          (modalState.kind === 'success' && modalState.session.sessionType === 'cross_device'),
        primary: modalState.kind === 'success' && modalState.session.sessionType === 'same_device',
      }"
      data-testid="close_button"
      @click="modalState.kind === 'created' ? emit('stop') : emit('close')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
        />
      </svg>
      <span>{{ t("close") }}</span>
    </button>

    <button
      v-if="modalState.kind === 'confirm-stop'"
      type="button"
      class="button link"
      data-testid="back_button"
      @click="emit('back')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20z" />
      </svg>
      <span>{{ t("no") }}</span>
    </button>
  </footer>
</template>
