/** The login session's CSRF token as returned by the wallet_provider `/admin-portal/api/csrf-token` endpoint. */
export interface CsrfTokenResponse {
  csrfToken: string
}
