<script setup lang="ts">
import { injectStrict, translationsKey } from "@/util/translations"

const props = defineProps<{
  helpBaseUrl: URL
}>()

const helpUrl = new URL("/deeplink", props.helpBaseUrl).toString()

const t = injectStrict(translationsKey)
</script>

<template>
  <section class="website" data-testid="website_link">
    <p>{{ t("help_title") }}</p>
    <p>
      <a :href="helpUrl" class="link">
        <svg width="16" height="16" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6.4 18.5 5 17.1l9.6-9.6H6v-2h12v12h-2V8.9z" />
        </svg>
        <span>{{ t("help_to_website") }}</span>
      </a>
    </p>
  </section>
</template>
