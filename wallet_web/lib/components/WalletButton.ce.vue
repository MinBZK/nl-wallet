<script setup lang="ts">
import { ref } from "vue"
import WalletModal from "./WalletModal.vue"

export interface Props {
  text?: string
  baseUrl?: string
  style?: string
}

withDefaults(defineProps<Props>(), {
  text: (): string => "Inloggen met NL Wallet",
  baseUrl: (): string => "http://localhost:3004"
})

const isVisible = ref(false)

function show() {
  isVisible.value = true
}

function hide() {
  isVisible.value = false
}
</script>

<template>
  <button part="button" type="button" class="default-button" :style="style" @click="show" data-testid="wallet_button">{{ text }}</button>
  <wallet-modal v-if="isVisible" :base-url="baseUrl" @close="hide"></wallet-modal>
</template>

<style>
@import "../style.css";
</style>
