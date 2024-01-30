import '../attribute/attribute.dart';
import '../attribute/data_attribute.dart';
import '../attribute/missing_attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import '../wallet_card.dart';

sealed class StartDisclosureResult {
  final Organization relyingParty;
  final String originUrl;
  final LocalizedText requestPurpose;
  final bool sharedDataWithOrganizationBefore;

  StartDisclosureResult(
    this.relyingParty,
    this.requestPurpose,
    this.originUrl,
    this.sharedDataWithOrganizationBefore,
  );
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;
  final Policy policy;

  StartDisclosureReadyToDisclose(
    super.relyingParty,
    this.policy,
    super.originUrl,
    super.requestPurpose,
    super.sharedDataWithOrganizationBefore,
    this.requestedAttributes,
  );
}

class StartDisclosureMissingAttributes extends StartDisclosureResult {
  final List<MissingAttribute> missingAttributes;

  StartDisclosureMissingAttributes(
    super.relyingParty,
    super.requestPurpose,
    super.originUrl,
    super.sharedDataWithOrganizationBefore,
    this.missingAttributes,
  );
}
