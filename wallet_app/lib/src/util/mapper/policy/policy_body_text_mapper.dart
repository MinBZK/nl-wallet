import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../../l10n/generated/app_localizations.dart';
import '../../../domain/model/attribute/attribute.dart';
import '../../../domain/model/policy/organization_policy.dart';
import '../../extension/build_context_extension.dart';
import '../../extension/duration_extension.dart';
import '../context_mapper.dart';

class PolicyBodyTextMapper extends ContextMapper<OrganizationPolicy, String> {
  @visibleForTesting
  AppLocalizations? appLocalizations;

  PolicyBodyTextMapper({this.appLocalizations});

  @override
  String map(BuildContext context, OrganizationPolicy input) {
    final l10n = appLocalizations ?? context.l10n;
    final policy = input.policy;
    final storageDuration = policy.storageDuration ?? Duration.zero;
    final organization = input.organization.displayName.l10nValue(context);
    final policyType = PolicyType.fromFlags(isShared: policy.dataIsShared, isStored: policy.dataIsStored);

    return _getSubtitleForPolicyType(
      policyType,
      l10n,
      organization,
      storageDuration.inMonths,
    );
  }

  String _getSubtitleForPolicyType(
    PolicyType policyType,
    AppLocalizations l10n,
    String organization,
    int storageDurationInMonths,
  ) {
    return switch (policyType) {
      PolicyType.sharedNotStored => l10n.disclosureConfirmDataAttributesPageSharedNotStoredSubtitle(organization),
      PolicyType.sharedAndStored => l10n.disclosureConfirmDataAttributesPageSharedAndStoredSubtitle(
        storageDurationInMonths,
        organization,
      ),
      PolicyType.notSharedNotStored => l10n.disclosureConfirmDataAttributesPageNotSharedNotStoredSubtitle(organization),
      PolicyType.notSharedButStored => l10n.disclosureConfirmDataAttributesPageNotSharedButStoredSubtitle(
        storageDurationInMonths,
        organization,
      ),
      PolicyType.unknown => kDebugMode ? throw UnsupportedError('No valid condition combination found') : '',
    };
  }
}

enum PolicyType {
  sharedNotStored,
  sharedAndStored,
  notSharedNotStored,
  notSharedButStored,
  unknown
  ;

  factory PolicyType.fromFlags({required bool isShared, required bool isStored}) {
    if (isShared && !isStored) return PolicyType.sharedNotStored;
    if (isShared && isStored) return PolicyType.sharedAndStored;
    if (!isShared && !isStored) return PolicyType.notSharedNotStored;
    if (!isShared && isStored) return PolicyType.notSharedButStored;
    return PolicyType.unknown;
  }
}
