<script setup lang="ts">
import { injectStrict, translationsKey } from "@/util/translations"

const t = injectStrict(translationsKey)
</script>

<template>
  <svg class="status" width="24" height="24" fill="currentColor">
    <path d="M9.657 11.543v4.628h1.414V9.1H4v1.414h4.686L3 16.2l1 1z" />
    <path
      d="M9 1h10c1.1 0 2 .9 2 2v18c0 1.1-.9 2-2 2H9c-1.1 0-2-.9-2-2v-3h2v2h10V4H9v2H7V3c0-1.1.9-2 2-2"
    />
  </svg>
  <h2 data-testid="in_progress">{{ t("in_progress_title") }}</h2>
</template>
