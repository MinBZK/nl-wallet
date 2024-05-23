<script setup lang="ts">
import { createEngagement } from "@/api/engagement"
import { getStatus } from "@/api/status"
import DeviceChoice from "@/components/DeviceChoice.vue"
import Error from "@/components/Error.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import QrCode from "@/components/QrCode.vue"
import Success from "@/components/Success.vue"
import { type EngagementUrls, SessionType } from "@/models/engagement"
import { Status } from "@/models/status"
import { isDesktop } from "@/util/useragent"
import { onUnmounted, ref, watch } from "vue"

const POLL_INTERVAL_IN_MS = 2000

export interface Props {
  baseUrl: string
  pollIntervalInMs?: number
}

enum ShowComponent {
  DeviceChoice,
  Qr,
  Success,
  Error,
}

const props = withDefaults(defineProps<Props>(), {
  pollIntervalInMs: () => POLL_INTERVAL_IN_MS
})

const emit = defineEmits(["close"])

const urls = ref<EngagementUrls | null>()
const engagementUrl = ref<string | null>()
const sessionType = ref<SessionType | null>(isDesktop(window.navigator.userAgent) ? SessionType.CrossDevice : null)
const pollHandle = ref()
const showComponent = ref<ShowComponent>()

watch(sessionType, async (type) => {
  if (type) {
    let response = await createEngagement(props.baseUrl, { session_type: type, usecase: "mijn_amsterdam" })
    if (response.urls) {
      urls.value = response.urls
      await checkStatus(response.urls.status_url)
      startPolling(response.urls.status_url)
    }
  }
}, { immediate: true })

watch([engagementUrl, sessionType], ([url, type]) => {
  if (!!url && !!type) {
    if (type === SessionType.SameDevice) {
      window.open(url, "_blank")
      return
    } else if (type === SessionType.CrossDevice) {
      showComponent.value = ShowComponent.Qr
      return
    }
  }

  if (!url && !type) {
    showComponent.value = ShowComponent.DeviceChoice
    return
  }
}, { immediate: true })

function startPolling(statusUrl: string) {
  pollHandle.value = setInterval(async () => await checkStatus(statusUrl), props.pollIntervalInMs)
}

function cancelPolling() {
  if (pollHandle.value) {
    clearInterval(pollHandle.value)
  }
}

async function checkStatus(statusUrl: string) {
  let statusResponse = await getStatus(statusUrl)
  switch (statusResponse.status) {
    case Status.Created:
      engagementUrl.value = statusResponse.engagement_url
      break
    case Status.WaitingForResponse:
      console.log(statusResponse)
      break
    case Status.Done:
      console.log(statusResponse)
      showComponent.value = ShowComponent.Success
      break
    case Status.Cancelled:
    case Status.Failed:
      console.log(statusResponse)
      showComponent.value = ShowComponent.Error
      cancelPolling()
      break
  }
}

async function handleChoice(choice: SessionType) {
  sessionType.value = choice
}

onUnmounted(cancelPolling)
</script>

<template>
  <aside
    class="modal reset visible" data-testid="wallet_modal">

    <div class="modal-guts">
      <modal-header></modal-header>
      <article>
        <device-choice v-if="showComponent === ShowComponent.DeviceChoice" @choice="handleChoice"></device-choice>
        <qr-code v-if="showComponent === ShowComponent.Qr && engagementUrl" :text="engagementUrl"></qr-code>
        <success v-if="showComponent === ShowComponent.Success"></success>
        <error v-if="showComponent === ShowComponent.Error"></error>
      </article>
      <modal-footer @close="emit('close')"></modal-footer>
    </div>
  </aside>

  <div class="bg"></div>
</template>
