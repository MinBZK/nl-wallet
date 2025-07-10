import '../../model/result/result.dart';
import '../wallet_usecase.dart';

abstract class TourOverviewViewedUseCase extends WalletUseCase {
  Future<Result<void>> invoke();
}
