import '../../../data/repository/organization/organization_repository.dart';
import '../attribute/data_attribute.dart';
import '../attribute/requested_attribute.dart';
import '../wallet_card.dart';

sealed class StartDisclosureResult {
  final Organization relyingParty;

  StartDisclosureResult(this.relyingParty);
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final Map<WalletCard, List<DataAttribute>> requestedAttributes;

  StartDisclosureReadyToDisclose(super.relyingParty, this.requestedAttributes);
}

class StartDisclosureMissingAttributes extends StartDisclosureResult {
  final List<RequestedAttribute> missingAttributes;

  StartDisclosureMissingAttributes(super.relyingParty, this.missingAttributes);
}
