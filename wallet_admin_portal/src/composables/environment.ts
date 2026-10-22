/**
 * Name of the environment the portal is deployed in, e.g. "Acceptatie".
 *
 * Extracted from the `data-environment` attribute of the app root element.
 * Returns `null` when it is missing or empty.
 */
export function useEnvironment(): { environment: string | null } {
  const environment = document.getElementById('app')?.dataset.environment?.trim()
  return { environment: environment || null }
}
