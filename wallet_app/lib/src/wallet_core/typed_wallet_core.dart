
import 'package:core_domain/core_domain.dart';

import '../../bridge_generated.dart';

abstract class TypedWalletCore {
  Future<PinValidationResult> isValidPin(String pin);

  Future<void> register(String pin);

  Future<bool> isRegistered();

  Future<void> lockWallet();

  Future<WalletUnlockResult> unlockWallet(String pin);

  Stream<bool> get isLocked;

  Future<String> getDigidAuthUrl();

  Stream<UriFlowEvent> processUri(Uri uri);

  Stream<FlutterConfiguration> observeConfig();
}
