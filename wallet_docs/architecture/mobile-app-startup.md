# Mobile App Startup

When the app is (cold) started the `wallet_core` is notified and given a multiple bridges so that it can call into the native iOS & Android world on its own merit. This process is documented in [initialization](#initialization).

Next to the main initialization of `wallet_core` we also setup two callbacks so that the `wallet_core` can notify the `wallet_app` of changes in its [configuration](#configuration) and [wallet lock](#wallet-lock) state.

## Initialization

This diagram captures the communication between the different app layers which occurs when the app is started.

Note that we check whether the `wallet_core` is already initialized, this is needed because the flutter engine can be destroyed while the native app (which includes `wallet_core`) is kept alive by the os. When this happens we want to rebind to the `wallet_core` instead of re-initializing it.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title App Startup [1.1]

    user->>platform: open app
    activate user
        par native init
            platform->>wallet_core: initPlatformSupport(signingKeyBridge, encryptionKeyBridge, utilitiesBridge)
            wallet_core->>wallet_core: store bridges
        and wallet_app (flutter) init
            platform->>wallet_app: start wallet_app
            wallet_app->>wallet_core: isInitialized()
            alt
                wallet_core-->>wallet_app: true
                wallet_app->>wallet_core: clear stale callbacks (e.g. config)
            else
                wallet_core-->>wallet_app: false
                wallet_app->>wallet_core: init()
                wallet_core-->>wallet_app: success
            end
            wallet_app->>wallet_app: initialization completed
            Note over wallet_app: setup callbacks, see 1.1, 1.2
        end
        wallet_app->>user: UI
    deactivate user
```

## Extra callbacks

While the callback setup doesn't strictly (all) happen during the app startup (i.e. the streams are configured but the reference isn't passed on to `wallet_core` until the stream has an observer), it is relevant here because it is initiated at app startup and these callbacks have to be cleared when they become stale (i.e. `isInitialized()` returned `true` during the [initialization](#initialization)).

### Configuration

This stream provides app configuration data to the wallet_app (e.g. background lock timeout duration). And can be updated throughout the complete lifecycle of the app.

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title Configuration stream setup [1.2]

    user->>platform: open app
    activate user
        Note over platform,wallet_core: initialization process, see 1.1
        wallet_app->>wallet_app: initialization completed
        wallet_app->>wallet_core: setConfigurationStream()
        wallet_app->>user: UI
    deactivate user
```

### Wallet lock

This stream provides information on the current wallet lock state (i.e. if the user is logged in and can view her attestations)

```{mermaid}
sequenceDiagram
    %% Force ordering by explicitly setting up participants
    actor user
    participant platform
    participant wallet_app
    participant wallet_core
    participant wallet_provider
    title Wallet lock stream setup [1.2]

    user->>platform: open app
    activate user
        Note over platform,wallet_core: initialization process, see 1.1
        wallet_app->>wallet_app: initialization completed
        wallet_app->>wallet_core: setLockStream()
        wallet_app->>user: UI
    deactivate user
```
