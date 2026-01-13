import '../../../../data/repository/revocation/revocation_code_repository.dart';
import '../../../model/result/result.dart';
import '../set_revocation_code_saved_usecase.dart';

class SetRevocationCodeSavedUseCaseImpl extends SetRevocationCodeSavedUseCase {
  final RevocationRepository _revocationRepository;

  SetRevocationCodeSavedUseCaseImpl(this._revocationRepository);

  @override
  Future<Result<void>> invoke({required bool saved}) {
    return tryCatch(
      () async => _revocationRepository.setRevocationCodeSaved(saved: saved),
      'Failed to set revocation code saved flag',
    );
  }
}
