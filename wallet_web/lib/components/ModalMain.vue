<script setup lang="ts">
import ConfirmStopSection from "@/components/ConfirmStopSection.vue"
import CreatedSection from "@/components/CreatedSection.vue"
import ErrorSection from "@/components/ErrorSection.vue"
import HelpSection from "@/components/HelpSection.vue"
import InProgressSection from "@/components/InProgressSection.vue"
import LoadingSection from "@/components/LoadingSection.vue"
import SuccessSection from "@/components/SuccessSection.vue"
import { type ModalState } from "@/models/state"
import { type SessionType } from "@/models/status"
import { onMounted, ref } from "vue"

defineProps<{
  modalState: ModalState
  helpBaseUrl: URL
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
    <loading-section v-if="['creating', 'loading'].includes(modalState.kind)"></loading-section>
    <created-section
      v-if="modalState.kind === 'created'"
      :same-device-ul="modalState.crossDeviceUl"
      :cross-device-ul="modalState.sameDeviceUl"
      :sessionType="modalState.session.sessionType"
      @choice="handleChoice"
    ></created-section>
    <in-progress-section v-if="modalState.kind === 'in-progress'"></in-progress-section>
    <confirm-stop-section v-if="modalState.kind === 'confirm-stop'" :helpBaseUrl></confirm-stop-section>
    <success-section
      v-if="modalState.kind === 'success'"
      :sessionType="modalState.session.sessionType"
    ></success-section>
    <error-section v-if="modalState.kind === 'error'" :errorType="modalState.errorType" :helpBaseUrl></error-section>
  </main>

  <help-section v-if="modalState.kind === 'created'" :helpBaseUrl></help-section>
</template>
