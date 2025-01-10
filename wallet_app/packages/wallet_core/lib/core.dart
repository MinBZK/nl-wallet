/// This implementation communicates with the rust implementation of the wallet_core
library;

import 'src/api/full.dart' as core;

/// Export [FlutterRustBridgeTaskConstMeta] so wallet_mock can implement [WalletCore]
export 'package:flutter_rust_bridge/flutter_rust_bridge.dart' show FlutterRustBridgeTaskConstMeta;

// export 'src/wallet_core.dart'; //todo: restore or remove?
export 'src/api/full.dart';
export 'src/frb_generated.dart';
export 'src/models/card.dart';
export 'src/models/config.dart';
export 'src/models/disclosure.dart';
export 'src/models/instruction.dart';
export 'src/models/pin.dart';
export 'src/models/uri.dart';
export 'src/models/version_state.dart';
export 'src/models/wallet_event.dart';

// Hardcoded docTypes, these are exposed here because the card data is still enriched
// based on this docType inside wallet_app (see [CardFrontMapper]). To be removed #someday
const kPidDocType = 'com.example.pid';
const kAddressDocType = 'com.example.address';

Future<void> postInit() async {
  if (await core.isInitialized()) {
    // The wallet_core is already initialized, this can happen when the Flutter
    // engine/activity was killed, but the application (and thus native code) was
    // kept alive by the platform. To recover from this we make sure the streams are reset,
    // as they can contain references to the previous Flutter engine.
    await core.clearLockStream();
    await core.clearConfigurationStream();
    await core.clearVersionStateStream();
    await core.clearCardsStream();
    await core.clearRecentHistoryStream();
    // Make sure the wallet is locked, as the [AutoLockObserver] was also killed.
    await core.lockWallet();
  }
}
