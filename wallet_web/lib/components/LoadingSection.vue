<script setup lang="ts">
import LoadingIndicator from "@/components/LoadingIndicator.vue"
import { injectStrict, translationsKey } from "@/util/translations"

const t = injectStrict(translationsKey)
</script>

<template>
  <section data-testid="loading" class="text">
    <h2>{{ t("loading_title") }}</h2>
    <p>{{ t("loading_body") }}</p>
  </section>
  <loading-indicator></loading-indicator>
</template>
