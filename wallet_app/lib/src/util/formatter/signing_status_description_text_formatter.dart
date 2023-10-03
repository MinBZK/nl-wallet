import 'package:flutter/material.dart';

import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class SigningStatusDescriptionTextFormatter {
  static String map(BuildContext context, SigningTimelineAttribute attribute) {
    switch (attribute.status) {
      case SigningStatus.success:
        return '';
      case SigningStatus.rejected:
        return context.l10n.historyDetailScreenSigningStatusRejectedDescription(attribute.organization.shortName);
    }
  }
}
