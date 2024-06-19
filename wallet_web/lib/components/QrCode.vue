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
      const qr = QRC.encodeText(text, QRC.Ecc.MEDIUM)
      drawCanvas(qr, small ? 3.0 : 4.0, 1, "#FFFFFF", "#000000", cv)
    }
  },
  { immediate: true },
)
</script>

<template>
  <article>
    <h2>Scan de QR-code met je NL Wallet app</h2>
    <section class="qr" data-testid="qr">
      <img
        id="logo"
        src="data:image/png;charset=utf-8;base64,iVBORw0KGgoAAAANSUhEUgAAADEAAAAwCAMAAACPHmKLAAAAxlBMVEX////////9/P/7+v9XUOxMVOb29f/4+P9TUupKVeRPU+dVUevv7v/09P9SUulQU+jo5v/x8P9ZUO3i3//Z1f/s6//q6P/m5P/Pyv+zrf3d2v/Hwf/z8v/j4f/Tz/9cZ+daT+7f3f/a1//V0f9pYO/QzP+2sf18fu7Kxv+zs/ZqX/BfXOzIwv/T1PmdnvNqYu9oaOxvc+teZ+fq6f7BvP6+vPqAdvNwZfFTX+WytPSpqPSoqfORi/OJku10fupdZelbXulYX+hI2HUlAAAAAXRSTlOArV5bRgAAAgpJREFUSMft1euSmjAYBmBRIeomLBESXJrQBVfQ6m61e+r5cP831Y/EEEjpTPuz030Hx5HJM++nBByNvL/L6EX892I8GG/siO7qiQlSLxPlXKFWIxQMBCFljLDr5df38/nc9+/0qfoyDGez2cXF/ikOlHGFhPUN8H8Qdeo2PIvpdF8QIEYYgL7MlYDcqnNirwGI6WeqiRUAglMrat17d66AvGVAYK6OQEi8NsBfmLGsiBgNgFgxQYR2xAfPO5087wjgLGSESVeMQeBWLBa1R45HGMtULG+KFEp6IhCsIxb3HoHJdlaUkruC8kb4Z/EJTt5/3FuRxM1YPYFbsYB83+3qMDQAxLpQX6Qnoo64hGigK5Y3eZY6HcQIVaGBIwY7/I7oDrV8A4KJrkDECgC2Q4PrYfEOQCtCLaa/FRMl7FBhb6hrLUivI3CErmg7noe+Oa/NUO4vBR1P+no41/xRgWHx8Os1R5Slu2+OMEM9X+Wwr1whWFok28eqqq76qaqH7Wad2b1rdzuXWZKvtofDq14O2xUANZQRZrvjCMg632xWvWzydZLFqboHnfucskjGWVkm/ZRlVkgAusIKIIRiHqVSxk5kmnIFXAEtgmLGOOeRCoejCWMYCxJoYAREkYAISjFWhw58pEIQ95loa8xzl8BB1BsEDTx3rdFBjZ3YmPWuAANqMP/a/+AfiJ9GkkQDQeYpEQAAAABJRU5ErkJggg=="
        alt="NL Wallet logo"
      />
      <canvas ref="canvas"></canvas>
    </section>
  </article>

  <modal-footer :state="FooterState.Close" @close="emit('close')"></modal-footer>
</template>
