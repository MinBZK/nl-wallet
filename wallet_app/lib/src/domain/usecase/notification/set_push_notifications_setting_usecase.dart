import '../wallet_usecase.dart';

/// Use case for setting the (in-app) push notifications setting.
abstract class SetPushNotificationsSettingUseCase extends WalletUseCase {
  /// Enables or disables push notifications. (in-app filter, permission handled by the OS).
  Future<void> invoke({required bool enabled});
}
