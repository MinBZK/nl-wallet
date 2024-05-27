<script setup lang="ts">
import { drawCanvas } from "@/util/draw_qr"
import { qrcodegen } from "@/util/qrcodegen"
import { ref, watch } from "vue"

const props = defineProps<{
  text: string
}>()

const canvas = ref<HTMLCanvasElement | null>()

watch([() => props.text, canvas], ([text, cv]) => {
  if (cv) {
    const QRC = qrcodegen.QrCode
    const qr = QRC.encodeText(text, QRC.Ecc.LOW)
    drawCanvas(qr, 4, 1, "#FFFFFF", "#000000", cv)
  }
}, { immediate: true })
</script>

<template>
  <h2 data-testid="qr_header">Scan de QR-code met je NL Wallet app</h2>
  <section class="qr">
    <canvas ref="canvas"></canvas>
  </section>
</template>
