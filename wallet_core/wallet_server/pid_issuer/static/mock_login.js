// On the first card submission, disable every card so a slow POST doesn't invite repeat clicks, and
// reveal the loading overlay. The submitting form still navigates as usual. This covers both the
// preselectable cards and the custom-BSN card; the latter only fires `submit` once its input passes
// the browser's constraint validation, so an empty/invalid BSN won't trip the overlay.
;(function () {
  const forms = document.querySelectorAll(".grid form")
  const overlay = document.getElementById("overlay")
  forms.forEach(function (form) {
    form.addEventListener("submit", function () {
      forms.forEach(function (other) {
        other.querySelector("button").disabled = true
      })
      if (overlay) {
        overlay.hidden = false
      }
    })
  })
  function checkBsn(bsn) {
    if (!/^[0-9]{9}$/.test(bsn)) {
      return null
    }
    let sum = 0
    for (let i = 0; i < 8; i++) {
      sum += (9 - i) * parseInt(bsn[i])
    }
    sum -= parseInt(bsn[8])
    return sum % 11 === 0
  }
  document.getElementById("custom-bsn").addEventListener("change", function () {
    if (checkBsn(this.value) === false) {
      // Only set custom validity when the BSN consists of 9 digits but is incorrect
      // Let the browser's constraint validation handle the other cases
      this.setCustomValidity(this.dataset.invalidBsn)
    } else {
      this.setCustomValidity("")
    }
  })
})()
