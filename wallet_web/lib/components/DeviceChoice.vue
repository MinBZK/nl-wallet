<script setup lang="ts">
import LoadingIndicator from "@/components/LoadingIndicator.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { SessionType } from "@/models/status"
import { mdiArrowRight } from "@mdi/js"
import { onMounted, ref } from "vue"

defineProps<{
  engagementUrl: string
}>()

const emit = defineEmits<{
  close: []
  choice: [session_type: SessionType]
}>()

const chosen = ref(false)

function handleChoice(sessionType: SessionType) {
  chosen.value = true
  emit("choice", sessionType)
}

onMounted(() => {
  chosen.value = false
})
</script>

<template>
  <article>
    <h2>Op welk apparaat staat je NL Wallet app?</h2>
    <section class="device-choice" data-testid="device_choice">
      <a
        :href="engagementUrl"
        target="_blank"
        class="button primary full-width"
        :class="{ disabled: chosen }"
        data-testid="same_device_button"
        @click="() => handleChoice(SessionType.SameDevice)"
      >
        <loading-indicator
          v-if="chosen"
          size="small"
          data-testid="same_device_loading_indicator"
        ></loading-indicator>
        <svg v-else fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowRight"></path>
        </svg>
        <span>Op dit apparaat</span>
      </a>
      <button
        type="button"
        class="secondary full-width"
        :class="{ loading: chosen }"
        :disabled="chosen"
        data-testid="cross_device_button"
        @click="() => handleChoice(SessionType.CrossDevice)"
      >
        <loading-indicator
          v-if="chosen"
          size="small"
          data-testid="cross_device_loading_indicator"
        ></loading-indicator>
        <svg v-else fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowRight"></path>
        </svg>
        <span>Op een ander apparaat</span>
      </button>
    </section>
  </article>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
