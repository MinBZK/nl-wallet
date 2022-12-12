import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTypeTextMapper {
  static String map(AppLocalizations locale, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionTypeTextFormatter.map(locale, attribute.interactionType);
    if (attribute is OperationAttribute) return OperationTypeTextFormatter.map(locale, attribute.operationType);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionTypeTextFormatter {
  static String map(AppLocalizations locale, InteractionType type) {
    switch (type) {
      case InteractionType.success:
        return locale.cardHistoryTimelineInteractionSuccess;
      case InteractionType.rejected:
        return locale.cardHistoryTimelineInteractionRejected;
      case InteractionType.failed:
        return locale.cardHistoryTimelineInteractionFailed;
    }
  }
}

class OperationTypeTextFormatter {
  static String map(AppLocalizations locale, OperationType type) {
    switch (type) {
      case OperationType.issued:
        return locale.cardHistoryTimelineOperationIssued;
      case OperationType.renewed:
        return locale.cardHistoryTimelineOperationRenewed;
      case OperationType.expired:
        return locale.cardHistoryTimelineOperationExpired;
    }
  }
}
