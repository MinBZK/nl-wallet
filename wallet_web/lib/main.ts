import { defineCustomElement } from "vue"
import WalletButton from "./WalletButton.ce.vue"

const NLWalletButton = defineCustomElement(WalletButton)
customElements.define("nl-wallet-button", NLWalletButton)

declare module "vue" {
  export interface GlobalComponents {
    "nl-wallet-button": typeof NLWalletButton,
  }
}
