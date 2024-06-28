<script setup lang="ts">
import { createAbsoluteUrl } from "@/util/base_url"
import { isMobileKey } from "@/util/projection_keys"
import { isDesktop } from "@/util/useragent"
import { computed, provide, ref } from "vue"
import WalletModal from "./WalletModal.vue"
import { RO_SANS_BOLD, RO_SANS_REGULAR } from "../non-free/fonts"

export interface Props {
  usecase: string
  id?: string
  text?: string
  baseUrl?: string
  style?: string
}

const props = withDefaults(defineProps<Props>(), {
  text: (): string => "Inloggen met NL Wallet",
  baseUrl: (): string => "",
})

const emit = defineEmits<{
  success: [session_token: string, session_type: string]
}>()

const isVisible = ref(false)
const isMobile = !isDesktop(window.navigator.userAgent)
const absoluteBaseUrl = computed(() =>
  createAbsoluteUrl(props.baseUrl, window.location.href, window.location.pathname),
)

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

// @font-face doesn't seem to be working in the shadow DOM, so we insert it into the parent
// document instead.
let fontFaceSheet = new CSSStyleSheet()
fontFaceSheet.replaceSync(`@font-face {
  font-family: "RO Sans";
  font-weight: normal;
  font-style: normal;
  src: url(data:application/font-woff2;charset=utf-8;base64,${RO_SANS_REGULAR}) format('woff2');
}

@font-face {
  font-family: "RO Sans";
  font-weight: bold;
  font-style: normal;
  src: url(data:application/font-woff2;charset=utf-8;base64,${RO_SANS_BOLD}) format('woff2');
}`)
document.adoptedStyleSheets = [...document.adoptedStyleSheets, fontFaceSheet]
</script>

<template>
  <button
    part="button"
    type="button"
    class="nl-wallet-button"
    :id="id"
    :style="style"
    @click="show"
    data-testid="wallet_button"
  >
    {{ text }}
  </button>
  <Transition name="fade">
    <wallet-modal
      v-if="isVisible"
      :base-url="absoluteBaseUrl"
      :usecase
      @close="hide"
      @success="success"
    ></wallet-modal>
  </Transition>
</template>

<style>
@import "../style.css";
</style>
