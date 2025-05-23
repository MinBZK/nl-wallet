<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import ModalHeader from "@/components/ModalHeader.vue"
import ModalMain from "@/components/ModalMain.vue"
import { type SessionType } from "@/models/openid4vc"
import { type ModalState } from "@/models/state"
import { isMobileKey } from "@/util/useragent"
import { ref, inject } from "vue"

const props = defineProps<{
  sameDeviceUl: URL
  crossDeviceUl: URL
  helpBaseUrl: URL
}>()

const emit = defineEmits<{
  close: []
}>()

const isMobile = inject(isMobileKey)

const modalState = ref<ModalState>({
  kind: "created",
  sameDeviceUl: props.sameDeviceUl,
  crossDeviceUl: props.crossDeviceUl,
  session: {
    // TODO this statusUrl is currently unused (PVW-4365)
    statusUrl: new URL("http://status.example.com/status"),
    sessionType: isMobile ? "same_device" : "cross_device",
    sessionToken: "",
  },
})

const handleChoice = async (choice: SessionType) => {
  if (modalState.value.kind === "created") {
    modalState.value.session = {
      statusUrl: modalState.value.session.statusUrl,
      sessionType: choice,
      sessionToken: "",
    }
  } else {
    modalState.value = {
      kind: "error",
      errorType: "failed",
      session: modalState.value.kind !== "creating" ? modalState.value.session : undefined,
    }
  }
}
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
      <modal-footer :modalState @stop="emit('close')"></modal-footer>
    </aside>
  </div>
</template>
