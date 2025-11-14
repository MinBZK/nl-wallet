import { type InjectionKey, inject } from "vue"
import enTranslations from "../l10n/en.json"
import nlTranslations from "../l10n/nl.json"

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
export type Word = keyof typeof enTranslations

const dictionary: Record<Language, Record<Word, string>> = {
  en: enTranslations,
  nl: nlTranslations,
}
