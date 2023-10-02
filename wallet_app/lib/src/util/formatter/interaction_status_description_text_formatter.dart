import 'package:flutter/material.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class InteractionStatusDescriptionTextFormatter {
  static String map(BuildContext context, InteractionTimelineAttribute attribute) {
    switch (attribute.status) {
      case InteractionStatus.success:
        return '';
      case InteractionStatus.rejected:
        return context.l10n.historyDetailScreenInteractionStatusRejectedDescription(attribute.organization.shortName);
      case InteractionStatus.failed:
        return context.l10n.historyDetailScreenInteractionStatusFailedDescription(attribute.organization.shortName);
    }
  }
}
