import '../../../../data/repository/sign/sign_repository.dart';
import '../../../model/result/result.dart';
import '../../../model/start_sign_result/start_sign_result.dart';
import '../start_sign_usecase.dart';

class StartSignUseCaseImpl extends StartSignUseCase {
  final SignRepository _signRepository;

  StartSignUseCaseImpl(this._signRepository);

  @override
  Future<Result<StartSignResult>> invoke(String signUri) async {
    return tryCatch(
      () async => _signRepository.startSigning(signUri),
      'Failed to fetch sign flow details',
    );
  }
}
