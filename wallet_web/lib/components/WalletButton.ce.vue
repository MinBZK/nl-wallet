<script setup lang="ts">
import WalletModal from "@/components/WalletModal.vue"
import { type ModalType } from "@/models/state"
import { RO_SANS_BOLD, RO_SANS_REGULAR } from "@/non-free/fonts"
import { type Language, translations, translationsKey } from "@/util/translations"
import { isDesktop, isMobileKey } from "@/util/useragent"
import { provide, ref } from "vue"

export interface Props {
  usecase?: string
  startUrl?: URL
  sameDeviceUl?: URL
  crossDeviceUl?: URL
  text?: string
  lang?: Language
  helpBaseUrl?: URL
  // ignored, but needed for the browser not giving a warning
  id?: string
  class?: string
}

const props = withDefaults(defineProps<Props>(), {
  text: (props): string => translations(props.lang as Language)("wallet_button_text"),
  startUrl: (): URL => new URL(document.location.href),
  helpBaseUrl: (): URL => new URL(import.meta.env.VITE_HELP_BASE_URL),
  lang: (): Language => "nl",
})

const emit = defineEmits<{
  success: [sessionToken: string, sessionType: string]
  failed: [sessionToken?: string, sessionType?: string]
}>()

const modalType: ModalType = props.usecase
  ? { strategy: "dynamic", usecase: props.usecase!, startUrl: props.startUrl! }
  : { strategy: "static", sameDeviceUl: props.sameDeviceUl!, crossDeviceUl: props.crossDeviceUl! }

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

const appVersion = __APP_VERSION__

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
  <meta itemprop="version" :content="appVersion" />
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
    :modalType
    :helpBaseUrl
    @close="close"
    @success="success"
    @failed="failed"
  ></wallet-modal>
</template>

<style>
@import "../style.css";
</style>
