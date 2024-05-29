<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { SessionType } from "@/models/engagement"
import { FooterState } from "@/models/footer-state"
import { mdiArrowRight } from "@mdi/js"

const emit = defineEmits(["choice", "close"])
</script>

<template>
  <article>
    <h2>Op welk apparaat staat je NL Wallet app?</h2>
    <section class="device-choice" data-testid="device_choice">
      <button
        type="button"
        class="primary full-width"
        data-testid="same_device_button"
        @click="emit('choice', SessionType.SameDevice)"
      >
        <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowRight"></path>
        </svg>
        <span>Op dit apparaat</span>
      </button>
      <button
        type="button"
        class="secondary full-width"
        data-testid="cross_device_button"
        @click="emit('choice', SessionType.CrossDevice)"
      >
        <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowRight"></path>
        </svg>
        <span>Op een ander apparaat</span>
      </button>
    </section>
  </article>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
