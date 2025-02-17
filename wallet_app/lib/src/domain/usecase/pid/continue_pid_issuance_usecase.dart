import '../../model/attribute/attribute.dart';
import '../../model/result/result.dart';
import '../wallet_usecase.dart';

typedef PreviewAttributes = List<Attribute>;

abstract class ContinuePidIssuanceUseCase extends WalletUseCase {
  Future<Result<PreviewAttributes>> invoke(String uri);
}
