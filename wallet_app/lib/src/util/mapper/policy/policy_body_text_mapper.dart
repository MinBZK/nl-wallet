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
    if (policy.dataIsShared && !policy.dataIsStored) {
      // Data IS shared but NOT stored
      return l10n.disclosureConfirmDataAttributesPageSharedNotStoredSubtitle(
        input.organization.displayName.l10nValue(context),
      );
    } else if (policy.dataIsShared && policy.dataIsStored) {
      // Data IS shared and IS stored
      return l10n.disclosureConfirmDataAttributesPageSharedAndStoredSubtitle(
        storageDuration.inMonths,
        input.organization.displayName.l10nValue(context),
      );
    } else if (!policy.dataIsShared && !policy.dataIsStored) {
      // Data is NOT shared and NOT stored
      return l10n.disclosureConfirmDataAttributesPageNotSharedNotStoredSubtitle(
        input.organization.displayName.l10nValue(context),
      );
    } else if (!policy.dataIsShared && policy.dataIsStored) {
      // Data is NOT shared but IS stored
      return l10n.disclosureConfirmDataAttributesPageNotSharedButStoredSubtitle(
        storageDuration.inMonths,
        input.organization.displayName.l10nValue(context),
      );
    }
    if (kDebugMode) throw UnsupportedError('No valid condition combination found');
    return '';
  }
}
