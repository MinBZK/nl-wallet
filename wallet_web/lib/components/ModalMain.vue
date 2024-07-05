<script setup lang="ts">
import CreatedSection from "@/components/CreatedSection.vue"
import ErrorSection from "@/components/ErrorSection.vue"
import HelpSection from "@/components/HelpSection.vue"
import InProgressSection from "@/components/InProgressSection.vue"
import LoadingSection from "@/components/LoadingSection.vue"
import SuccessSection from "@/components/SuccessSection.vue"
import { type ModalState, SessionState } from "@/models/state"
import { SessionType } from "@/models/status"
import { onMounted, onUnmounted, ref } from "vue"

defineProps<{
  modalState: ModalState
}>()

const emit = defineEmits<{
  choice: [sessionType: SessionType]
}>()

const main = ref<HTMLDivElement | null>(null)
const handleChoice = (sessionType: SessionType) => emit("choice", sessionType)

onMounted(async () => setTimeout(() => main.value && main.value.focus(), 0))
</script>

<template>
  <main ref="main" tabindex="0">
    <loading-section v-if="modalState.kind === SessionState.Loading"></loading-section>
    <created-section
      v-if="modalState.kind === SessionState.Created"
      :ul="modalState.ul"
      :sessionType="modalState.sessionType"
      @choice="handleChoice"
    ></created-section>
    <in-progress-section v-if="modalState.kind === SessionState.InProgress"></in-progress-section>
    <success-section
      v-if="modalState.kind === SessionState.Success"
      :sessionType="modalState.sessionType"
    ></success-section>
    <error-section
      v-if="modalState.kind === SessionState.Error"
      :errorType="modalState.errorType"
    ></error-section>
  </main>

  <help-section v-if="modalState.kind === SessionState.Created"></help-section>
</template>
