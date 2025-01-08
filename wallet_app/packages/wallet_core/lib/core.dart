/// This implementation communicates with the rust implementation of the wallet_core
library;

/// Export [FlutterRustBridgeTaskConstMeta] so wallet_mock can implement [WalletCore]
export 'package:flutter_rust_bridge/flutter_rust_bridge.dart' show FlutterRustBridgeTaskConstMeta;

export 'src/frb_generated.dart';
// export 'src/wallet_core.dart'; //todo: restore or remove?
export 'src/api/full.dart';
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
