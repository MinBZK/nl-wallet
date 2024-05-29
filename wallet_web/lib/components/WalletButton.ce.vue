<script setup lang="ts">
import { isMobileKey } from "@/util/projection_keys"
import { isDesktop } from "@/util/useragent"
import { provide, ref } from "vue"
import WalletModal from "./WalletModal.vue"

export interface Props {
  usecase: string
  text?: string
  baseUrl?: string
  style?: string
}

withDefaults(defineProps<Props>(), {
  text: (): string => "Inloggen met NL Wallet",
  baseUrl: (): string => "http://localhost:3004",
})

const isVisible = ref(false)
const isMobile = !isDesktop(window.navigator.userAgent)

function show() {
  isVisible.value = true
}

function hide() {
  isVisible.value = false
}

function open_universal_link(url: string) {
  window.open(url, "_blank")
}

provide(isMobileKey, isMobile)
</script>

<template>
  <button
    part="button"
    type="button"
    class="default-button primary"
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
    @open_link="open_universal_link"
  ></wallet-modal>
</template>

<style>
@import "../style.css";
</style>
