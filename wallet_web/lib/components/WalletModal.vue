<script setup lang="ts">
import { createSession } from "@/api/session"
import { getStatus } from "@/api/status"
import DeviceChoice from "@/components/DeviceChoice.vue"
import ErrorSection from "@/components/ErrorSection.vue"
import InProgressSection from "@/components/InProgressSection.vue"
import LoadingSection from "@/components/LoadingSection.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import QrCode from "@/components/QrCode.vue"
import SuccessSection from "@/components/SuccessSection.vue"
import { type ModalState, type StatusUrl } from "@/models/modal-state"
import { SessionType } from "@/models/status"
import { isMobileKey } from "@/util/projection_keys"
import { inject, onMounted, onUnmounted, ref, watch } from "vue"

const POLL_INTERVAL_IN_MS = 2000

export interface Props {
  baseUrl: string
  usecase: string
  pollIntervalInMs?: number
}

const props = withDefaults(defineProps<Props>(), {
  pollIntervalInMs: () => POLL_INTERVAL_IN_MS,
})

const emit = defineEmits<{
  close: []
  success: [session_token: string, session_type: SessionType]
}>()

const isMobile = inject(isMobileKey)

const pollHandle = ref<NodeJS.Timeout>()
const modalState = ref<ModalState>({ kind: "loading" })

watch(modalState, (state) => {
  switch (state.kind) {
    case "created":
    case "in-progress": {
      pollStatus(state.status_url, state.session_type, state.session_token)
      break
    }
    case "loading":
    case "success":
    case "error": {
      cancelPolling()
      break
    }
  }
})

function pollStatus(statusUrl: StatusUrl, sessionType: SessionType, session_token: string) {
  if (pollHandle.value) {
    cancelPolling()
  }

  pollHandle.value = setTimeout(
    async () => await checkStatus(statusUrl, sessionType, session_token),
    props.pollIntervalInMs,
  )
}

function cancelPolling() {
  if (pollHandle.value) {
    clearTimeout(pollHandle.value)
  }
}

async function startSession() {
  try {
    modalState.value = { kind: "loading" }

    let response = await createSession(props.baseUrl, {
      usecase: props.usecase,
    })
    await checkStatus(
      response.status_url,
      isMobile ? SessionType.SameDevice : SessionType.CrossDevice,
      response.session_token,
    )
  } catch (e) {
    console.error(e)
    modalState.value = { kind: "error", error_type: "failed" }
  }
}

async function checkStatus(statusUrl: StatusUrl, sessionType: SessionType, session_token: string) {
  try {
    let statusResponse = await getStatus(statusUrl, sessionType)

    switch (statusResponse.status) {
      case "CREATED":
        modalState.value = {
          kind: "created",
          ul: statusResponse.ul,
          status_url: statusUrl,
          session_type: sessionType,
          session_token,
        }
        break
      case "WAITING_FOR_RESPONSE":
        modalState.value = {
          kind: "in-progress",
          status_url: statusUrl,
          session_type: sessionType,
          session_token,
        }
        break
      case "DONE":
        modalState.value = {
          kind: "success",
          session_type: sessionType,
          session_token,
        }
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
  } catch (e) {
    console.error(e)
    modalState.value = {
      kind: "error",
      error_type: "failed",
    }
  }
}

async function handleChoice(choice: SessionType) {
  if (modalState.value.kind === "created") {
    cancelPolling()

    let status_url = modalState.value.status_url
    let session_token = modalState.value.session_token
    if (choice === SessionType.CrossDevice) {
      modalState.value = { kind: "loading" }
    }
    await checkStatus(status_url, choice, session_token)
  } else {
    modalState.value = {
      kind: "error",
      error_type: "failed",
    }
  }
}

function success(session_token: string, session_type: SessionType) {
  emit("success", session_token, session_type)
}

function close() {
  emit("close")
}

async function retry() {
  await startSession()
}

onMounted(async () => {
  await startSession()
})
onUnmounted(cancelPolling)
</script>

<template>
  <div class="modal-anchor">
    <aside class="modal" :class="modalState.kind" data-testid="wallet_modal">
      <modal-header></modal-header>

      <loading-section v-if="modalState.kind === 'loading'" @stop="close"></loading-section>
      <device-choice
        v-if="modalState.kind === 'created' && modalState.session_type === SessionType.SameDevice"
        :ul="modalState.ul"
        @choice="handleChoice"
        @close="close"
      ></device-choice>
      <qr-code
        v-if="modalState.kind === 'created' && modalState.session_type === SessionType.CrossDevice"
        :text="modalState.ul"
        @close="close"
      ></qr-code>
      <in-progress-section
        v-if="modalState.kind === 'in-progress'"
        @stop="close"
      ></in-progress-section>
      <success-section
        v-if="modalState.kind === 'success'"
        :sessionType="modalState.session_type"
        @close="success(modalState.session_token, modalState.session_type)"
      ></success-section>
      <error-section
        v-if="modalState.kind === 'error'"
        :error_type="modalState.error_type"
        @close="close"
        @retry="retry"
      ></error-section>
    </aside>
  </div>
</template>
