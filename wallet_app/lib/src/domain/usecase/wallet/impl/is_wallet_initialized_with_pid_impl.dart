import '../../../../data/repository/card/data_attribute_repository.dart';
import '../../../../data/repository/wallet/wallet_repository.dart';
import '../../../model/attribute/attribute.dart';
import '../is_wallet_initialized_with_pid_usecase.dart';

class IsWalletInitializedWithPidUseCaseImpl implements IsWalletInitializedWithPidUseCase {
  final WalletRepository _walletRepository;
  final DataAttributeRepository _dataAttributeRepository;

  IsWalletInitializedWithPidUseCaseImpl(this._walletRepository, this._dataAttributeRepository);

  @override
  Future<bool> invoke() async {
    final isInitialized = await _walletRepository.isRegistered();
    if (!isInitialized) return false;
    final pidOnlyAttribute = await _dataAttributeRepository.find(AttributeType.citizenshipNumber);
    return pidOnlyAttribute != null;
  }
}
