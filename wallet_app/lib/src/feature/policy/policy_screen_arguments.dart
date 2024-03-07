import '../../domain/model/policy/policy.dart';

class PolicyScreenArguments {
  final Policy policy;
  final bool showSignatureRow;

  PolicyScreenArguments({required this.policy, this.showSignatureRow = false});
}
