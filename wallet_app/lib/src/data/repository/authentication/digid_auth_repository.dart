import '../../../../bridge_generated.dart';

abstract class DigidAuthRepository {
  Future<String> getAuthUrl();

  void notifyDigidStateUpdate(DigidState? state);

  Stream<DigidAuthStatus> observeAuthStatus();
}

enum DigidAuthStatus { idle, authenticating, success, error }
