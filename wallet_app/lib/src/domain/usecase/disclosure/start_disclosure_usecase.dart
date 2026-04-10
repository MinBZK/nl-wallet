import '../../../data/repository/disclosure/disclosure_repository.dart';
import '../../model/disclosure/start_disclosure_request.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class StartDisclosureUseCase extends WalletUseCase {
  Future<Result<StartDisclosureResult>> invoke(StartDisclosureRequest request);
}
