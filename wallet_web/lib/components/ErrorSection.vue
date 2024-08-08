<script setup lang="ts">
import { type ErrorType } from "@/models/state"
import { injectStrict, translationsKey } from "@/util/translations"

defineProps<{
  errorType: ErrorType
}>()

const t = injectStrict(translationsKey)
</script>

<template>
  <svg width="24" height="24" fill="currentColor">
    <path
      d="m13 8.2-1-1-4 4-4-4-1 1 4 4-4 4 1 1 4-4 4 4 1-1-4-4zM19 1H9c-1.1 0-2 .9-2 2v3h2V4h10v16H9v-2H7v3c0 1.1.9 2 2 2h10c1.1 0 2-.9 2-2V3c0-1.1-.9-2-2-2"
    />
  </svg>
  <section class="text">
    <template v-if="errorType === 'expired'">
      <h2 data-testid="expired_header">{{ t("expired_title") }}</h2>
      <p>
        {{ t("expired_body") }}
      </p>
    </template>
    <template v-else-if="errorType === 'failed'">
      <h2 data-testid="failed_header">{{ t("failed_title") }}</h2>
      <p>{{ t("failed_body") }}</p>
    </template>
    <template v-else-if="errorType === 'cancelled'">
      <h2 data-testid="cancelled_header">{{ t("cancelled_title") }}</h2>
      <p>{{ t("cancelled_body") }}</p>
    </template>
    <template v-else-if="errorType === 'network'">
      <h2 data-testid="network_header">{{ t("network_title") }}</h2>
      <p>{{ t("network_body") }}</p>
    </template>
  </section>
</template>
