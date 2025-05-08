<script setup lang="ts">
import { cancelSession } from "@/api/cancel"
import { createSession } from "@/api/session"
import { getStatus } from "@/api/status"
import ModalFooter from "@/components/ModalFooter.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import ModalMain from "@/components/ModalMain.vue"
import { type ModalState, type Session } from "@/models/state"
import { type SessionType } from "@/models/status"
import { errorTypeOrDefault } from "@/util/error_type"
import { isMobileKey } from "@/util/useragent"
import { inject, onMounted, onUnmounted, ref, watch } from "vue"

const POLL_INTERVAL_IN_MS = 2000

export interface Props {
  usecase: string
  startUrl: URL
  helpBaseUrl: URL
  pollIntervalInMs?: number
}

const props = withDefaults(defineProps<Props>(), {
  pollIntervalInMs: () => POLL_INTERVAL_IN_MS,
})

const emit = defineEmits<{
  close: []
  success: [sessionToken: string, sessionType: SessionType]
  failed: [sessionToken?: string, sessionType?: SessionType]
}>()

const isMobile = inject(isMobileKey)

const pollHandle = ref<NodeJS.Timeout>()
const modalState = ref<ModalState>({ kind: "creating" })

watch(modalState, (state) => {
  switch (state.kind) {
    case "created":
    case "in-progress": {
      pollStatus(state.session)
      break
    }
    case "creating":
    case "loading":
    case "success":
    case "confirm-stop":
    case "error": {
      cancelPolling()
      break
    }
  }
})

const pollStatus = (session: Session) => {
  if (pollHandle.value) {
    cancelPolling()
  }

  pollHandle.value = setTimeout(async () => await checkStatus(session), props.pollIntervalInMs)
}

const cancelPolling = () => {
  if (pollHandle.value) {
    clearTimeout(pollHandle.value)
  }
}

const startSession = async () => {
  try {
    modalState.value = { kind: "creating" }

    let response = await createSession(props.startUrl, {
      usecase: props.usecase,
    })
    await checkStatus({
      statusUrl: response.status_url,
      sessionType: isMobile ? "same_device" : "cross_device",
      sessionToken: response.session_token,
    })
  } catch (e) {
    modalState.value = {
      kind: "error",
      errorType: errorTypeOrDefault(e),
      // session is undefined
    }
  }
}

const checkStatus = async (session: Session) => {
  try {
    let statusResponse = await getStatus(session.statusUrl, session.sessionType)

    switch (statusResponse.status) {
      case "CREATED":
        modalState.value = {
          kind: "created",
          sameDeviceUl: statusResponse.ul,
          crossDeviceUl: statusResponse.ul,
          session,
        }
        break
      case "WAITING_FOR_RESPONSE":
        modalState.value = {
          kind: "in-progress",
          session,
        }
        break
      case "DONE":
        modalState.value = {
          kind: "success",
          session,
        }
        break
      case "EXPIRED":
        modalState.value = {
          kind: "error",
          errorType: "expired",
          session,
        }
        break
      case "CANCELLED":
        modalState.value = {
          kind: "error",
          errorType: "cancelled",
          session,
        }
        break
      case "FAILED":
        modalState.value = {
          kind: "error",
          errorType: "failed",
          session,
        }
        break
    }
  } catch (e) {
    modalState.value = {
      kind: "error",
      errorType: errorTypeOrDefault(e),
      session,
    }
  }
}

const handleChoice = async (choice: SessionType) => {
  if (modalState.value.kind === "created") {
    cancelPolling()

    let session: Session = {
      statusUrl: modalState.value.session.statusUrl,
      sessionType: choice,
      sessionToken: modalState.value.session.sessionToken,
    }

    if (choice === "cross_device") {
      modalState.value = {
        kind: "loading",
        session,
      }
    }

    await checkStatus(session)
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}

const close = async () => {
  switch (modalState.value.kind) {
    case "success":
      emit("success", modalState.value.session.sessionToken, modalState.value.session.sessionType)
      break
    case "error":
      emit("failed", modalState.value.session?.sessionToken, modalState.value.session?.sessionType)
      break
    case "creating":
    case "loading":
      emit("close")
      break
    default:
      modalState.value = {
        kind: "error",
        errorType: "failed",
        session: modalState.value.session,
      }
  }
}

const stop = async () => {
  if (modalState.value.kind === "created" || modalState.value.kind === "confirm-stop") {
    let kind = modalState.value.kind

    modalState.value = {
      kind: "loading",
      session: modalState.value.session,
    }

    await cancelSession(modalState.value.session.statusUrl)

    if (kind === "created") {
      emit("close")
    } else {
      await checkStatus(modalState.value.session)
    }
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}

const retry = async () => {
  if (modalState.value.kind === "error") {
    await startSession()
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}

const confirm = async () => {
  if (modalState.value.kind === "loading" || modalState.value.kind === "in-progress") {
    modalState.value = {
      kind: "confirm-stop",
      prev: modalState.value,
      session: modalState.value.session,
    }
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}

const back = async () => {
  if (modalState.value.kind === "confirm-stop") {
    modalState.value = modalState.value.prev
    if (modalState.value.kind !== "creating" && modalState.value.session !== undefined) {
      await checkStatus(modalState.value.session)
    } else {
      modalState.value = {
        kind: "error",
        errorType: "failed",
        // session is undefined
      }
    }
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}

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
      :class="[modalState.kind, modalState.kind === 'success' && modalState.session.sessionType]"
      data-testid="wallet_modal"
    >
      <modal-header></modal-header>
      <modal-main :modalState :helpBaseUrl @choice="handleChoice"></modal-main>
      <modal-footer
        :modalState
        @close="close"
        @stop="stop"
        @confirm="confirm"
        @retry="retry"
        @back="back"
      ></modal-footer>
    </aside>
  </div>
</template>
