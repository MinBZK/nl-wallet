import '../../../../data/repository/revocation/revocation_code_repository.dart';
import '../../../model/result/result.dart';
import '../get_revocation_code_usecase.dart';

class GetRevocationCodeUseCaseImpl extends GetRevocationCodeUseCase {
  final RevocationRepository _revocationRepository;

  GetRevocationCodeUseCaseImpl(this._revocationRepository);

  @override
  Future<Result<String>> invoke(String pin) {
    return tryCatch(
      () => _revocationRepository.getRevocationCode(pin),
      'Failed to fetch revocation code',
    );
  }
}
