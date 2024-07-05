<script setup lang="ts">
import { SessionType } from "@/models/status"

defineProps<{
  ul: string
}>()

const emit = defineEmits<{
  choice: [sessionType: SessionType]
}>()
</script>

<template>
  <h2>Op welk apparaat staat je NL Wallet app?</h2>
  <section class="buttons" data-testid="device_choice">
    <a
      role="button"
      :href="ul"
      target="_blank"
      class="button primary"
      data-testid="same_device_button"
      @click="emit('choice', SessionType.SameDevice)"
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
      @click="emit('choice', SessionType.CrossDevice)"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M4 11h12.17l-5.59-5.59L12 4l8 8-8 8-1.41-1.41L16.17 13H4z" />
      </svg>
      <span>Op een ander apparaat</span>
    </button>
  </section>
</template>
