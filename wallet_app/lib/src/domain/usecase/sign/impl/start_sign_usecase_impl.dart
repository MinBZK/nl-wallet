import '../../../../data/repository/sign/sign_repository.dart';
import '../../../model/start_sign_result/start_sign_result.dart';
import '../start_sign_usecase.dart';

class StartSignUseCaseImpl implements StartSignUseCase {
  final SignRepository _signRepository;

  StartSignUseCaseImpl(this._signRepository);

  @override
  Future<StartSignResult> invoke(String signUri) async => _signRepository.startSigning(signUri);
}
