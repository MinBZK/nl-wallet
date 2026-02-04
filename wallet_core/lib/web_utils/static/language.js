const lang_toggle = document.getElementById("lang_toggle")
const lang_toggle_label = document.querySelector("label[for='lang_toggle']")

lang_toggle_label.addEventListener("keydown", (event) => {
  console.log(event.keyCode)
  let isSpaceOrEnter = event.keyCode === 13 || event.keyCode === 32
  if (isSpaceOrEnter) {
    event.preventDefault()
    lang_toggle.checked = !lang_toggle.checked
  }
})
