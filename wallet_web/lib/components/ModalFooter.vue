<script setup lang="ts">
import { FooterState } from "@/models/footer-state"

defineProps<{
  state: FooterState
}>()

const emit = defineEmits(["close", "stop", "retry"])
</script>

<template>
  <footer>
    <a
      v-if="state === FooterState.Stop || state === FooterState.Cancel"
      href="/help"
      class="button link"
      data-testid="help"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M6.4 18.5 5 17.1l9.6-9.6H6v-2h12v12h-2V8.9z" />
      </svg>
      <span>Hulp nodig?</span>
    </a>

    <button
      v-if="state === FooterState.Stop || state === FooterState.Cancel"
      type="button"
      class="button secondary"
      data-testid="cancel_button"
      @click="emit('stop')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2m0 18c-4.42 0-8-3.58-8-8 0-1.85.63-3.55 1.69-4.9L16.9 18.31A7.9 7.9 0 0 1 12 20m6.31-3.1L7.1 5.69A7.9 7.9 0 0 1 12 4c4.42 0 8 3.58 8 8 0 1.85-.63 3.55-1.69 4.9"
        />
      </svg>
      <span>Stoppen</span>
    </button>

    <button
      v-if="state === FooterState.Retry"
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
      <span>Probeer opnieuw</span>
    </button>

    <button
      v-if="state === FooterState.Close || state === FooterState.Ok || state === FooterState.Retry"
      type="button"
      class="button"
      :class="{
        link: state === FooterState.Close || state === FooterState.Retry,
        primary: state === FooterState.Ok,
      }"
      data-testid="close_button"
      @click="emit('close')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path
          d="M19 6.41 17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
        />
      </svg>
      <span>Sluiten</span>
    </button>
  </footer>
</template>
