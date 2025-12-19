import '../wallet_usecase.dart';

/// Use case for observing the push notifications (in-app) setting.
abstract class ObservePushNotificationsSettingUseCase extends WalletUseCase {
  /// Observes a stream of the push notifications setting, emits when the setting is toggled.
  Stream<bool> invoke();
}
