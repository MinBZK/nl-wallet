The intention of this readme is to provide insight into the communication that happens between the different layers of the app.

# Sequence Diagrams

## Participants
A brief summary of the participants used in diagrams found below.

| Participant | Description                                                                                                       |
|---|-------------------------------------------------------------------------------------------------------------------|
| User | The end-user that downloads, installs and uses the application.                                                   |
| Native | The platform specific native layer, referring to iOS (Swift) or Android (Kotlin), depending on the host platform. |
| Flutter | The Flutter application code (i.e. Dart).                                                                         |
| Rust Core | The core business logic, built using Rust.                                                                        |
| Backend | The backend, its business logic is mostly kept out of scope. Often referred to as the Wallet Provider.           |

## App Startup

This diagram captures the communication between the different app layers which occurs when the app is started (cold start). Notably, besides the start of the Flutter app the `Rust Core` is initiated and given a reference so that it can call into the native iOS & Android world on its own merit.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor User
    participant Native
    participant Flutter
    participant Rust
    participant Backend
    title App Startup

    User->>Native: Open App
    activate User
        par Rust init
            Native->>Rust: Provide callback reference
            Rust->>Rust: Store reference
        and Flutter init
            Native->>Flutter: Start Flutter app 
            Flutter->>Flutter: Initialize app
        end
        Flutter->>User: UI
    deactivate User
```

## Pin Validation

This diagram illustrates the local validation that happens when the user selects a new pin.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor User
    participant Native
    participant Flutter
    participant Rust
    participant Backend
    title Pin Validation

    User->>Flutter: Enter new pin
    activate User
        Flutter->>Rust: Validate pin
        alt Pin Invalid
            Rust-->>Flutter: Pin Error
            Flutter-->>User: Render error & request new pin
        else Pin Valid
            Rust-->>Flutter: Pin Success
            Flutter-->>User: Render success
        end
    deactivate User
```


## Create Wallet

The diagram below illustrates the Wallet creation process. Including certificate generation and registration with the backend.

```mermaid
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor User
    participant Native
    participant Flutter
    participant Rust
    participant Backend
    title Create Wallet

    User->>Flutter: Provide valid pin
    activate User
        Flutter->>Rust: Enroll with pin
        Rust->>Backend: Fetch challenge
        Backend->>Rust: Challenge
        par key setup (pin key)
            Rust->>Rust: Generate salt
            Rust->>Native: Store salt
            Native->>Native: Persist salt
        and key setup (hw key)
            Rust->>Native: Generate HW key
            note over Native, Rust: HW key = Hardware backed key
        end
        par sign challenge (pin key)
            Rust->>Rust: Derive ECDSA keypair from pin + salt
            Rust->>Rust: Sign challenge with derived key
        and sign challenge (hw key)  
            Rust->>Native: Request HW signed challenge
            Native->>Native: Sign challange with HW key
            Native->>Rust: HW signed challenge
        end
        Rust->>Native: Get HW public key
        Native->>Rust: HW public key
        Rust->>Backend: Create Wallet<br/>[Pin PubKey & Pin signed challenge]<br/>[HW PubKey & HW signed challenge]
        Backend->>Backend: Create Wallet<br/>[Wallet ID]<br/>[Wallet Cert]
        Backend->>Rust: Wallet ID & Wallet Cert
        Rust->>Native: Store Wallet ID & Wallet Cert
        Native->>Native: Persist Wallet ID & Wallet Cert
        Rust->>Flutter: Enrollment success
        Flutter->>User: Render wallet created
    deactivate User
```