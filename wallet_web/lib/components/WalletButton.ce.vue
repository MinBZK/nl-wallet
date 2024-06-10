<script setup lang="ts">
import { isMobileKey } from "@/util/projection_keys"
import { isDesktop } from "@/util/useragent"
import { provide, ref } from "vue"
import WalletModal from "./WalletModal.vue"

export interface Props {
  usecase: string
  id?: string
  text?: string
  baseUrl?: string
  style?: string
}

withDefaults(defineProps<Props>(), {
  text: (): string => "Inloggen met NL Wallet",
  baseUrl: (): string => "",
})

const emit = defineEmits<{
  success: [session_token: string, session_type: string]
}>()

const isVisible = ref(false)
const isMobile = !isDesktop(window.navigator.userAgent)

function show() {
  isVisible.value = true
}

function hide() {
  isVisible.value = false
}

function success(session_token: string, session_type: string) {
  isVisible.value = false
  emit("success", session_token, session_type)
}

provide(isMobileKey, isMobile)
</script>

<template>
  <button
    part="button"
    type="button"
    class="default-button primary"
    :id="id"
    :style="style"
    @click="show"
    data-testid="wallet_button"
  >
    {{ text }}
  </button>
  <wallet-modal
    v-if="isVisible"
    :base-url="baseUrl"
    :usecase
    @close="hide"
    @success="success"
  ></wallet-modal>
</template>

<style>
@import "../style.css";
</style>
