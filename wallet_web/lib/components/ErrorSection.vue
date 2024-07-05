<script setup lang="ts">
import { type ErrorType } from "@/models/state"

defineProps<{
  errorType: ErrorType
}>()
</script>

<template>
  <svg width="24" height="24" fill="currentColor">
    <path
      d="m13 8.2-1-1-4 4-4-4-1 1 4 4-4 4 1 1 4-4 4 4 1-1-4-4zM19 1H9c-1.1 0-2 .9-2 2v3h2V4h10v16H9v-2H7v3c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V3c0-1.1-.9-2-2-2"
    />
  </svg>
  <section class="text">
    <template v-if="errorType === 'expired'">
      <h2 data-testid="expired_header">Sorry, de tijd is voorbij</h2>
      <p>
        Deze actie is gestopt omdat er teveel tijd voorbij is gegaan. Dit is bedoeld om je gegevens
        veilig te houden. Probeer het opnieuw.
      </p>
    </template>
    <template v-else-if="errorType === 'failed'">
      <h2 data-testid="failed_header">Sorry, er gaat iets mis</h2>
      <p>Deze actie is niet gelukt. Dit kan verschillende redenen hebben. Probeer het opnieuw.</p>
    </template>
    <template v-else-if="errorType === 'cancelled'">
      <h2 data-testid="cancelled_header">Gestopt</h2>
      <p>Omdat je bent gestopt zijn er geen gegevens gedeeld.</p>
    </template>
  </section>
</template>
