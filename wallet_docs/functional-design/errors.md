# Errors

This page provides describes the different types of errors a user can encounter when using NL Wallet. For each error type, this page provides:

- A short description of possible causes (most likely causes listed first).
- Suggested resolutions (in the advised order).

The use case descriptions in the functional design specify when such errors can be encountered. Note that the software and logging use more specific error types.

---

## List of User Facing Error Types

- [No Internet](#no-internet)
- [Server Unreachable](#server-unreachable)
- [Session Expired](#session-expired)
- [Device Insecure](#device-insecure)
- [Generic Error](#generic-error)
- [Known Issuer Error](#known-issuer-error)
- [Unknown Issuer Error](#unknown-issuer-error)
- [Known Verifier Error](#known-verifier-error)
- [Unknown Verifier Error](#unknown-verifier-error)

---

## No Internet

The mobile device does not seem to have an active (or stable) internet connection. May occur when the app contacts its own backend services or third party backend services.

**Possible causes:**

- Device not connected to the internet.
- Airplane mode enabled.
- Poor network coverage.

**Possible resolutions:**

1. Check internet connection (Wi-Fi or mobile data).
2. Retry after reconnecting.
3. If the problem persists, contact the service desk.

---

## Server Unreachable

The mobile device _does_ seem to have an internet connection but the NL Wallet backend services cannot be reached. May occur when the app contacts its own backend services.

**Possible causes:**

- Temporary server outage.
- Firewall or DNS issue.
- Network instability.

**Possible resolutions:**

1. Retry after a few moments.
2. Check if other services/websites are reachable.
3. If the issue persists, contact the service desk.

---

## Session Expired

A session between the app and a verifying or issuing organization has expired. Likely to occur when the organization sets very limited time constraints, or users get distracted in the process.

**Possible causes:**

- User session timed out due to inactivity.
- Token expired.

**Possible resolutions:**

1. Restart the flow from the beginning.
2. If the issue persists, contact the service desk of the issuing or verifying organization.

---

## Device Insecure

The user's device does not meet the security requirements of the NL Wallet. May occur during activation or later, when requirements change.

**Possible causes:**

- Device does not meet minimum requirements (OS version, security features).
- Unsupported device type.
- The device is rooted.

**Possible resolutions:**

1. Update device software.
2. Use another compatible device.
3. If the issue persists, contact the service desk.

---

## Generic Error

Either the app or backend services encounter an unexpected error. May occur at any time.

**Possible causes:**

- Programming error
- Configuration error

**Possible resolutions:**

1. Retry the action.
2. Restart the application.
3. If the issue persists, contact the service desk.

---

## Known Issuer Error

An error occurred while issuing card(s) which was likely caused by the issuing organization. The error occurred _after_ the app has identified the issuing organization (hence 'known'), so it can display and log its identity information.

**Possible causes:**

- The issuer is unreachable.
- The issuer certificates are invalid, expired or revoked.
- The issuer did not properly implement their part of the interface.
- The issuer has made a configuration error.

**Possible resolutions:**

1. Retry later.
2. If the issue persists, contact the service desk of the issuing organization.

---

## Unknown Issuer Error

An error occurred while issuing card(s) which was likely caused by the issuing organization. The error occurred _before_ the app has identified the issuing organization (hence 'unknown'), so it cannot display or log its identity information. The user should know who the issuing organization is.

**Possible causes:**

- The issuer is unreachable.
- The issuer certificates are invalid, expired or revoked.
- The issuer did not properly implement their part of the interface.
- The issuer has made a configuration error.

**Possible resolutions:**

1. Retry later.
2. If the issue persists, contact the service desk of the issuing organization.

---

## Known Verifier Error

An error occurred while verifying card(s) which was likely caused by the verifying organization. The error occurred _after_ the app has identified the verifying organization (hence 'known'), so it can display and log its identity information.

**Possible causes:**

- The verifier is unreachable.
- The verifier certificates are invalid, expired or revoked.
- The verifier did not properly implement their part of the interface.
- The verifier has made a configuration error.

**Possible resolutions:**

1. Retry later.
2. If the issue persists, contact the service desk of the verifying organization.

---

## Unknown Verifier Error

An error occurred while verifying card(s) which was likely caused by the verifying organization. The error occurred _before_ the app has identified the verifying organization (hence 'unknown'), so it cannot display or log its identity information. The user should know who the verifying organization is.

**Possible causes:**

- The verifier is unreachable.
- The verifier certificates are invalid, expired or revoked.
- The verifier did not properly implement their part of the interface.
- The verifier has made a configuration error.

**Possible resolutions:**

1. Retry later.
2. If the issue persists, contact the service desk of the verifying organization.
