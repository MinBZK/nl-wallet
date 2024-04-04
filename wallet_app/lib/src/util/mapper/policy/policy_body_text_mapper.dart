import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../../domain/model/policy/policy.dart';
import '../../extension/build_context_extension.dart';
import '../../extension/duration_extension.dart';
import '../context_mapper.dart';

class PolicyBodyTextMapper extends ContextMapper<Policy, String> {
  @override
  String map(BuildContext context, Policy input) {
    final storageDuration = input.storageDuration;
    bool dataIsStored = storageDuration != null;
    if (input.dataIsShared && !dataIsStored) {
      // Data IS shared but NOT stored
      return context.l10n.disclosureConfirmDataAttributesPageSharedNotStoredSubtitle;
    } else if (input.dataIsShared && dataIsStored) {
      // Data IS shared and IS stored
      return context.l10n.disclosureConfirmDataAttributesPageSharedAndStoredSubtitle(storageDuration.inMonths);
    } else if (!input.dataIsShared && !dataIsStored) {
      // Data is NOT shared and NOT stored
      return context.l10n.disclosureConfirmDataAttributesPageNotSharedNotStoredSubtitle;
    } else if (!input.dataIsShared && dataIsStored) {
      // Data is NOT shared but IS stored
      return context.l10n.disclosureConfirmDataAttributesPageNotSharedButStoredSubtitle(storageDuration.inMonths);
    }
    if (kDebugMode) throw UnsupportedError('No valid condition combination found');
    return '';
  }
}
