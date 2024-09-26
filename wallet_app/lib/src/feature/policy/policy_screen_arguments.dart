import '../../domain/model/organization.dart';
import '../../domain/model/policy/policy.dart';

class PolicyScreenArguments {
  final Policy policy;
  final Organization relyingParty;
  final bool showSignatureRow;

  PolicyScreenArguments({
    required this.relyingParty,
    required this.policy,
    this.showSignatureRow = false,
  });
}
