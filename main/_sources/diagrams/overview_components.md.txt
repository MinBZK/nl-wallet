# Components overview

The diagram below shows a global overview of how the components interact with
each other. It contains some details about the application layer protocols and
technologies used, but leaves out exact (data) flow and detailed steps. Arrow
directions indicate which party initiated the interaction and not the flow of
data.

```{mermaid}
graph
    brp["`
        BRP
        _[Software system]_
        Contains the personal data of people
    `"]
    digid["`
        DigiD
        _[Software system]_
    `"]

    relying_party["`
        Relying Party
    `"]

    wallet_app("`
        Wallet App
        _[Container: Flutter/Rust]_
    `")

    wallet_app -- "`
        Provides key instructions
        _[JSON/HTTPS]_
    `"--> wallet_provider

    subgraph Wallet Provider
        wallet_provider["`
            Wallet Provider
            _[Container: Rust (wallet_provider)]_
        `"]
        wallet_provider_migrations["`
            Wallet Provider Migrations
            _[Container: Rust]_
        `"]
        postgres[("`
            Database
            _[Container: Postgres]_
        `")]
        hsm["`
            HSM
            _[Container: SoftHSMv2]_
        `"]

        wallet_provider -- "`
            Encrypts/decrypts/signs data
            [PKCS#11]
        `" --> hsm
        wallet_provider -- "`
            Reads from/writes to
            _[SQL/TCP]_
        `" --> postgres
        wallet_provider_migrations -- "`
            Manages schema
            _[SQL/TCP]_
        `" --> postgres
    end

    subgraph PID[PID Issuer]
        digid_connector["`
            DigiD connector
            _[Container: Python]_
        `"]
        pid_issuer["`
            PID Issuer
            _[Container: Rust (pid_issuer)]_
        `"]
        cache[("`
            Cache
            _[Container: Postgres]_
        `")]
        issuer_migrations["`
            Issuer Migrations
            _[Container: Rust]_
        `"]

        pid_issuer -- "`
            Gets BSN
            _[JSON/HTTPS]_
        `" --> digid_connector
        issuer_migrations -- "`
            Manages schema
            _[SQL/TCP]_
        `" --> cache
        pid_issuer -- "`
            Stores session
            _[SQL/TCP]_
        `" --> cache
    end

    subgraph RP[Relying Party]
        relying_party_server["`
            Relying Party Server
            _[Container: Rust (verification_server)]_
        `"]
        relying_party_migrations["`
            Relying Party Migrations
            _[Container: Rust]_
        `"]
        cache_rp[("`
            Cache
            _[Container: Postgres]_
        `")]
        relying_party_server -- "`
            Stores session
            _[SQL/TCP]_
        `"--> cache_rp
        relying_party_migrations -- "`
            Manages schema
            _[SQL/TCP]_
        `" --> cache_rp
    end

    wallet_app -- "`
        Authenticates user
        _[OIDC/HTTPS]_
    `" --> digid_connector
    wallet_app -- "`
        Gets PID issued
        _[OpenID4VCI/HTTPS]_
    `" --> pid_issuer
    wallet_app -- "`
        Gets session
        _[OpenID4VCI/HTTPS]_
    `" --> pid_issuer

    digid_connector -- "`
        Identifies user
        _[SAML/HTTPS]_
    `" --> digid
    pid_issuer -- "`
        Looks up PID attributes
        _[JSON/HTTPS]_
    `" --> brp

    wallet_app -- "`
        Discloses attributes
        _[MDOC/HTTPS]_
    `" --> relying_party_server
    wallet_app -- "`
        Gets session
        _[MDOC/HTTPS]_
    `" --> relying_party_server

    relying_party -- "`
        Gets disclosed attributes
        _[JSON/HTTPS_]
    `" --> relying_party_server
    relying_party -- "`
        Starts session
        _[JSON/HTTPS_]
    `" --> relying_party_server
```
