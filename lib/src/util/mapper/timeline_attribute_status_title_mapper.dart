import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeStatusTitleTextMapper {
  static String map(AppLocalizations locale, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionStatusTextFormatter.map(locale, attribute.status);
    if (attribute is OperationAttribute) return attribute.cardTitle;
    if (attribute is SigningAttribute) return SigningStatusTextFormatter.map(locale, attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusTextFormatter {
  static String map(AppLocalizations locale, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return locale.cardHistoryTimelineInteractionSuccess;
      case InteractionStatus.rejected:
        return locale.cardHistoryTimelineInteractionRejected;
      case InteractionStatus.failed:
        return locale.cardHistoryTimelineInteractionFailed;
    }
  }
}

class SigningStatusTextFormatter {
  static String map(AppLocalizations locale, SigningStatus status) {
    switch (status) {
      case SigningStatus.success:
        return locale.cardHistoryTimelineSigningSuccess;
      case SigningStatus.rejected:
        return locale.cardHistoryTimelineSigningRejected;
    }
  }
}
