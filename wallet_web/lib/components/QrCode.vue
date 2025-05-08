<script setup lang="ts">
import { drawCanvas } from "@/util/draw_qr"
import { qrcodegen } from "@/util/qrcodegen"
import { injectStrict, translationsKey } from "@/util/translations"
import { ref, watch } from "vue"

const props = defineProps<{
  ul: URL
}>()

const t = injectStrict(translationsKey)
const canvas = ref<HTMLCanvasElement | null>()

watch(
  [() => props.ul.toString(), canvas],
  ([text, cv]) => {
    if (cv) {
      const QRC = qrcodegen.QrCode
      const qr = QRC.encodeText(text, QRC.Ecc.LOW)
      drawCanvas(qr, cv)
    }
  },
  { immediate: true },
)
</script>

<template>
  <h2>{{ t("qr_code_title") }}</h2>
  <div class="qr" data-testid="qr">
    <canvas ref="canvas"></canvas>
    <div role="img" class="logo" aria-label='{{ t("qr_code_label") }}'></div>
  </div>
</template>
