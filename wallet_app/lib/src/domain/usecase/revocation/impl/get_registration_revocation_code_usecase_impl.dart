import '../../../../data/repository/revocation/revocation_code_repository.dart';
import '../../../model/result/result.dart';
import '../get_registration_revocation_code_usecase.dart';

class GetRegistrationRevocationCodeUseCaseImpl extends GetRegistrationRevocationCodeUseCase {
  final RevocationRepository _revocationRepository;

  GetRegistrationRevocationCodeUseCaseImpl(this._revocationRepository);

  @override
  Future<Result<String>> invoke() {
    return tryCatch(
      _revocationRepository.getRegistrationRevocationCode,
      'Failed to fetch revocation code (registration)',
    );
  }
}
