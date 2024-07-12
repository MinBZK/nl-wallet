const wallet_buttons = document.getElementsByTagName("nl-wallet-button");
for (let button of wallet_buttons) {
    button.addEventListener(
        "success",
        (e) => {
            if (e.detail && e.detail.length > 1) {
                const session_token = e.detail[0];
                const session_type = e.detail[1];
                const usecase = button.attributes.getNamedItem("usecase").value;

                if (session_type === "cross_device") {
                    window.location.assign("../" + usecase + "/return?session_token=" + session_token);
                }
            }
        },
        false,
    );
}
