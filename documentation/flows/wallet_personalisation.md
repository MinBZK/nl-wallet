# Wallet personalisation

This document describes the steps of personalising the wallet. This includes login in with [DigiD](#41-personalisation-digid-login) and retrieving the [attestations](#pid-issuance).

## Personalisation (DigiD Login)

During personalisation the user will login with 'DigiD Hoog' to validate its identity. This diagram describes the login flow that leads to the PID.

The actual PID issuance is detailed separately in the diagram below. Also note that in this diagram we assume the `digid_connector` provides an encrypted BSN that only the server can decrypt.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant digid_connector
    title DigiD Login [4.1]

    user->>wallet_app: click login with digid
    wallet_app->>+wallet_core: getAuthUrl()
    wallet_core->>+digid_connector: getAuthUrl()
    opt server error
        digid_connector->>wallet_core: server error
        wallet_core->>wallet_app: error
        wallet_app->>user: show server error
    end
    digid_connector-->>-wallet_core: authUrl
    wallet_core-->>-wallet_app: authUrl
    wallet_app->>+platform_browser: launchUrl(authUrl)
    platform_browser->>platform: redirect to digid app (deeplink)
    platform->>digid: open digid app
    activate digid
        digid->>digid: login
        digid-->>+digid_connector: digid login result
    deactivate digid
    digid_connector-->>-platform_browser: digid login result
    opt digid login failed
        platform_browser->>platform: redirect to app (deeplink)
        platform->>wallet_app: open wallet_app<br>with error
        wallet_app->>user: render login with digid failed
    end
    platform_browser->>-platform: redirect back to app with result
    platform->>wallet_app: open app with digid deeplink
    wallet_app->>wallet_core: processUri(digidDeeplink)
    activate wallet_core
    wallet_core-->>wallet_app: notify is digid login uri
    opt 
        wallet_app-->>wallet_app: open auth screen in correct state
        Note over wallet_app, wallet_app: helpful if app is killed while backgrounded
    end
    wallet_core->>+digid_connector: getAccessToken(authorizationCode)
    digid_connector->>+digid: resolveSamlArtifact()
    digid-->>-digid_connector: bsn 
    digid_connector->>digid_connector: store bsn as userinfo
    opt server error
        note over digid_connector, user: same as error above
    end
    digid_connector-->>-wallet_core: accessToken
    wallet_core->>+pid_issuer: issuePid(accessToken)
    pid_issuer->>+digid_connector: getUserInfo(accessToken)
    digid_connector->>digid_connector: Lookup bsn, encrypt
    digid_connector-->>-pid_issuer: jwe(bsn)
    pid_issuer->>pid_issuer: validate & decrypt jwe(bsn)
    pid_issuer->>pid_issuer: lookup attributes using bsn
    pid_issuer-->>-wallet_core: PID issuance
    note over wallet_core: See PID Issuance [4.2]
    wallet_core-->>wallet_app: PID issue success
    wallet_app-->>user: display result
    deactivate wallet_core
```

## PID issuance

High level overview of PID issuance using mdoc.

The protocol supports issuance of multiple copies of an mdoc as well as multiple distinct mdocs simultaneously, but for simplicity the diagram assumes issuance of a single mdoc.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant wallet_app
    participant wallet_core
    participant pid_issuer
    participant wallet_provider
    title PID Issuance [4.2]

    pid_issuer->>wallet_core: ServiceEngagement(url)
    wallet_core->>+pid_issuer: StartProvisioningMessage
    pid_issuer-->>-wallet_core: ReadyToProvisionMessage(sessionID)
    wallet_core->>+pid_issuer: StartIssuingMessage(sessionID)
    pid_issuer-->>-wallet_core: RequestKeyGenerationMessage(doctype, attributes, nonce)
    note over wallet_core,pid_issuer: attributes are not yet signed
    wallet_core->>wallet_app: doctype, attributes
    wallet_app->>user: show doctype, attributes. Proceed?
    break user refuses
        user->>wallet_app: no
        wallet_app->>wallet_core: no
        wallet_core->>+pid_issuer: RequestEndSessionMessage(sessionID)
        pid_issuer-->>-wallet_core: EndSessionMessage
    end
    user->>wallet_app: ok
    wallet_app->>wallet_core: ok
    wallet_core->>+wallet_provider: requestChallenge(wp_certificate)
    wallet_provider-->>-wallet_core: challenge
    wallet_core->>+wallet_provider: generateKeyAndSign(wp_certificate, pin_sig(challenge), hw_sig(challenge), nonce)
    wallet_provider->>wallet_provider: verify signatures over challenge
    note left of wallet_provider: user is authenticated
    wallet_provider->>wallet_provider: generate public/private key<br/>store private key<br/>sign nonce w/ private key
    wallet_provider->>-wallet_core: public key, key id, signature
    wallet_core->>+pid_issuer: KeyGenerationResponseMessage(sessionID, public key, signature)
    pid_issuer->>pid_issuer: verify public key, signature over nonce
    pid_issuer->>pid_issuer: sign PID mdoc <br/> containing doctype, attributes, public key
    pid_issuer-->>-wallet_core: PID mdoc
    wallet_core->>wallet_core: store key id, PID mdoc
```
