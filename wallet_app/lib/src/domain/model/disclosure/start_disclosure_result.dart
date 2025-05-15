import '../attribute/attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import '../requested_attributes.dart';
import 'disclosure_session_type.dart';
import 'disclosure_type.dart';

sealed class StartDisclosureResult {
  final Organization relyingParty;
  final String originUrl;
  final LocalizedText requestPurpose;
  final bool sharedDataWithOrganizationBefore;
  final DisclosureSessionType sessionType;

  StartDisclosureResult({
    required this.relyingParty,
    required this.originUrl,
    required this.requestPurpose,
    required this.sessionType,
    required this.sharedDataWithOrganizationBefore,
  });
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final RequestedAttributes requestedAttributes;
  final Policy policy;
  final DisclosureType type;

  StartDisclosureReadyToDisclose({
    required super.relyingParty,
    required super.originUrl,
    required super.requestPurpose,
    required super.sessionType,
    required this.type,
    required this.requestedAttributes,
    required this.policy,
    required super.sharedDataWithOrganizationBefore,
  });
}

class StartDisclosureMissingAttributes extends StartDisclosureResult {
  final List<MissingAttribute> missingAttributes;

  StartDisclosureMissingAttributes({
    required super.relyingParty,
    required super.originUrl,
    required super.requestPurpose,
    required super.sessionType,
    required this.missingAttributes,
    required super.sharedDataWithOrganizationBefore,
  });
}
