<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { SessionType } from "@/models/status"

defineProps<{
  sessionType: SessionType
}>()

const emit = defineEmits(["close"])
</script>

<template>
  <main v-if="sessionType === SessionType.SameDevice" class="success">
    <svg width="24" height="24" fill="currentColor">
      <path
        d="m14 8.5-1-1-6 6-2-2-1 1 2 2 1 1 1-1zM18 1H8c-1.1 0-2 .9-2 2v3h2V4h10v16H8v-2H6v3c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V3c0-1.1-.9-2-2-2"
      />
    </svg>
    <section class="text">
      <h2>Gelukt!</h2>
      <p>Sluit deze pagina en ga verder in het nieuw geopende tabblad.</p>
    </section>
  </main>

  <main v-else class="success cross-device">
    <svg width="72" height="72" fill="currentColor">
      <path
        d="M36 0C16.118 0 0 16.118 0 36s16.118 36 36 36 36-16.118 36-36S55.882 0 36 0m7.598 29.6 1.4 1.4-12 12-5.6-5.6 1.4-1.4 4.2 4.2z"
      />
    </svg>
    <section class="text">
      <h2>Gelukt!</h2>
    </section>
  </main>

  <modal-footer
    :state="sessionType === SessionType.SameDevice ? FooterState.Ok : FooterState.Close"
    @close="emit('close')"
  ></modal-footer>
</template>
