// Progressively enhance displaying the revocation datetime in the browser's locale
;(function () {
  document.addEventListener("DOMContentLoaded", function () {
    const el = document.getElementById("success_message")

    if (!el) return

    const iso = el.dataset.revokedAt
    const lang = el.dataset.language // "nl" or "en"
    const template = el.dataset.template

    if (!iso || !template) return
    const d = new Date(iso)
    if (Number.isNaN(d.getTime())) return

    const locale = lang === "nl" ? "nl" : "en"

    const dateStr = new Intl.DateTimeFormat(locale, {
      year: "numeric",
      month: "long",
      day: "2-digit",
    }).format(d)

    const timeStr = new Intl.DateTimeFormat(locale, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(d)

    el.textContent = template.replace("{date}", dateStr).replace("{time}", timeStr)
  })
})()
