<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { type ErrorType } from "@/models/modal-state"

defineProps<{
  error_type: ErrorType
}>()

const emit = defineEmits(["close", "retry"])
</script>

<template>
  <main class="error">
    <svg width="24" height="24" fill="currentColor">
      <path
        d="m13 8.2-1-1-4 4-4-4-1 1 4 4-4 4 1 1 4-4 4 4 1-1-4-4zM19 1H9c-1.1 0-2 .9-2 2v3h2V4h10v16H9v-2H7v3c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V3c0-1.1-.9-2-2-2"
      />
    </svg>
    <section class="text">
      <template v-if="error_type === 'expired'">
        <h2 data-testid="expired_header">Sorry, de tijd is voorbij</h2>
        <p>
          Deze actie is gestopt omdat er teveel tijd voorbij is gegaan. Dit is bedoeld om je
          gegevens veilig te houden. Probeer het opnieuw.
        </p>
      </template>
      <template v-else-if="error_type === 'failed'">
        <h2 data-testid="failed_header">Sorry, er gaat iets mis</h2>
        <p>Deze actie is niet gelukt. Dit kan verschillende redenen hebben. Probeer het opnieuw.</p>
      </template>
      <template v-else-if="error_type === 'cancelled'">
        <h2 data-testid="cancelled_header">Gestopt</h2>
        <p>Omdat je bent gestopt zijn er geen gegevens gedeeld.</p>
      </template>
    </section>
  </main>

  <modal-footer
    :state="FooterState.Retry"
    @close="emit('close')"
    @retry="emit('retry')"
  ></modal-footer>
</template>
