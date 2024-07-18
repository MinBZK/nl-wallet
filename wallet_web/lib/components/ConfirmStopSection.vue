<script setup lang="ts">
import { injectStrict, translationsKey } from "@/util/translations"

const t = injectStrict(translationsKey)
</script>

<template>
  <section class="text">
    <h2 data-testid="confirm_stop">{{ t("confirm_stop_title") }}</h2>
    <p>
      {{ t("confirm_stop_body") }}
    </p>
  </section>
</template>
