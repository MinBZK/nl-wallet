import '../../../../data/repository/configuration/configuration_repository.dart';
import '../observe_configuration_expired_usecase.dart';

class ObserveConfigurationExpiredUseCaseImpl extends ObserveConfigurationExpiredUseCase {
  final ConfigurationRepository _configurationRepository;

  ObserveConfigurationExpiredUseCaseImpl(this._configurationRepository);

  @override
  Stream<bool> invoke() => _configurationRepository.observeConfigExpired;
}
