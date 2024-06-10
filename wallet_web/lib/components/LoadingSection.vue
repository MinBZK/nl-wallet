<script setup lang="ts">
import LoadingIndicator from "@/components/LoadingIndicator.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"

const emit = defineEmits(["stop"])
</script>

<template>
  <article>
    <section class="loading" data-testid="loading">
      <h2>Even geduld</h2>
      <p>De gegevens worden opgehaald</p>
      <loading-indicator size="large"></loading-indicator>
    </section>
  </article>

  <modal-footer :state="FooterState.Cancel" @stop="emit('stop')"></modal-footer>
</template>
