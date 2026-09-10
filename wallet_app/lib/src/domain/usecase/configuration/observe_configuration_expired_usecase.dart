import '../wallet_usecase.dart';

/// Observes configuration expiry.
///
/// Emits `true` if the configuration is expired, meaning trust anchors cannot
/// be trusted and app interactions should be restricted.
abstract class ObserveConfigurationExpiredUseCase extends WalletUseCase {
  Stream<bool> invoke();
}
