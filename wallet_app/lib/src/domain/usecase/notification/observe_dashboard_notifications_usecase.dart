import '../../../feature/banner/wallet_banner.dart';
import '../wallet_usecase.dart';

abstract class ObserveDashboardNotificationsUseCase extends WalletUseCase {
  Stream<List<WalletBanner>> invoke();
}
