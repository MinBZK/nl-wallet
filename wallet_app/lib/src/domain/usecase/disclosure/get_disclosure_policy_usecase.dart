import '../../model/policy/policy.dart';

abstract class GetDisclosurePolicyUseCase {
  Future<Policy> invoke(String sessionId);
}
