<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"

const emit = defineEmits(["stop"])
</script>

<template>
  <main class="in-progress" data-testid="in_progress">
    <svg width="24" height="24" fill="currentColor">
      <path d="M9.657 11.543v4.628h1.414V9.1H4v1.414h4.686L3 16.2l1 1z" />
      <path
        d="M9 1h10c1.1 0 2 .9 2 2v18c0 1.1-.9 2-2 2H9c-1.1 0-2-.9-2-2v-3h2v2h10V4H9v2H7V3c0-1.1.9-2 2-2"
      />
    </svg>
    <h2>Volg de stappen in de NL Wallet app</h2>
  </main>

  <modal-footer :state="FooterState.Stop" @stop="emit('stop')"></modal-footer>
</template>
