import '../attribute/attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import '../wallet_card.dart';

sealed class StartIssuanceResult {
  final Organization relyingParty;
  final Policy policy;

  StartIssuanceResult({required this.relyingParty, required this.policy});
}

class StartIssuanceReadyToDisclose extends StartIssuanceResult {
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;

  StartIssuanceReadyToDisclose({
    required super.relyingParty,
    required super.policy,
    required this.requestedAttributes,
  });
}

class StartIssuanceMissingAttributes extends StartIssuanceResult {
  final List<MissingAttribute> missingAttributes;

  StartIssuanceMissingAttributes({
    required super.relyingParty,
    required super.policy,
    required this.missingAttributes,
  });
}
