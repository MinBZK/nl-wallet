import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../model/card/wallet_card.dart';
import '../../../model/result/result.dart';
import '../check_is_pid.dart';

class CheckIsPidUseCaseImpl extends CheckIsPidUseCase {
  final ConfigurationRepository _configurationRepository;

  CheckIsPidUseCaseImpl(this._configurationRepository);

  @override
  Future<Result<bool>> invoke(WalletCard card) {
    return tryCatch(
      () async {
        final appConfig = await _configurationRepository.appConfiguration.first;
        return appConfig.pidAttestationTypes.contains(card.attestationType);
      },
      'Failed to check if ${card.attestationId} is PID',
    );
  }
}
