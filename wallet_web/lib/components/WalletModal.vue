<script setup lang="ts">
import DynamicWalletModal from "@/components/DynamicWalletModal.vue"
import StaticWalletModal from "@/components/StaticWalletModal.vue"
import { type SessionType } from "@/models/openid4vc"
import { type ModalType } from "@/models/state"

defineProps<{
  modalType: ModalType
  helpBaseUrl: URL
}>()

const emit = defineEmits<{
  close: []
  success: [sessionToken: string, sessionType: SessionType]
  failed: [sessionToken?: string, sessionType?: SessionType]
}>()
</script>

<template>
  <dynamic-wallet-modal
    v-if="modalType.strategy === 'dynamic'"
    :startUrl="modalType.startUrl"
    :usecase="modalType.usecase"
    :helpBaseUrl
    @close="emit('close')"
    @success="(...args) => emit('success', ...args)"
    @failed="(...args) => emit('failed', ...args)"
  ></dynamic-wallet-modal>

  <static-wallet-modal
    v-if="modalType.strategy === 'static'"
    :sameDeviceUl="modalType.sameDeviceUl"
    :crossDeviceUl="modalType.crossDeviceUl"
    :helpBaseUrl
    @close="emit('close')"
  ></static-wallet-modal>
</template>
