<script setup lang="ts">
import { FooterState } from "@/models/footer-state"
import { isMobileKey } from "@/util/projection_keys"
import { mdiArrowTopRightThin, mdiCancel, mdiRefresh, mdiWindowClose } from "@mdi/js"
import { inject } from "vue"

defineProps<{
  state: FooterState
}>()

const isMobile = inject(isMobileKey)

const emit = defineEmits(["close", "stop", "retry"])

function close() {
  emit("close")
}

function stop() {
  emit("stop")
}

function retry() {
  emit("retry")
}
</script>

<template>
  <section v-if="state == FooterState.Close" class="website-link" data-testid="website_link">
    <p v-if="isMobile" data-testid="mobile_text">Nog geen NL Wallet app? Of hulp nodig?</p>
    <p>
      <a href="/" class="link">
        <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowTopRightThin"></path>
        </svg>
        <span>Naar NL Wallet website</span>
      </a>
    </p>
  </section>

  <footer>
    <section v-if="state == FooterState.Stop" class="help" data-testid="help">
      <a href="/" class="link">
        <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowTopRightThin"></path>
        </svg>
        <span>Hulp nodig?</span>
      </a>
    </section>

    <button
      v-if="state == FooterState.Stop"
      type="button"
      class="secondary full-width"
      data-testid="cancel_button"
      @click="stop"
    >
      <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
        <path :d="mdiCancel"></path>
      </svg>
      <span>Stoppen</span>
    </button>

    <button
      v-if="state == FooterState.Retry"
      type="button"
      class="primary full-width"
      data-testid="retry_button"
      @click="retry"
    >
      <svg fill="currentColor" width="24" height="24" viewBox="0 0 24 24">
        <path :d="mdiRefresh"></path>
      </svg>
      <span>Probeer opnieuw</span>
    </button>

    <button
      v-if="state === FooterState.Close || state === FooterState.Ok || state === FooterState.Retry"
      type="button"
      class="full-width"
      :class="{
        link: state === FooterState.Close || state === FooterState.Retry,
        primary: state === FooterState.Ok,
      }"
      data-testid="close_button"
      @click="close"
    >
      <svg fill="currentColor" width="24" height="24" viewBox="0 0 24 24">
        <path :d="mdiWindowClose"></path>
      </svg>
      <span>Sluiten</span>
    </button>
  </footer>
</template>
