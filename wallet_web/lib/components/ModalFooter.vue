<script setup lang="ts">
import { mdiArrowTopRightThin, mdiWindowClose } from "@mdi/js"

const emit = defineEmits(["close"])

function close() {
  emit("close", { success: "42" })
}
</script>

<template>
  <section class="help">
    <div>Nog geen NL Wallet app? Of hulp nodig?</div>
    <div>
      <a href="/" class="link">
        <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
          <path :d="mdiArrowTopRightThin"></path>
        </svg>
        <span>Naar NL Wallet website</span>
      </a>
    </div>
  </section>

  <section class="footer">
    <button type="button" class="link" @click="close">
      <svg fill="currentColor" width="16" height="16" viewBox="0 0 24 24">
        <path :d="mdiWindowClose"></path>
      </svg>
      <span>Sluiten</span>
    </button>
  </section>
</template>
