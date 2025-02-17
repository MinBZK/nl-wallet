import '../../../data/repository/disclosure/disclosure_repository.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

export '../../../data/repository/disclosure/disclosure_repository.dart';

abstract class StartDisclosureUseCase extends WalletUseCase {
  Future<Result<StartDisclosureResult>> invoke(String disclosureUri, {bool isQrCode = false});
}
