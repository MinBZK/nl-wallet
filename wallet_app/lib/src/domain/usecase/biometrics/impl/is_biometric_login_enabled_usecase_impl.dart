import '../../../../data/repository/biometric/biometric_repository.dart';
import '../is_biometric_login_enabled_usecase.dart';

class IsBiometricLoginEnabledUseCaseImpl extends IsBiometricLoginEnabledUseCase {
  final BiometricRepository _biometricRepository;

  IsBiometricLoginEnabledUseCaseImpl(this._biometricRepository);

  @override
  Future<bool> invoke() async => _biometricRepository.isBiometricLoginEnabled();
}
