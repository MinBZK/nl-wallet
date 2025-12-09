import '../../model/notification/os_notification.dart';
import '../wallet_usecase.dart';

abstract class ObserveOsNotificationsUseCase extends WalletUseCase {
  Stream<List<OsNotification>> invoke();
}
