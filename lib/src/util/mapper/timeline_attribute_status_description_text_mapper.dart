import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/timeline/timeline_attribute.dart';

class TimelineAttributeStatusDescriptionTextMapper {
  static String map(AppLocalizations locale, TimelineAttribute attribute) {
    if (attribute is InteractionAttribute) return InteractionStatusTextFormatter.map(locale, attribute.status);
    if (attribute is OperationAttribute) return OperationStatusTextFormatter.map(locale, attribute.status);
    if (attribute is SigningAttribute) return SigningStatusTextFormatter.map(locale, attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusTextFormatter {
  static String map(AppLocalizations locale, InteractionStatus status) {
    switch (status) {
      case InteractionStatus.success:
        return '';
      case InteractionStatus.rejected:
        return locale.historyDetailScreenInteractionStatusRejectedDescription;
      case InteractionStatus.failed:
        return locale.historyDetailScreenInteractionStatusFailedDescription;
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
  static String map(AppLocalizations locale, SigningStatus status) {
    switch (status) {
      case SigningStatus.success:
        return '';
      case SigningStatus.rejected:
        return locale.historyDetailScreenSigningStatusRejectedDescription;
    }
  }
}
