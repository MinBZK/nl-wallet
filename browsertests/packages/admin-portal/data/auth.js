const AUTH_DIR = "playwright/.auth"

/**
 * Playwright storage state per user profile, written by `tests/auth.setup.js` before the browser
 * projects run. The portal keeps its session in a single `sid` cookie, so restoring the state is
 * enough to start a test logged in.
 *
 * Note that the session has an absolute expiry (`admin_portal.session_ttl_in_ms`, 15 minutes by
 * default) that requests do not extend, so a run lasting longer than that outlives its state.
 */
export function authFile(profile) {
  return `${AUTH_DIR}/${profile}.json`
}
