/// This implementation communicates with the rust implementation of the wallet_core
library;

/// Export [FlutterRustBridgeTaskConstMeta] so wallet_mock can implement [WalletCore]
export 'package:flutter_rust_bridge/flutter_rust_bridge.dart' show FlutterRustBridgeTaskConstMeta;

export 'src/bridge_generated.dart';
export 'src/wallet_core.dart';

// Hardcoded docTypes, these are exposed here because the card data is still enriched
// based on this docType inside wallet_app (see [CardFrontMapper]). To be removed #someday
const kPidDocType = 'com.example.pid';
const kAddressDocType = 'com.example.address';
