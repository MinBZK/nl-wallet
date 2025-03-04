import '../attribute/attribute.dart';
import '../card/wallet_card.dart';
import '../organization.dart';
import '../policy/policy.dart';
import 'disclosure_session_type.dart';
import 'disclosure_type.dart';

sealed class StartDisclosureResult {
  final Organization relyingParty;
  final String originUrl;
  final LocalizedText requestPurpose;
  final bool sharedDataWithOrganizationBefore;
  final DisclosureSessionType sessionType;

  StartDisclosureResult(
    this.relyingParty,
    this.originUrl,
    this.requestPurpose,
    this.sessionType, {
    required this.sharedDataWithOrganizationBefore,
  });
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final Policy policy;
  final DisclosureType type;

  StartDisclosureReadyToDisclose(
    super.relyingParty,
    super.originUrl,
    super.requestPurpose,
    super.sessionType,
    this.type,
    this.requestedAttributes,
    this.policy, {
    required super.sharedDataWithOrganizationBefore,
  });
}

class StartDisclosureMissingAttributes extends StartDisclosureResult {
  final List<MissingAttribute> missingAttributes;

  StartDisclosureMissingAttributes(
    super.relyingParty,
    super.originUrl,
    super.requestPurpose,
    super.sessionType,
    this.missingAttributes, {
    required super.sharedDataWithOrganizationBefore,
  });
}
