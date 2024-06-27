<script setup lang="ts">
import LoadingIndicator from "@/components/LoadingIndicator.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"

const emit = defineEmits(["stop"])
</script>

<template>
  <main class="loading" data-testid="loading">
    <section class="text">
      <h2>Even geduld</h2>
      <p>De gegevens worden opgehaald</p>
    </section>
    <loading-indicator></loading-indicator>
  </main>

  <modal-footer :state="FooterState.Cancel" @stop="emit('stop')"></modal-footer>
</template>
