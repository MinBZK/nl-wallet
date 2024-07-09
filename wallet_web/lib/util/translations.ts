import { type InjectionKey, inject } from "vue"

export const translationsKey = Symbol("TRANSLATIONS") as InjectionKey<(input: Word) => string>

// from https://logaretm.com/blog/making-the-most-out-of-vuejs-injections/#requiring-injections
export const injectStrict: <T>(key: InjectionKey<T>, fallback?: T) => T = (key, fallback) => {
  const resolved = inject(key, fallback)
  if (!resolved) {
    throw new Error(`Could not resolve ${key.description}`)
  }

  return resolved
}

export const translations: (lang: Language) => (input: Word) => string = (lang) => {
  const words = dictionary[lang]
  return (input) => words[input]
}

export type Language = "nl" | "en"
export type Word =
  | "wallet_button_text"
  | "modal_header"
  | "need_help"
  | "stop"
  | "retry"
  | "close"
  | "device_choice_title"
  | "device_choice_same_device"
  | "device_choice_cross_device"
  | "expired_title"
  | "expired_body"
  | "failed_title"
  | "failed_body"
  | "cancelled_title"
  | "cancelled_body"
  | "help_title"
  | "help_to_website"
  | "in_progress_title"
  | "loading_title"
  | "loading_body"
  | "qr_code_title"
  | "qr_code_label"
  | "success_title"
  | "success_body"

const dictionary: Record<Language, Record<Word, string>> = {
  en: {
    wallet_button_text: "Login with NL Wallet",
    modal_header: "NL Wallet",
    need_help: "Need help?",
    stop: "Stop",
    retry: "Try again",
    close: "Close",
    device_choice_title: "Which device is your NL Wallet is installed?",
    device_choice_same_device: "On this device",
    device_choice_cross_device: "On another device",
    expired_title: "Sorry, time is over",
    expired_body:
      "This action has been stopped because too much time has passed. This happens to keep your data safe. Please try again.",
    failed_title: "Sorry, something went wrong",
    failed_body: "This action was unsuccessful. This may have several reasons. Please try again.",
    cancelled_title: "Stopped",
    cancelled_body:
      "Because you have stopped, no data has been shared with the Municipality of Amsterdam",
    help_title: "No NL Wallet App yet? Or need help?",
    help_to_website: "To NL Wallet website",
    in_progress_title: "Follow the steps in your NL Wallet app",
    loading_title: "Please wait",
    loading_body: "Your request is being retrieved",
    qr_code_title: "Scan the QR code with your NL Wallet app",
    qr_code_label: "QR code",
    success_title: "Success!",
    success_body: "Close this page and continue in the new opened tab.",
  },
  nl: {
    wallet_button_text: "Inloggen met NL Wallet",
    modal_header: "NL Wallet",
    need_help: "Hulp nodig?",
    stop: "Stoppen",
    retry: "Probeer opnieuw",
    close: "Sluiten",
    device_choice_title: "Op welk apparaat staat je NL Wallet app?",
    device_choice_same_device: "Op dit apparaat",
    device_choice_cross_device: "Op een ander apparaat",
    expired_title: "Sorry, de tijd is voorbij",
    expired_body:
      "Deze actie is gestopt omdat er teveel tijd voorbij is gegaan. Dit is bedoeld om je gegevens veilig te houden. Probeer het opnieuw.",
    failed_title: "Sorry, er gaat iets mis",
    failed_body:
      "Deze actie is niet gelukt. Dit kan verschillende redenen hebben. Probeer het opnieuw.",
    cancelled_title: "Gestopt",
    cancelled_body: "Omdat je bent gestopt zijn er geen gegevens gedeeld.",
    help_title: "Nog geen NL Wallet app? Of hulp nodig?",
    help_to_website: "Naar NL Wallet website",
    in_progress_title: "Volg de stappen in de NL Wallet app",
    loading_title: "Even geduld",
    loading_body: "De gegevens worden opgehaald",
    qr_code_title: "Scan de QR-code met je NL Wallet app",
    qr_code_label: "QR code",
    success_title: "Gelukt!",
    success_body: "Sluit deze pagina en ga verder in het nieuw geopende tabblad.",
  },
}
