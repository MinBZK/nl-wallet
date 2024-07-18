import { isError, type ErrorType } from "@/models/state"

export const errorTypeOrDefault = (e: unknown): ErrorType => {
  if (isError(e)) {
    return e
  } else {
    return "failed"
  }
}
