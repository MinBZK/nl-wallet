# Overview

The intention of this readme is to provide insight into the communication that happens between the different layers of the app. This file mainly serves as an introduction and a table of contents, the actual (sequence) diagrams can be found in the referenced files.

Note that these flows (for now) will mainly serve as a guide as the project is still rapidly evolving. As such they should be updated when they are implemented or the implementation changes.

## Participants

A brief summary of the participants used in diagrams.

| Participant | Description                                                                                                         |
|---|-------------------------------------------------------------------------------------------------------------------------------|
| user | The end-user that downloads, installs and uses the application.                                                            |
| platform | The platform specific native layer, referring to iOS (Swift) or Android (Kotlin), depending on the host platform.      |
| platform_browser | The platform specific browser, user configurable.                                                              |
| wallet_app | The Flutter application code (i.e. Dart).                                                                            |
| wallet_core | The core business logic, built using Rust.                                                                          |
| wallet_provider | The backend, its business logic is often kept out of scope for now.                                             |
| digid_connector | Service used to abstract the DigiD SAML protocol.                                                               |
| pid_issuer | The service that will provide the PID to the wallet.                                                                 |
| digid | DigiD service, used to authenticate users and retrieve their BSN.                                                         |


## Components overview

A global overview of how the different components interact with each other. For a more detailed view, see the flows below.

See [components.md](./components.md)

## 1. App Startup

Details what happens when the app is started, mainly focusing on the initialisation of the `wallet_core`.

See [app_startup.md](./flows/app_startup.md)

## 2. Create Wallet

Details what happens when the wallet is created, this includes registering with the `wallet_provider`.

See [wallet_creation.md](./flows/wallet_creation.md)

## 3. Pin Validation

Details what happens when the user enters a pin before registering with the `wallet_provier` (local validation) and what happens when the user tries to unlock the wallet after registration.

See [pin_validation.md](./flows/pin_validation.md)

## 4. Personalise Wallet

Details what happens after registering with `wallet_provider`, when it's time to fetch and add some attestations.

See [openid.md](./flows/openid.md)

## 5. Disclosure flow (verification)

Details what happens when disclosing attestations.

See [disclosure.md](./flows/disclosure.md)
