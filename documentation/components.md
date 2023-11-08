# Components overview

The diagram below shows a global overview of how the components interact with each other. It contains some details about the application layer protocols and technologies used, but leaves out exact (data) flow and detailed steps. Arrow directions indicate which party initiated the interaction and not the flow of data.
```mermaid
graph
    brp["`
        BRP
        _[Software system]_
        Contains the personal data of people
    `"]
    digid["`
        DigiD
        _[Software system]_
        Used in dealings with Dutch government
    `"]

    wallet_app("`
        Wallet App
        _[Container: Flutter/Rust]_
    `")

    wallet_app -- "`
        Makes API calls to
        _[JSON/HTTPS]_
    `"--> wallet_provider

    subgraph Wallet Provider
        wallet_provider["`
            Wallet Provider
            _[Container: Rust]_
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
            _[Container: Rust]_
        `"]
        pid_issuer_server["`
            PID Issuer Server
            _[Container: Rust (wallet_server)]_
        `"]
        redis[("`
            Cache
            _[Container: Redis]_
        `")]

        pid_issuer -- "`
            Gets BSN
            _[JSON/HTTPS]_
        `" --> digid_connector
        pid_issuer -- "`
            Starts session with PID
            _[JSON/HTTPS]_
        `" --> pid_issuer_server
        pid_issuer_server -- "`
            Stores session
            _[JSON/HTTPS]_
        `" --> redis
    end

    subgraph RP[Relying Party]
        relying_party_server["`
            Relying Party Server
            _[Container: Rust (wallet_server)]_
        `"]
        relying_party["`
            Relying Party
            _[Container: Rust]_
        `"]
        redis_rp[("`
            Cache
            _[Container: Redis]_
        `")]

        relying_party -- "`
            Starts session
            _[JSON/HTTPS_]
        `" --> relying_party_server
        relying_party_server -- "`
            Disclosed attributes
            _[JSON/HTTPS_]
        `" --> relying_party
        relying_party_server -- "`
            Stores session
            _[JSON/HTTPS]_
        `"--> redis_rp
    end

    wallet_app -- "`
        Authenticates user
        _[OIDC/HTTPS]_
    `" --> digid_connector
    wallet_app -- "`
        Gets session
        _[MDOC/HTTPS]_
    `" --> pid_issuer
    wallet_app -- "`
        Gets PID issued
        _[MDOC/HTTPS]_
    `" --> pid_issuer_server

    digid_connector -- "`
        Uses
        _[SAML/HTTPS]_
    `" --> digid
    pid_issuer -- "`
        Looks up attributes
        _[JSON/HTTPS]_
    `" --> brp

    wallet_app -- "`
        Gets session
        _[MDOC/HTTPS]_
    `" --> relying_party
    wallet_app -- "`
        Discloses attributes
        _[MDOC/HTTPS]_
    `" --> relying_party_server
```
