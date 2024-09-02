<script setup lang="ts">
import { injectStrict, translationsKey } from "@/util/translations"

const props = defineProps<{
  helpBaseUrl: URL
}>()

const helpUrl = new URL("/help", props.helpBaseUrl).toString()

const t = injectStrict(translationsKey)
</script>

<template>
  <a :href="helpUrl" class="link" data-testid="help">
    <span>{{ t("need_help") }}</span>
    <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
      <path d="M6.4 18.5 5 17.1l9.6-9.6H6v-2h12v12h-2V8.9z" />
    </svg>
  </a>
</template>
