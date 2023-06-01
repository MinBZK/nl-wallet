import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeStatusDescriptionTextMapper {
  static String map(AppLocalizations locale, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionStatusTextFormatter.map(locale, attribute);
    if (attribute is OperationTimelineAttribute) return OperationStatusTextFormatter.map(locale, attribute.status);
    if (attribute is SigningTimelineAttribute) return SigningStatusTextFormatter.map(locale, attribute);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusTextFormatter {
  static String map(AppLocalizations locale, InteractionTimelineAttribute attribute) {
    switch (attribute.status) {
      case InteractionStatus.success:
        return '';
      case InteractionStatus.rejected:
        return locale.historyDetailScreenInteractionStatusRejectedDescription(attribute.organization.shortName);
      case InteractionStatus.failed:
        return locale.historyDetailScreenInteractionStatusFailedDescription(attribute.organization.shortName);
    }
  }
}

class OperationStatusTextFormatter {
  static String map(AppLocalizations locale, OperationStatus status) {
    switch (status) {
      case OperationStatus.issued:
        return locale.historyDetailScreenOperationStatusIssuedDescription;
      case OperationStatus.renewed:
        return locale.historyDetailScreenOperationStatusRenewedDescription;
      case OperationStatus.expired:
        return locale.historyDetailScreenOperationStatusExpiredDescription;
    }
  }
}

class SigningStatusTextFormatter {
  static String map(AppLocalizations locale, SigningTimelineAttribute attribute) {
    switch (attribute.status) {
      case SigningStatus.success:
        return '';
      case SigningStatus.rejected:
        return locale.historyDetailScreenSigningStatusRejectedDescription(attribute.organization.shortName);
    }
  }
}
