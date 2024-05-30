<script setup lang="ts">
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { drawCanvas } from "@/util/draw_qr"
import { qrcodegen } from "@/util/qrcodegen"
import { ref, watch } from "vue"

const props = defineProps<{
  text: string
  small: boolean
}>()

const emit = defineEmits(["close"])

const canvas = ref<HTMLCanvasElement | null>()

watch(
  [() => props.text, () => props.small, canvas],
  ([text, small, cv]) => {
    if (cv) {
      const QRC = qrcodegen.QrCode
      const qr = QRC.encodeText(text, QRC.Ecc.LOW)
      drawCanvas(qr, small ? 3.4 : 4.5, 1, "#FFFFFF", "#000000", cv)
    }
  },
  { immediate: true },
)
</script>

<template>
  <article>
    <h2>Scan de QR-code met je NL Wallet app</h2>
    <section class="qr" data-testid="qr">
      <canvas ref="canvas"></canvas>
    </section>
  </article>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
