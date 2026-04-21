import '../attribute/attribute.dart';
import '../organization.dart';
import '../policy/policy.dart';
import 'disclose_card_request.dart';
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

  StartDisclosureResult copyWith({
    Organization? relyingParty,
    String? originUrl,
    LocalizedText? requestPurpose,
    DisclosureSessionType? sessionType,
    bool? sharedDataWithOrganizationBefore,
  });
}

class StartDisclosureReadyToDisclose extends StartDisclosureResult {
  final List<DiscloseCardRequest> cardRequests;
  final Policy policy;
  final DisclosureType type;

  StartDisclosureReadyToDisclose({
    required super.relyingParty,
    required super.originUrl,
    required super.requestPurpose,
    required super.sessionType,
    required this.type,
    required this.cardRequests,
    required this.policy,
    required super.sharedDataWithOrganizationBefore,
  });

  @override
  StartDisclosureReadyToDisclose copyWith({
    Organization? relyingParty,
    String? originUrl,
    LocalizedText? requestPurpose,
    DisclosureSessionType? sessionType,
    bool? sharedDataWithOrganizationBefore,
    DisclosureType? type,
    List<DiscloseCardRequest>? cardRequests,
    Policy? policy,
  }) {
    return StartDisclosureReadyToDisclose(
      relyingParty: relyingParty ?? this.relyingParty,
      originUrl: originUrl ?? this.originUrl,
      requestPurpose: requestPurpose ?? this.requestPurpose,
      sessionType: sessionType ?? this.sessionType,
      sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore ?? this.sharedDataWithOrganizationBefore,
      type: type ?? this.type,
      cardRequests: cardRequests ?? this.cardRequests,
      policy: policy ?? this.policy,
    );
  }
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

  @override
  StartDisclosureMissingAttributes copyWith({
    Organization? relyingParty,
    String? originUrl,
    LocalizedText? requestPurpose,
    DisclosureSessionType? sessionType,
    bool? sharedDataWithOrganizationBefore,
    List<MissingAttribute>? missingAttributes,
  }) {
    return StartDisclosureMissingAttributes(
      relyingParty: relyingParty ?? this.relyingParty,
      originUrl: originUrl ?? this.originUrl,
      requestPurpose: requestPurpose ?? this.requestPurpose,
      sessionType: sessionType ?? this.sessionType,
      sharedDataWithOrganizationBefore: sharedDataWithOrganizationBefore ?? this.sharedDataWithOrganizationBefore,
      missingAttributes: missingAttributes ?? this.missingAttributes,
    );
  }
}
