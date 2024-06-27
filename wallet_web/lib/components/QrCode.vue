<script setup lang="ts">
import HelpSection from "@/components/HelpSection.vue"
import ModalFooter from "@/components/ModalFooter.vue"
import { FooterState } from "@/models/footer-state"
import { drawCanvas } from "@/util/draw_qr"
import { qrcodegen } from "@/util/qrcodegen"
import { ref, watch } from "vue"

const props = defineProps<{
  text: string
}>()

const emit = defineEmits(["close"])

const canvas = ref<HTMLCanvasElement | null>()

watch(
  [() => props.text, canvas],
  ([text, cv]) => {
    if (cv) {
      const QRC = qrcodegen.QrCode
      const qr = QRC.encodeText(text, QRC.Ecc.LOW)
      drawCanvas(qr, cv, 22)
    }
  },
  { immediate: true },
)
</script>

<template>
  <main>
    <h2>Scan de QR-code met je NL Wallet app</h2>
    <div class="qr" data-testid="qr">
      <canvas ref="canvas"></canvas>
    </div>
  </main>

  <help-section></help-section>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
