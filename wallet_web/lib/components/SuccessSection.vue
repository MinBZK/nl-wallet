<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { SessionType } from "@/models/engagement"
import { FooterState } from "@/models/footer-state"
import { mdiCellphoneCheck, mdiCheckCircle } from "@mdi/js"

defineProps<{
  sessionType: SessionType
}>()

const emit = defineEmits(["close"])
</script>

<template>
  <article>
    <section
      v-if="sessionType === SessionType.SameDevice"
      class="success status-update same-device"
    >
      <div>
        <svg fill="currentColor" width="24" height="24" viewBox="0 0 24 24">
          <path :d="mdiCellphoneCheck"></path>
        </svg>
      </div>
      <h2>Gelukt!</h2>
      <p>Sluit deze pagina en ga verder in het nieuw geopende tabblad.</p>
    </section>

    <section v-else class="success">
      <div>
        <svg fill="currentColor" width="64" height="64" viewBox="0 0 24 24">
          <path :d="mdiCheckCircle"></path>
        </svg>
      </div>
      <h2>Gelukt!</h2>
    </section>
  </article>

  <modal-footer :state="FooterState.Ok" @close="emit('close')"></modal-footer>
</template>
