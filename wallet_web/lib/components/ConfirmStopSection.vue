<script setup lang="ts">
import HelpLink from "@/components/HelpLink.vue"
import { injectStrict, translationsKey } from "@/util/translations"

defineProps<{
  helpBaseUrl: URL
}>()

const t = injectStrict(translationsKey)
</script>

<template>
  <section class="text">
    <h2 data-testid="confirm_stop">{{ t("confirm_stop_title") }}</h2>
    <p>
      {{ t("confirm_stop_body") }}
    </p>
  </section>
  <help-link :helpBaseUrl></help-link>
</template>
