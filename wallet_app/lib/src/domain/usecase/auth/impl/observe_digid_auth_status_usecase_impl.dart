import 'dart:async';

import '../../../../data/repository/authentication/digid_auth_repository.dart';
import '../observe_digid_auth_status_usecase.dart';

class ObserveDigidAuthStatusUseCaseImpl implements ObserveDigidAuthStatusUseCase {
  final DigidAuthRepository _authRepository;

  ObserveDigidAuthStatusUseCaseImpl(this._authRepository);

  @override
  Stream<DigidAuthStatus> invoke() {
    return _authRepository.observeAuthStatus().where((status) => status != DigidAuthStatus.idle);
  }
}
