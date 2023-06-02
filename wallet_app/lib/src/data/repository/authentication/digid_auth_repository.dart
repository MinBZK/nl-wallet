import 'package:core_domain/core_domain.dart';

abstract class DigidAuthRepository {
  Future<String> getAuthUrl();

  void notifyDigidStateUpdate(DigidState? state);

  Stream<DigidAuthStatus> observeAuthStatus();
}

enum DigidAuthStatus { idle, authenticating, success, error }
