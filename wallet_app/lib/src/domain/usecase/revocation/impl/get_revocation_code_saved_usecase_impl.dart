import '../../../../data/repository/revocation/revocation_code_repository.dart';
import '../../../model/result/result.dart';
import '../get_revocation_code_saved_usecase.dart';

class GetRevocationCodeSavedUseCaseImpl extends GetRevocationCodeSavedUseCase {
  final RevocationRepository _revocationRepository;

  GetRevocationCodeSavedUseCaseImpl(this._revocationRepository);

  @override
  Future<Result<bool>> invoke() {
    return tryCatch(
      _revocationRepository.getRevocationCodeSaved,
      'Failed to read revocation code saved flag',
    );
  }
}
