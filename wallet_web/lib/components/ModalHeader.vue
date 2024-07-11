<script setup lang="ts">
import { injectStrict, translationsKey } from "@/util/translations"

const t = injectStrict(translationsKey)
</script>

<template>
  <header>
    <h1>{{ t("modal_header") }}</h1>
  </header>
</template>
