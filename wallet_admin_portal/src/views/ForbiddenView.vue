<template>
  <PageErrorState
    title="Je hebt geen toegang"
    description="Je kunt deze pagina niet bekijken. Ga terug naar je taken."
    action-label="Naar het beginscherm"
    @retry="goHome"
  />
</template>

<script setup lang="ts">
import { useNavigation } from '@/composables/navigation.ts'
import PageErrorState from '@/components/ui/PageErrorState.vue'

const { goHome } = useNavigation()
</script>
