import '../../model/notification/os_notification.dart';
import '../wallet_usecase.dart';

/// Registers a callback to handle notifications that should be shown immediately.
/// Note: this usecase takes the 'notifications enabled' setting into account.
abstract class SetDirectOsNotificationCallbackUsecase extends WalletUseCase {
  void invoke(Function(OsNotification) callback);
}
