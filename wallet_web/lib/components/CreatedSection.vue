<script setup lang="ts">
import DeviceChoice from "@/components/DeviceChoice.vue"
import QrCode from "@/components/QrCode.vue"
import { type SessionType } from "@/models/status"

defineProps<{
  sameDeviceUl: URL
  crossDeviceUl: URL
  sessionType: SessionType
}>()

const emit = defineEmits<{
  choice: [sessionType: SessionType]
}>()

const handleChoice = (sessionType: SessionType) => emit("choice", sessionType)
</script>

<template>
  <device-choice v-if="sessionType === 'same_device'" :ul="sameDeviceUl" @choice="handleChoice"></device-choice>
  <qr-code v-if="sessionType === 'cross_device'" :ul="crossDeviceUl"></qr-code>
</template>
