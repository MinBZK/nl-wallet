const wallet_buttons = document.getElementsByTagName("nl-wallet-button")
for (const button of wallet_buttons) {
  const return_url_callback = (e) => {
    if (e.detail && e.detail.length > 1) {
      const session_token = e.detail[0]
      const session_type = e.detail[1]
      const usecase = button.attributes.getNamedItem("usecase").value

      // this only works for cross_device without a configured return URL
      if (session_type === "cross_device") {
        window.location.assign("../" + usecase + "/return?session_token=" + session_token)
      }
    }
  }

  button.addEventListener("success", return_url_callback, false)
  button.addEventListener("failed", return_url_callback, false)
}
