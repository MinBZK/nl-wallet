<script setup lang="ts">
import { createEngagement } from "@/api/engagement"
import DeviceChoice from "@/components/DeviceChoice.vue"
import { type EngagementUrls, SessionType } from "@/models/engagement"
import { isMobileKey } from "@/util/projection_keys"
import { inject, ref, watch } from "vue"

const props = defineProps<{
  baseUrl: string
  usecase: string
}>()

const isMobile = inject(isMobileKey)

const emit = defineEmits<{
  created: [sessionType: SessionType, urls: EngagementUrls]
  failed: []
  close: []
}>()

const sessionType = ref<SessionType | null>(initSessionType())

function initSessionType(): SessionType | null {
  return isMobile ? null : SessionType.CrossDevice
}

async function handleChoice(choice: SessionType) {
  sessionType.value = choice
}

async function startEngagement(type: SessionType) {
  try {
    let response = await createEngagement(props.baseUrl, {
      session_type: type,
      usecase: props.usecase,
    })
    emit("created", type, response.urls)
  } catch {
    emit("failed")
  }
}

watch(
  sessionType,
  async (type) => {
    if (type) {
      await startEngagement(type)
    }
  },
  { immediate: true },
)
</script>

<template>
  <device-choice v-if="isMobile" @choice="handleChoice" @close="emit('close')"></device-choice>
</template>
