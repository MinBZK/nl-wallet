import 'package:flutter/cupertino.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class TimelineAttributeStatusDescriptionTextMapper {
  static String map(BuildContext context, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionStatusTextFormatter.map(context, attribute);
    if (attribute is OperationTimelineAttribute) return OperationStatusTextFormatter.map(context, attribute.status);
    if (attribute is SigningTimelineAttribute) return SigningStatusTextFormatter.map(context, attribute);
    throw ('Unsupported attribute: $attribute');
  }
}

class InteractionStatusTextFormatter {
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

class OperationStatusTextFormatter {
  static String map(BuildContext context, OperationStatus status) {
    switch (status) {
      case OperationStatus.issued:
        return context.l10n.historyDetailScreenOperationStatusIssuedDescription;
      case OperationStatus.renewed:
        return context.l10n.historyDetailScreenOperationStatusRenewedDescription;
      case OperationStatus.expired:
        return context.l10n.historyDetailScreenOperationStatusExpiredDescription;
    }
  }
}

class SigningStatusTextFormatter {
  static String map(BuildContext context, SigningTimelineAttribute attribute) {
    switch (attribute.status) {
      case SigningStatus.success:
        return '';
      case SigningStatus.rejected:
        return context.l10n.historyDetailScreenSigningStatusRejectedDescription(attribute.organization.shortName);
    }
  }
}
