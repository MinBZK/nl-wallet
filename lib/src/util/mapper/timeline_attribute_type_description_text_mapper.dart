import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline_attribute.dart';

class TimelineAttributeTypeDescriptionTextMapper {
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
        return '';
      case InteractionType.rejected:
        return locale.historyDetailScreenInteractionTypeRejectedDescription;
      case InteractionType.failed:
        return locale.historyDetailScreenInteractionTypeFailedDescription;
    }
  }
}

class OperationTypeTextFormatter {
  static String map(AppLocalizations locale, OperationType type) {
    switch (type) {
      case OperationType.issued:
        return locale.historyDetailScreenOperationTypeIssuedDescription;
      case OperationType.renewed:
        return locale.historyDetailScreenOperationTypeRenewedDescription;
      case OperationType.expired:
        return locale.historyDetailScreenOperationTypeExpiredDescription;
    }
  }
}
