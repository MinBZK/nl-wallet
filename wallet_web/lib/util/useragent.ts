import { type InjectionKey } from "vue"

export const isMobileKey = Symbol() as InjectionKey<boolean>

/// Decide whether a desktop is used. The implementation is loosely based on
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Browser_detection_using_the_user_agent#mobile_tablet_or_desktop
export const isDesktop = (userAgent: string): boolean => {
  const mobile = /Mobi/.test(userAgent)
  const android = /Android/i.test(userAgent)
  const iPhone = /iPhone/.test(userAgent)
  const iPad = /iPad/.test(userAgent) || (userAgent.includes("Macintosh") && navigator.maxTouchPoints > 1)

  return !(mobile || android || iPhone || iPad)
}
