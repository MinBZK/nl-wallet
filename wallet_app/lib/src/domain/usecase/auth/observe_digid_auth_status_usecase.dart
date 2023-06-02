import '../../../data/repository/authentication/digid_auth_repository.dart';

abstract class ObserveDigidAuthStatusUseCase {
  Stream<DigidAuthStatus> invoke();
}
