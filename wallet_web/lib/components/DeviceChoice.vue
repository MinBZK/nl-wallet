<script setup lang="ts">
import { SessionType } from "@/models/engagement"

const emit = defineEmits(["choice"])
</script>

<template>
  <h2>Op welk apparaat staat je NL Wallet app?</h2>
  <section class="device-choice" data-testid="device_choice">
    <button type="button" @click="emit('choice', SessionType.SameDevice)">Op dit apparaat</button>
    <button type="button" @click="emit('choice', SessionType.CrossDevice)">Op een ander apparaat</button>
  </section>
</template>
