<script setup lang="ts">
import { createSession } from "@/api/session"
import { getStatus } from "@/api/status"
import ModalFooter from "@/components/ModalFooter.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import ModalMain from "@/components/ModalMain.vue"
import { SessionState, type ModalState, type StatusUrl } from "@/models/state"
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
  success: [sessionToken: string, sessionType: SessionType]
}>()

const isMobile = inject(isMobileKey)

const pollHandle = ref<NodeJS.Timeout>()
const modalState = ref<ModalState>({ kind: SessionState.Loading })

watch(modalState, (state) => {
  switch (state.kind) {
    case SessionState.Created:
    case SessionState.InProgress: {
      pollStatus(state.statusUrl, state.sessionType, state.sessionToken)
      break
    }
    case SessionState.Loading:
    case SessionState.Success:
    case SessionState.Error: {
      cancelPolling()
      break
    }
  }
})

const pollStatus = (statusUrl: StatusUrl, sessionType: SessionType, sessionToken: string) => {
  if (pollHandle.value) {
    cancelPolling()
  }

  pollHandle.value = setTimeout(
    async () => await checkStatus(statusUrl, sessionType, sessionToken),
    props.pollIntervalInMs,
  )
}

const cancelPolling = () => {
  if (pollHandle.value) {
    clearTimeout(pollHandle.value)
  }
}

const startSession = async () => {
  try {
    modalState.value = { kind: SessionState.Loading }

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
    modalState.value = { kind: SessionState.Error, errorType: "failed" }
  }
}

const checkStatus = async (
  statusUrl: StatusUrl,
  sessionType: SessionType,
  sessionToken: string,
) => {
  try {
    let statusResponse = await getStatus(statusUrl, sessionType)

    switch (statusResponse.status) {
      case "CREATED":
        modalState.value = {
          kind: SessionState.Created,
          ul: statusResponse.ul,
          statusUrl,
          sessionType,
          sessionToken,
        }
        break
      case "WAITING_FOR_RESPONSE":
        modalState.value = {
          kind: SessionState.InProgress,
          statusUrl,
          sessionType,
          sessionToken,
        }
        break
      case "DONE":
        modalState.value = {
          kind: SessionState.Success,
          sessionType,
          sessionToken,
        }
        break
      case "EXPIRED":
        modalState.value = {
          kind: SessionState.Error,
          errorType: "expired",
        }
        break
      case "CANCELLED":
        modalState.value = {
          kind: SessionState.Error,
          errorType: "cancelled",
        }
        break
      case "FAILED":
        modalState.value = {
          kind: SessionState.Error,
          errorType: "failed",
        }
        break
    }
  } catch (e) {
    console.error(e)
    modalState.value = {
      kind: SessionState.Error,
      errorType: "failed",
    }
  }
}

const handleChoice = async (choice: SessionType) => {
  if (modalState.value.kind === "created") {
    cancelPolling()

    let statusUrl = modalState.value.statusUrl
    let sessionToken = modalState.value.sessionToken
    if (choice === SessionType.CrossDevice) {
      modalState.value = { kind: SessionState.Loading }
    }
    await checkStatus(statusUrl, choice, sessionToken)
  } else {
    modalState.value = {
      kind: SessionState.Error,
      errorType: "failed",
    }
  }
}

const success = (sessionToken: string, sessionType: SessionType) =>
  emit("success", sessionToken, sessionType)
const stop = async () => {
  await cancelPolling()
  emit("close")
} // TODO implement cancelsession
const retry = async () => await startSession()

onMounted(async () => {
  await startSession()
})

onUnmounted(cancelPolling)
</script>

<template>
  <div class="modal-anchor">
    <aside
      aria-modal="true"
      role="dialog"
      aria-label="NL Wallet"
      class="modal"
      :class="[modalState.kind, modalState.kind == SessionState.Success && modalState.sessionType]"
      data-testid="wallet_modal"
    >
      <modal-header></modal-header>
      <modal-main :modalState="modalState" @choice="handleChoice"></modal-main>
      <modal-footer
        :state="modalState.kind"
        :type="modalState.kind == SessionState.Success ? modalState.sessionType : null"
        @retry="retry"
        @close="emit('close')"
        @stop="stop"
        @success="
          modalState.kind == SessionState.Success &&
            success(modalState.sessionToken, modalState.sessionType)
        "
      ></modal-footer>
    </aside>
  </div>
</template>
