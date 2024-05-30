import { computed, onMounted, onUnmounted, ref } from "vue"

export const useWindowsSize = () => {
  const windowWidth = ref(window.innerWidth)
  const windowHeight = ref(window.innerHeight)

  const onWidthChange = () => {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }
  onMounted(() => window.addEventListener("resize", onWidthChange))
  onUnmounted(() => window.removeEventListener("resize", onWidthChange))

  const width = computed(() => windowWidth.value)
  const height = computed(() => windowHeight.value)

  return { width, height }
}
