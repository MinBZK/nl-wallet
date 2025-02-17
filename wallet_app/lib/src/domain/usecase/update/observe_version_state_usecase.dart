import '../../model/update/version_state.dart';
import '../wallet_usecase.dart';

export '../../model/update/version_state.dart';

abstract class ObserveVersionStateUsecase extends WalletUseCase {
  Stream<VersionState> invoke();
}
