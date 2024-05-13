<script setup lang="ts">
import { faArrowUpRightFromSquare } from "@fortawesome/free-solid-svg-icons/faArrowUpRightFromSquare"
import { faXmark } from "@fortawesome/free-solid-svg-icons/faXmark"
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome"
import { ref } from "vue"
import "./style.css"

export interface Props {
  text?: string
  classes?: string
  style?: string
}

withDefaults(defineProps<Props>(), {
  text: (): string => "Inloggen met NL Wallet"
})

const emit = defineEmits(["close"])

const isVisible = ref(false)

function show() {
  isVisible.value = true
}

function hide() {
  isVisible.value = false
}

function close() {
  hide()
  emit("close", { success: "42" })
}
</script>

<template>
  <button part="button" type="button" class="default" :style="style" @click="show">{{ text }}</button>

  <aside
    class="modal reset"
    :class="{ hidden: !isVisible, visible: isVisible}">

    <h1>NL Wallet</h1>
    <h2>Scan de QR-code met je NL Wallet app</h2>

    <section class="qr">Icon</section>

    <section class="help">
      <div>Nog geen NL Wallet app? Of hulp nodig?</div>
      <div>
        <a href="/" class="link">
          <FontAwesomeIcon :icon="faArrowUpRightFromSquare" class="xs"></FontAwesomeIcon>
          <span>Naar NL Wallet website</span>
        </a>
      </div>
    </section>

    <section class="footer">
      <button type="button" class="link" @click="close">
        <FontAwesomeIcon :icon="faXmark" class="xs"></FontAwesomeIcon>
        <span>Sluiten</span>
      </button>
    </section>
  </aside>

  <div :class="{ bg: isVisible}"></div>
</template>

<style>
@import "style.css";

.xs {
  width: 0.8rem;
}

.modal *:not(svg, path) {
  all: unset;
}

.modal div {
  display: block;
}

@-webkit-keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.hidden {
  display: none;
  opacity: 0;
}

.visible {
  opacity: 1;
  display: block;
  animation: fadeIn 300ms;
}

.bg {
  position: fixed;
  top: 0;
  left: 0;
  z-index: 1040;
  width: 100vw;
  height: 100vh;
  background-color: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(1px);
}

.default {
  background-color: var(--blue-color);
  border: none;
  border-radius: 5px;
  color: white;
  cursor: pointer;
  padding: 10px 12px 10px 12px;
}

.default:hover {
  background-color: var(--bluer-color);
}

.modal .link {
  color: var(--blue-color);
  cursor: pointer;
  display: flex;
  gap: 0.5rem;
}

.modal {
  background-color: white;
  border-radius: 0.375rem;
  position: fixed;
  box-shadow: black 0 0 0 0, black 0 0 0 0, rgba(0, 0, 0, 0.4) 0 25px 50px -12px;
  transform: translate(-50%, -30%);
  top: 30%;
  left: 50%;
  width: 50%;
  z-index: 1050;
}

.modal h1 {
  display: flex;
  justify-content: center;
  border-bottom: 1px solid lightgrey;
  font-size: 2rem;
  margin-top: 10px;
  padding: 20px 0;
}

.modal h2 {
  display: flex;
  justify-content: center;
  margin-top: 20px;
  font-size: 1.5rem;
}

section.qr {
  display: flex;
  justify-content: center;
  padding: 32px 0;
}

section.help {
  font-size: 0.8rem;
}

section.help div {
  display: flex;
  justify-content: center;
  padding-top: 15px;
}

section.footer {
  display: flex;
  justify-content: center;
  border-top: 1px solid lightgrey;
  margin-top: 20px;
  padding: 20px 0;
}
</style>
