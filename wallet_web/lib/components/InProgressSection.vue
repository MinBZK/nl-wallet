<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { mdiCellphoneArrowDown } from "@mdi/js"

const emit = defineEmits(["stop"])
</script>

<template>
  <article>
    <section class="in-progress" data-testid="in_progress">
      <div>
        <svg fill="currentColor" width="32" height="32" viewBox="0 0 24 24">
          <path :d="mdiCellphoneArrowDown"></path>
        </svg>
      </div>
      <h2>Volg de stappen in de NL Wallet app</h2>
    </section>
  </article>

  <modal-footer :state="FooterState.Stop" @stop="emit('stop')"></modal-footer>
</template>
