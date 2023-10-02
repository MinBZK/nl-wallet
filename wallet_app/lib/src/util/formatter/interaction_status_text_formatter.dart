import 'package:flutter/material.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class InteractionStatusTextFormatter {
  static String map(BuildContext context, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return context.l10n.cardHistoryTimelineInteractionSuccess;
      case InteractionStatus.rejected:
        return context.l10n.cardHistoryTimelineInteractionRejected;
      case InteractionStatus.failed:
        return context.l10n.cardHistoryTimelineInteractionFailed;
    }
  }
}
