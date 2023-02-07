import '../../../data/repository/card/data_attribute_repository.dart';
import '../../../data/repository/wallet/wallet_repository.dart';
import '../../model/attribute/attribute.dart';

class IsWalletInitializedWithPid {
  final WalletRepository _walletRepository;
  final DataAttributeRepository _dataAttributeRepository;

  IsWalletInitializedWithPid(this._walletRepository, this._dataAttributeRepository);

  Future<bool> invoke() async {
    final isInitialized = await _walletRepository.isInitializedStream.first;
    if (!isInitialized) return false;
    final pidOnlyAttribute = await _dataAttributeRepository.find(AttributeType.citizenshipNumber);
    return pidOnlyAttribute != null;
  }
}
