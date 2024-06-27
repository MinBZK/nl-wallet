<script setup lang="ts">
import HelpSection from "@/components/HelpSection.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { SessionType } from "@/models/status"

defineProps<{
  engagementUrl: string
}>()

const emit = defineEmits<{
  close: []
  choice: [session_type: SessionType]
}>()

function handleChoice(sessionType: SessionType) {
  emit("choice", sessionType)
}
</script>

<template>
  <main data-testid="device_choice">
    <h2>Op welk apparaat staat je NL Wallet app?</h2>
    <section class="buttons">
      <a
        :href="engagementUrl"
        target="_blank"
        class="button primary"
        data-testid="same_device_button"
        @click="() => handleChoice(SessionType.SameDevice)"
      >
        <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
          <path d="M4 11h12.17l-5.59-5.59L12 4l8 8-8 8-1.41-1.41L16.17 13H4z" />
        </svg>
        <span>Op dit apparaat</span>
      </a>
      <button
        type="button"
        class="button secondary"
        data-testid="cross_device_button"
        @click="() => handleChoice(SessionType.CrossDevice)"
      >
        <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
          <path d="M4 11h12.17l-5.59-5.59L12 4l8 8-8 8-1.41-1.41L16.17 13H4z" />
        </svg>
        <span>Op een ander apparaat</span>
      </button>
    </section>
  </main>

  <help-section></help-section>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
