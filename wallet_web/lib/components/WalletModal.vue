<script setup lang="ts">
import { getStatus } from "@/api/status"
import ErrorSection from "@/components/ErrorSection.vue"
import InProgressSection from "@/components/InProgressSection.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import QrCode from "@/components/QrCode.vue"
import StartEngagement from "@/components/StartEngagement.vue"
import SuccessSection from "@/components/SuccessSection.vue"
import { useWindowsSize } from "@/composables/use-windows-size"
import { type EngagementUrls, SessionType } from "@/models/engagement"
import { type ModalState, type StatusUrl } from "@/models/modal-state"
import { onUnmounted, ref, watch } from "vue"

const POLL_INTERVAL_IN_MS = 2000

export interface Props {
  baseUrl: string
  usecase: string
  pollIntervalInMs?: number
}

const props = withDefaults(defineProps<Props>(), {
  pollIntervalInMs: () => POLL_INTERVAL_IN_MS,
})

const emit = defineEmits(["close", "open_link"])

const pollHandle = ref()
const modalState = ref<ModalState>({ kind: "starting" })
const linkOpened = ref(false)

const { height } = useWindowsSize()

watch(modalState, (state) => {
  switch (state.kind) {
    case "starting": {
      cancelPolling()
      break
    }
    case "created": {
      if (state.session_type === SessionType.SameDevice && !linkOpened.value) {
        linkOpened.value = true
        emit("open_link", state.engagement_url)
      }
      startPolling(state.status_url, state.session_type)
      break
    }
    case "in_progress": {
      break
    }
    case "success":
    case "error": {
      cancelPolling()
      break
    }
  }
})

function startPolling(statusUrl: StatusUrl, sessionType: SessionType) {
  if (!pollHandle.value) {
    pollHandle.value = setInterval(
      async () => await checkStatus(statusUrl, sessionType),
      props.pollIntervalInMs,
    )
  }
}

function cancelPolling() {
  if (pollHandle.value) {
    clearInterval(pollHandle.value)
  }
}

async function checkStatus(statusUrl: StatusUrl, sessionType: SessionType) {
  try {
    let statusResponse = await getStatus(statusUrl)

    switch (statusResponse.status) {
      case "CREATED":
        modalState.value = {
          kind: "created",
          engagement_url: statusResponse.engagement_url,
          status_url: statusUrl,
          session_type: sessionType,
        }
        break
      case "WAITING_FOR_RESPONSE":
        modalState.value = { kind: "in_progress" }
        break
      case "DONE":
        modalState.value = { kind: "success", session_type: sessionType }
        break
      case "EXPIRED":
        modalState.value = {
          kind: "error",
          error_type: "expired",
        }
        break
      case "CANCELLED":
        modalState.value = {
          kind: "error",
          error_type: "cancelled",
        }
        break
      case "FAILED":
        modalState.value = {
          kind: "error",
          error_type: "failed",
        }
        break
    }
  } catch {
    modalState.value = {
      kind: "error",
      error_type: "failed",
    }
  }
}

async function engagementStarted(type: SessionType, urls: EngagementUrls) {
  await checkStatus(urls.status_url, type)
}

function close() {
  emit("close")
}

function retry() {
  modalState.value = { kind: "starting" }
  linkOpened.value = false
}

onUnmounted(cancelPolling)
</script>

<template>
  <div class="modal-anchor">
    <aside class="modal reset visible" data-testid="wallet_modal">
      <modal-header></modal-header>

      <start-engagement
        v-if="modalState.kind === 'starting'"
        :baseUrl
        :usecase
        @close="close"
        @created="engagementStarted"
        @failed="
          modalState = {
            kind: 'error',
            error_type: 'failed',
          }
        "
      ></start-engagement>
      <qr-code
        v-if="modalState.kind === 'created' && modalState.session_type === SessionType.CrossDevice"
        :text="modalState.engagement_url"
        :small="height <= 900"
        @close="close"
      ></qr-code>
      <in-progress-section
        v-if="
          modalState.kind === 'in_progress' ||
          (modalState.kind === 'created' && modalState.session_type === SessionType.SameDevice)
        "
        @stop="close"
      ></in-progress-section>
      <success-section
        v-if="modalState.kind === 'success'"
        :sessionType="modalState.session_type"
        @close="close"
      ></success-section>
      <error-section
        v-if="modalState.kind === 'error'"
        :error_type="modalState.error_type"
        @close="close"
        @retry="retry"
      ></error-section>
    </aside>
  </div>
  <div class="bg"></div>
</template>
