import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../model/result/result.dart';

/// Check the provided pin, optionally providing a result. Refer to the
/// individual usecases for info on this result. This interface is by the
/// [PinPage] & [PinBloc] to have consistent (business) logic and UI for
/// all pin entry flows throughout the app.
abstract class CheckPinUseCase extends WalletUseCase {
  Future<Result<dynamic>> invoke(String pin);
}
