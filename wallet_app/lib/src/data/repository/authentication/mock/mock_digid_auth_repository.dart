import 'package:fimber/fimber.dart';

import '../../../../../bridge_generated.dart';
import '../digid_auth_repository.dart';

class MockDigidAuthRepository extends DigidAuthRepository {
  @override
  Future<String> getAuthUrl() async => 'mock://auth_url';

  @override
  void notifyDigidStateUpdate(DigidState? state) {
    Fimber.d('Received DigidState update: $state');
  }

  @override
  Stream<DigidAuthStatus> observeAuthStatus() => const Stream.empty();
}
