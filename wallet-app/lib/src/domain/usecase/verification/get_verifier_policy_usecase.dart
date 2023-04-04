import '../../model/policy/policy.dart';

abstract class GetVerifierPolicyUseCase {
  Future<Policy> invoke(String sessionId);
}
