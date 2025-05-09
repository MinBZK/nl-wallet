<script setup lang="ts">
import { type SessionType } from "@/models/status"
import { injectStrict, translationsKey } from "@/util/translations"

defineProps<{
  ul: URL
}>()

const t = injectStrict(translationsKey)

const emit = defineEmits<{
  choice: [sessionType: SessionType]
}>()
</script>

<template>
  <h2>{{ t("device_choice_title") }}</h2>
  <section class="buttons" data-testid="device_choice">
    <a
      role="button"
      :href="ul.toString()"
      target="_blank"
      class="button primary"
      data-testid="same_device_button"
      @click="emit('choice', 'same_device')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M4 11h12.17l-5.59-5.59L12 4l8 8-8 8-1.41-1.41L16.17 13H4z" />
      </svg>
      <span>{{ t("device_choice_same_device") }}</span>
    </a>
    <button
      type="button"
      class="button secondary"
      data-testid="cross_device_button"
      @click="emit('choice', 'cross_device')"
    >
      <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
        <path d="M4 11h12.17l-5.59-5.59L12 4l8 8-8 8-1.41-1.41L16.17 13H4z" />
      </svg>
      <span>{{ t("device_choice_cross_device") }}</span>
    </button>
  </section>
</template>
