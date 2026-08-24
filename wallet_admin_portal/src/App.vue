<template>
  <main v-if="isFullscreen">
    <RouterView />
  </main>
  <PageLayout v-else>
    <RouterView />
  </PageLayout>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { RouterView, useRoute } from 'vue-router'
import PageLayout from './layout/PageLayout.vue'

const route = useRoute()
const isFullscreen = computed(() => !!route.meta.fullscreen)
</script>
