<script setup lang="ts">
import LoadingIndicator from "@/components/LoadingIndicator.vue"
</script>

<template>
  <section data-testid="loading" class="text">
    <h2>Even geduld</h2>
    <p>De gegevens worden opgehaald</p>
  </section>
  <loading-indicator></loading-indicator>
</template>
