const lang_toggle = document.getElementById("lang_toggle")
const lang_toggle_label = document.querySelector("label[for='lang_toggle']")

lang_toggle_label.addEventListener("keydown", (event) => {
  const enterKeyCode = 13
  const spaceKeyCode = 32

  let isSpaceOrEnter = event.keyCode === enterKeyCode || event.keyCode === spaceKeyCode
  if (isSpaceOrEnter) {
    event.preventDefault()
    lang_toggle.checked = !lang_toggle.checked
  }
})
