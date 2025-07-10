import '../wallet_usecase.dart';

abstract class ObserveShowTourBannerUseCase extends WalletUseCase {
  Stream<bool> invoke();
}
