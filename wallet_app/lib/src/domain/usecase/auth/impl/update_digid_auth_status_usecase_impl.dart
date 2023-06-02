import 'package:core_domain/core_domain.dart';

import '../../../../data/repository/authentication/digid_auth_repository.dart';
import '../update_digid_auth_status_usecase.dart';

class UpdateDigidAuthStatusUseCaseImpl extends UpdateDigidAuthStatusUseCase {
  final DigidAuthRepository _authRepository;

  UpdateDigidAuthStatusUseCaseImpl(this._authRepository);

  @override
  Future<void> invoke(DigidState state) async {
    _authRepository.notifyDigidStateUpdate(state);
  }
}
