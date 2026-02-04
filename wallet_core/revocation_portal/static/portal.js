export const ALLOWED_REGEX = /[^0-9A-HJKMNP-TV-Z]/g

/**
 * Normalizes and formats a raw deletion code input string.
 * - Converts to uppercase
 * - Replaces confusable characters (I/L → 1, O → 0)
 * - Strips invalid characters
 * - Inserts hyphens every 4 characters
 * - Adds a trailing hyphen at group boundaries (for live typing UX)
 */
export function formatDeletionCode(value) {
  let val = value.toUpperCase()
  val = val.replace(/[IL]/g, "1").replace(/O/g, "0")
  const rawValue = val.replace(ALLOWED_REGEX, "")
  const parts = rawValue.match(/.{1,4}/g)
  let formatted = parts ? parts.join("-") : rawValue

  const addTrailingHyphen = rawValue.length > 0 && rawValue.length < 18 && rawValue.length % 4 === 0
  if (addTrailingHyphen) {
    formatted += "-"
  }

  return { formatted, rawValue, addTrailingHyphen }
}

/**
 * Calculates the correct cursor position in the formatted string,
 * preserving the user's position relative to raw characters.
 */
export function calculateCursorPosition(oldVal, oldPos, formatted, rawValue, addTrailingHyphen) {
  const rawPosBefore = oldVal.substring(0, oldPos).replace(/-/g, "").length
  const rawPosInNewValue = Math.min(rawPosBefore, rawValue.length)
  let newPos = rawPosInNewValue + Math.floor(rawPosInNewValue / 4)

  if (addTrailingHyphen && rawPosBefore >= rawValue.length) {
    newPos = formatted.length
  }

  return newPos
}

/**
 * Returns a validation key for the raw (hyphen-stripped) deletion code.
 * - 'required'       → empty input
 * - 'invalid_length' → not exactly 18 characters
 * - null             → valid
 */
export function validateDeletionCode(rawValue) {
  if (rawValue.length === 0) return "required"
  if (rawValue.length !== 18) return "invalid_length"
  return null
}
