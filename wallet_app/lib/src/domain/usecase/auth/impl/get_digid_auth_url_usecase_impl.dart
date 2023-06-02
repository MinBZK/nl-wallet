import '../../../../data/repository/authentication/digid_auth_repository.dart';
import '../get_digid_auth_url_usecase.dart';

class GetDigidAuthUrlUseCaseImpl implements GetDigidAuthUrlUseCase {
  final DigidAuthRepository _authRepository;

  GetDigidAuthUrlUseCaseImpl(this._authRepository);

  @override
  Future<String> invoke() => _authRepository.getAuthUrl();
}
