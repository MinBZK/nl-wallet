import '../attribute/attribute.dart';
import '../attribute/data_attribute.dart';
import '../attribute/missing_attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import '../wallet_card.dart';
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
    this.sharedDataWithOrganizationBefore,
    this.sessionType,
  );
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final Policy policy;
  final DisclosureType type;

  StartDisclosureReadyToDisclose(
    super.relyingParty,
    super.originUrl,
    super.requestPurpose,
    super.sharedDataWithOrganizationBefore,
    super.sessionType,
    this.type,
    this.requestedAttributes,
    this.policy,
  );
}

class StartDisclosureMissingAttributes extends StartDisclosureResult {
  final List<MissingAttribute> missingAttributes;

  StartDisclosureMissingAttributes(
    super.relyingParty,
    super.requestPurpose,
    super.originUrl,
    super.sharedDataWithOrganizationBefore,
    super.sessionType,
    this.missingAttributes,
  );
}
