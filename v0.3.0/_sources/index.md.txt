# NL-Wallet Documentation

The intention of this documentation is to provide insight into how the wallet
app and its dependent infrastructure works. We aim to document the components
used, the communication that happens between the different layers of the app
and various related flow-diagrams for various things you can do with the app.

This file mainly serves as an introduction and a table of contents, the actual
(sequence) diagrams and other docs can be found in the linked files.

Note that the flows (for now) will mainly serve as a guide as the project is
still rapidly evolving. As such they should be updated when they are implemented
or the implementation changes.

```{toctree}
:maxdepth: 1
:caption: Contents:
relying-party
spec-references
generic_issuance/technical_attestation_schema
spec-profiles/sd-jwt-vc
api/README
diagrams/app_startup
diagrams/disclosure
diagrams/disclosure_based_issuance
diagrams/issuance
diagrams/overview_components
diagrams/pin_validation
diagrams/wallet_creation
lokalise
chores/update-rust
wow/definition-of-done
wow/merge-requests
wow/release-howto
templates/release-notes
release-notes/releases
```

## Participants

A brief summary of the participants used in diagrams.

| Participant      | Description                                                                       |
|------------------|-----------------------------------------------------------------------------------|
| user             | The end-user that downloads, installs and uses the application.                   |
| platform         | The platform specific native layer, referring to iOS (Swift) or Android (Kotlin). |
| platform_browser | The platform specific browser, user configurable.                                 |
| wallet_app       | The Flutter application code (i.e. Dart).                                         |
| wallet_core      | The core business logic, built using Rust.                                        |
| wallet_provider  | The backend, its business logic is often kept out of scope for now.               |
| digid_connector  | Service used to abstract the DigiD SAML protocol.                                 |
| pid_issuer       | The service that will provide the PID to the wallet.                              |
| digid            | DigiD service, used to authenticate users and retrieve their BSN.                 |

## Components

A global overview of how the different components interact with each other.

See: [overview_components.md](./diagrams/overview_components.md)

## Flows

Various aspects of using the app can be expressed in flows. When a user starts
the app, creates a wallet and starts using it, approximately the following
flows are involved:

  1. App startup
  2. Create wallet
  3. Pin validation
  4. Personalize wallet
  5. Disclosure

The following sub-sections describe those flows and link to flow diagrams.

### App Startup

Details what happens when the app is started, mainly focusing on the
initialisation of the `wallet_core`.

See: [app_startup.md](./diagrams/app_startup.md)

### Create Wallet

Details what happens when the wallet is created, this includes registering with
the `wallet_provider`.

See: [wallet_creation.md](./diagrams/wallet_creation.md)

### Pin Validation

Details what happens when the user enters a pin before registering with the
`wallet_provier` (local validation) and what happens when the user tries to
unlock the wallet after registration.

See: [pin_validation.md](./diagrams/pin_validation.md)

### Issuance (Personalise Wallet)

Details what happens after registering with `wallet_provider`, when it's time to
fetch and add some attestations.

See: [issuance.md](./diagrams/issuance.md)

Details what happens when receiving attestations from an issuer, based on a disclosure.

See: [disclosure_based_issuance.md](./diagrams/disclosure_based_issuance.md)

## API Documentation

We have API documentation in OpenAPI v3 format:

See: [README.md](./api/README.md)

## Relying Party Step-by-Step

A Relying Party (also known as a verifier, a party that needs to verify
attestations presented by the wallet). A relying party needs to have a general
idea of what has to be done to integrate with the wallet environment.

See: [relying-party.md](./relying-party.md)

For details on the disclosure flow, see: [disclosure.md](./diagrams/disclosure.md)

## Localisation

We use a commercial service called "Lokalise" to manage translations.

See: [lokalise.md](./lokalise.md)

## Release notes

We have a release note template: [release-notes.md](./templates/release-notes.md)

We have release notes for various releases [here](./release-notes/releases.md).

## Way of working

We have documents about how we do certain things. For now just one:

See: [definition-of-done.md](./wow/definition-of-done.md)

## Chores

We have various re-occuring chores, but currently just one documented:

See: [update-rust.md](./chores/update-rust.md)

## Implementation profiles for external formal specifications.



| Specification      | Link to profile description                                           |
|--------------------|-----------------------------------------------------------------------|
| SD-JWT VC          | [NL-Wallet profile for SD-JWT VC](./spec-profiles/sd-jwt-vc.md)       | 

