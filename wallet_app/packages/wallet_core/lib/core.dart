/// This implementation communicates with the rust implementation of the wallet_core
library;

import 'src/api/full.dart' as core;

export 'src/api/full.dart';
export 'src/frb_generated.dart';
export 'src/models/attestation.dart';
export 'src/models/config.dart';
export 'src/models/disclosure.dart';
export 'src/models/image.dart';
export 'src/models/instruction.dart';
export 'src/models/localize.dart';
export 'src/models/pin.dart';
export 'src/models/transfer.dart';
export 'src/models/uri.dart';
export 'src/models/version_state.dart';
export 'src/models/wallet_event.dart';

Future<void> postInit() async {
  if (await core.isInitialized()) {
    // We always reset the streams here to avoid an invalid state where the native code
    // still contains references to the old Flutter engine. This can happen when the activity
    // was killed by the operating system, this causes the flutter engine to be killed, but
    // the native code might be kept alive.
    await core.clearLockStream();
    await core.clearConfigurationStream();
    await core.clearVersionStateStream();
    await core.clearAttestationsStream();
    await core.clearRecentHistoryStream();
    // Make sure the wallet is locked, as the [AutoLockObserver] was also killed.
    await core.lockWallet();
  }
}
