<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { type ErrorType } from "@/models/modal-state"
import { mdiCellphoneRemove } from "@mdi/js"

defineProps<{
  error_type: ErrorType
}>()

const emit = defineEmits(["close", "retry"])
</script>

<template>
  <article>
    <section class="error status-update">
      <div>
        <svg fill="currentColor" width="24" height="24" viewBox="0 0 24 24">
          <path :d="mdiCellphoneRemove"></path>
        </svg>
      </div>
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
  </article>

  <modal-footer
    :state="FooterState.Retry"
    @close="emit('close')"
    @retry="emit('retry')"
  ></modal-footer>
</template>
