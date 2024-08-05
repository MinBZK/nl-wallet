<script setup lang="ts">
import { translations, translationsKey, type Language } from "@/util/translations"
import { isDesktop, isMobileKey } from "@/util/useragent"
import { provide, ref } from "vue"
import WalletModal from "./WalletModal.vue"
import { RO_SANS_BOLD, RO_SANS_REGULAR } from "../non-free/fonts"

export interface Props {
  usecase: string
  startUrl?: URL
  text?: string
  lang?: Language
  // ignored, but needed for the browser not giving a warning
  id?: string
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  text: (props): string => translations(props.lang as Language)("wallet_button_text"),
  startUrl: (): URL => new URL(document.location.href),
  lang: (): Language => "nl",
})

const emit = defineEmits<{
  success: [sessionToken: string, sessionType: string]
  failed: [sessionToken?: string, sessionType?: string]
}>()

const isVisible = ref(false)
const button = ref<HTMLDivElement | null>(null)

const isMobile = !isDesktop(window.navigator.userAgent)

const success = (sessionToken: string, sessionType: string) => {
  close()
  emit("success", sessionToken, sessionType)
}

const failed = (sessionToken?: string, sessionType?: string) => {
  close()
  emit("failed", sessionToken, sessionType)
}

const close = () => {
  isVisible.value = false
  button.value && button.value.focus()
}

provide(isMobileKey, isMobile)
provide(translationsKey, translations(props.lang))

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
    ref="button"
    :aria-hidden="isVisible"
    @click="isVisible = true"
    data-testid="wallet_button"
  >
    <span part="button-span">{{ text }}</span>
  </button>
  <wallet-modal
    v-if="isVisible"
    :start-url="startUrl"
    :usecase
    @close="close"
    @success="success"
    @failed="failed"
  ></wallet-modal>
</template>

<style>
@import "../style.css";
</style>
