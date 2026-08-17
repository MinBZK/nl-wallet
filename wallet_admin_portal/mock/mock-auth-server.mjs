#!/usr/bin/env node
// Note: This is a temporary implementation (mostly generated) and can be
// removed once the wallet_provider implements the authentication APIs.
//
// Admin Portal WalletBackend (BFF) mock, authenticating against a real
// KeyCloak instance for local testing. A local KeyCloak instance can be
// started using "../scripts/start-devenv.sh kc"
//
// Runs a single HTTP server:
//   - WalletBackend on WALLET_PORT (default 3000)
//
// The Vue dev server (Vite) should proxy `/api` and `/auth` to WalletBackend
// so that, from the browser's point of view, the SPA and WalletBackend share
// one origin and the session cookie "just works" — see vite.config.ts.
//
// Point KEYCLOAK_URL/KEYCLOAK_REALM/KEYCLOAK_CLIENT_ID at a running KeyCloak
// instance. The client there must be a public client with PKCE (S256)
// enabled and must allow REDIRECT_URI (printed on startup) as a redirect URI.
//
// Env vars: WALLET_PORT, KEYCLOAK_URL, KEYCLOAK_REALM, KEYCLOAK_CLIENT_ID,
// FRONTEND_ORIGIN, FRONTEND_ENTRY_PATH, FRONTEND_ERROR_PATH, SESSION_TTL_MS.

import { createServer } from 'node:http'
import { randomBytes, createHash, createPublicKey, verify as cryptoVerify } from 'node:crypto'

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

const WALLET_PORT = Number(process.env.WALLET_PORT ?? 3000)
const FRONTEND_ORIGIN = process.env.FRONTEND_ORIGIN ?? 'http://localhost:5173'
const FRONTEND_ENTRY_PATH = process.env.FRONTEND_ENTRY_PATH ?? '/'
const FRONTEND_ERROR_PATH = process.env.FRONTEND_ERROR_PATH ?? '/error'
const SESSION_TTL_MS = Number(process.env.SESSION_TTL_MS ?? 15 * 60 * 1000)
const TEMP_SESSION_TTL_MS = 10 * 60 * 1000

const KEYCLOAK_URL = process.env.KEYCLOAK_URL ?? 'http://localhost:11080'
const KEYCLOAK_REALM = process.env.KEYCLOAK_REALM ?? 'nl-wallet'
const CLIENT_ID = process.env.KEYCLOAK_CLIENT_ID ?? 'wallet-backend'

const ISSUER = `${KEYCLOAK_URL}/realms/${KEYCLOAK_REALM}`
const AUTHORIZATION_ENDPOINT = `${ISSUER}/protocol/openid-connect/auth`
const TOKEN_ENDPOINT = `${ISSUER}/protocol/openid-connect/token`
const JWKS_URI = `${ISSUER}/protocol/openid-connect/certs`
const END_SESSION_ENDPOINT = `${ISSUER}/protocol/openid-connect/logout`
// KeyCloak's "wallet-backend" client only accepts registered redirect URIs.
// This mock serves plain HTTP directly on WALLET_PORT, so that exact URI
// (http, not https) must be added to the client's redirectUris.
const REDIRECT_URI = `http://localhost:${WALLET_PORT}/auth/callback`

// ---------------------------------------------------------------------------
// In-memory state
// ---------------------------------------------------------------------------

/** state -> { state, nonce, codeVerifier, createdAt } */
const tempSessions = new Map()
/** sessionId -> { sub, displayName, roles, exp } */
const sessions = new Map()

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const base64url = (buf) => buf.toString('base64').replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')

const base64urlDecode = (str) => Buffer.from(str.replaceAll('-', '+').replaceAll('_', '/'), 'base64')

const randomToken = (bytes = 32) => base64url(randomBytes(bytes))

const sha256 = (input) => createHash('sha256').update(input).digest()

// KeyCloak signs id_tokens with RS256. Fetch its JWKS once and cache it —
// real clients would refresh on a kid miss (overkill for local dev/test process).
let jwksCache = null

async function getJwks() {
  if (!jwksCache) {
    const res = await fetch(JWKS_URI)
    if (!res.ok) throw new Error(`failed to fetch JWKS from ${JWKS_URI}: ${res.status}`)
    jwksCache = await res.json()
  }
  return jwksCache
}

async function verifyIdToken(token) {
  const parts = token.split('.')
  if (parts.length !== 3) return null
  const [headerPart, payloadPart, signaturePart] = parts

  let header, payload
  try {
    header = JSON.parse(base64urlDecode(headerPart).toString('utf8'))
    payload = JSON.parse(base64urlDecode(payloadPart).toString('utf8'))
  } catch {
    return null
  }

  const jwks = await getJwks()
  const jwk = jwks.keys.find((key) => key.kid === header.kid && (key.use === 'sig' || !key.use))
  if (!jwk) return null

  const publicKey = createPublicKey({ key: jwk, format: 'jwk' })
  const signingInput = Buffer.from(`${headerPart}.${payloadPart}`)
  const signature = base64urlDecode(signaturePart)
  const valid = cryptoVerify('RSA-SHA256', signingInput, publicKey, signature)

  return valid ? payload : null
}

function parseCookies(req) {
  const header = req.headers.cookie
  if (!header) return {}
  return Object.fromEntries(
    header.split(';').map((pair) => {
      const idx = pair.indexOf('=')
      return [pair.slice(0, idx).trim(), decodeURIComponent(pair.slice(idx + 1).trim())]
    }),
  )
}

function sendJson(res, status, body, extraHeaders = {}) {
  const json = JSON.stringify(body)
  res.writeHead(status, { 'Content-Type': 'application/json', 'Content-Length': Buffer.byteLength(json), ...extraHeaders })
  res.end(json)
}

function redirect(res, location, extraHeaders = {}) {
  res.writeHead(302, { Location: location, ...extraHeaders })
  res.end()
}

function log(component, ...args) {
  console.log(`[${component}]`, ...args)
}

function cleanupExpired() {
  const now = Date.now()
  for (const [key, value] of tempSessions) if (now - value.createdAt > TEMP_SESSION_TTL_MS) tempSessions.delete(key)
  for (const [key, value] of sessions) if (now > value.exp) sessions.delete(key)
}

// ---------------------------------------------------------------------------
// WalletBackend (BFF) — port WALLET_PORT
// ---------------------------------------------------------------------------

const walletServer = createServer(async (req, res) => {
  cleanupExpired()
  const url = new URL(req.url, `http://localhost:${WALLET_PORT}`)
  const { pathname, searchParams } = url

  try {
    if (req.method === 'GET' && pathname === '/auth/login') return handleLogin(req, res, searchParams)
    if (req.method === 'GET' && pathname === '/auth/callback') return await handleCallback(req, res, searchParams)
    if (req.method === 'GET' && pathname === '/auth/logout') return handleLogout(req, res)
    if (req.method === 'GET' && pathname === '/api/me') return handleMe(req, res)

    if (pathname === '/__debug/state') return handleDebugState(req, res)
    if (pathname === '/__debug/reset') return handleDebugReset(req, res)
    if (pathname === '/__debug/expire') return handleDebugExpire(req, res)

    sendJson(res, 404, { error: 'not_found' })
  } catch (err) {
    log('wallet', 'unhandled error', err)
    sendJson(res, 500, { error: 'internal_error' })
  }
})

function handleLogin(req, res, searchParams) {
  const state = randomToken(16)
  const nonce = randomToken(16)
  const codeVerifier = randomToken(32)
  const codeChallenge = base64url(sha256(codeVerifier))

  tempSessions.set(state, { state, nonce, codeVerifier, createdAt: Date.now() })

  const authUrl = new URL(AUTHORIZATION_ENDPOINT)
  authUrl.searchParams.set('response_type', 'code')
  authUrl.searchParams.set('client_id', CLIENT_ID)
  authUrl.searchParams.set('redirect_uri', REDIRECT_URI)
  authUrl.searchParams.set('scope', 'openid profile')
  authUrl.searchParams.set('state', state)
  authUrl.searchParams.set('nonce', nonce)
  authUrl.searchParams.set('code_challenge', codeChallenge)
  authUrl.searchParams.set('code_challenge_method', 'S256')

  const loginHint = searchParams.get('login_hint')
  if (loginHint) authUrl.searchParams.set('login_hint', loginHint)

  log('wallet', `/auth/login -> ${KEYCLOAK_URL} (realm ${KEYCLOAK_REALM})`)
  redirect(res, authUrl.toString())
}

async function handleCallback(req, res, searchParams) {
  const state = searchParams.get('state')
  const code = searchParams.get('code')
  const error = searchParams.get('error')

  const temp = state ? tempSessions.get(state) : undefined
  if (state) tempSessions.delete(state)

  if (!temp) {
    log('wallet', '/auth/callback: unknown or expired state')
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=invalid_state`)
  }

  if (error) {
    log('wallet', `/auth/callback: upstream error=${error}`)
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=${encodeURIComponent(error)}`)
  }

  if (!code) {
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=missing_code`)
  }

  let tokenResponse
  try {
    const body = new URLSearchParams({
      grant_type: 'authorization_code',
      code,
      redirect_uri: REDIRECT_URI,
      client_id: CLIENT_ID,
      code_verifier: temp.codeVerifier,
    })
    const tokenRes = await fetch(TOKEN_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body,
    })
    if (!tokenRes.ok) {
      log('wallet', `/auth/callback: token endpoint returned ${tokenRes.status}`)
      return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=token_exchange_failed`)
    }
    tokenResponse = await tokenRes.json()
  } catch (err) {
    log('wallet', '/auth/callback: token exchange failed', err)
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=token_exchange_failed`)
  }

  const idToken = await verifyIdToken(tokenResponse.id_token)
  const now = Math.floor(Date.now() / 1000)
  const valid =
    idToken &&
    idToken.iss === ISSUER &&
    idToken.aud === CLIENT_ID &&
    idToken.nonce === temp.nonce &&
    idToken.exp > now &&
    idToken.iat <= now

  if (!valid) {
    log('wallet', '/auth/callback: id_token validation failed')
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ERROR_PATH}?reason=invalid_token`)
  }

  // KeyCloak's "realm roles" mapper puts realm_access.roles on the access
  // token, not the id_token (see wallet_docs/development/keycloak-setup.md).
  const accessToken = await verifyIdToken(tokenResponse.access_token)
  const accessTokenValid = accessToken && accessToken.iss === ISSUER && accessToken.exp > now && accessToken.iat <= now
  if (!accessTokenValid) {
    log('wallet', '/auth/callback: access_token validation failed, proceeding without roles')
  }

  const sessionId = randomToken(32)
  sessions.set(sessionId, {
    sub: idToken.sub,
    displayName: idToken.name,
    roles: (accessTokenValid ? accessToken.realm_access?.roles : undefined) ?? [],
    idToken: tokenResponse.id_token,
    exp: Date.now() + SESSION_TTL_MS,
  })

  log('wallet', `/auth/callback: session created for ${idToken.sub}`)
  redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ENTRY_PATH}`, {
    'Set-Cookie': `sid=${sessionId}; HttpOnly; SameSite=Lax; Path=/; Max-Age=${Math.floor(SESSION_TTL_MS / 1000)}`,
  })
}

function handleLogout(req, res) {
  const { sid } = parseCookies(req)
  const session = sid ? sessions.get(sid) : undefined
  if (sid) sessions.delete(sid)

  const clearCookie = { 'Set-Cookie': 'sid=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0' }

  if (!session?.idToken) {
    log('wallet', '/auth/logout: no id_token on session, skipping KeyCloak end-session')
    return redirect(res, `${FRONTEND_ORIGIN}${FRONTEND_ENTRY_PATH}`, clearCookie)
  }

  const endSessionUrl = new URL(END_SESSION_ENDPOINT)
  endSessionUrl.searchParams.set('id_token_hint', session.idToken)
  endSessionUrl.searchParams.set('client_id', CLIENT_ID)
  endSessionUrl.searchParams.set('post_logout_redirect_uri', `${FRONTEND_ORIGIN}${FRONTEND_ENTRY_PATH}`)

  log('wallet', '/auth/logout -> KeyCloak end-session')
  redirect(res, endSessionUrl.toString(), clearCookie)
}

function handleMe(req, res) {
  const { sid } = parseCookies(req)
  const session = sid ? sessions.get(sid) : undefined

  if (!session || Date.now() > session.exp) {
    if (sid) sessions.delete(sid)
    sendJson(res, 401, { error: 'unauthorized' })
    return
  }

  // Sliding expiry: every authenticated call extends the session.
  session.exp = Date.now() + SESSION_TTL_MS

  sendJson(res, 200, {
    displayName: session.displayName,
    privileges: rolesToPrivileges(session.roles),
  })
}

// KeyCloak realm roles are named "privilege_<name>", remove this prefix.
const PRIVILEGE_ROLE_PREFIX = 'privilege_'

function rolesToPrivileges(roles) {
  return roles
    .filter((role) => role.startsWith(PRIVILEGE_ROLE_PREFIX))
    .map((role) => role.slice(PRIVILEGE_ROLE_PREFIX.length))
}

function handleDebugState(req, res) {
  sendJson(res, 200, {
    tempSessions: [...tempSessions.values()],
    sessions: [...sessions.entries()].map(([sessionId, v]) => ({ sessionId, ...v })),
  })
}

function handleDebugReset(req, res) {
  tempSessions.clear()
  sessions.clear()
  sendJson(res, 200, { ok: true })
}

function handleDebugExpire(req, res) {
  const { sid } = parseCookies(req)
  let expired = 0
  if (sid && sessions.has(sid)) {
    sessions.get(sid).exp = 0
    expired = 1
  } else {
    for (const session of sessions.values()) session.exp = 0
    expired = sessions.size
  }
  sendJson(res, 200, { ok: true, expired })
}

// ---------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------

walletServer.listen(WALLET_PORT, () => {
  log('wallet', `WalletBackend (BFF) listening on http://localhost:${WALLET_PORT}`)
  log('wallet', `KeyCloak issuer=${ISSUER} client_id=${CLIENT_ID}`)
  log('wallet', `redirect_uri=${REDIRECT_URI} — must be a registered redirect URI on the KeyCloak client`)
  log('wallet', `frontend origin=${FRONTEND_ORIGIN} entry=${FRONTEND_ENTRY_PATH} error=${FRONTEND_ERROR_PATH}`)
})
