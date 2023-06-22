import 'package:flutter/cupertino.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class TimelineAttributeStatusTitleTextMapper {
  static String map(BuildContext context, TimelineAttribute attribute) {
    if (attribute is InteractionTimelineAttribute) return InteractionStatusTextFormatter.map(context, attribute.status);
    if (attribute is OperationTimelineAttribute) return attribute.cardTitle;
    if (attribute is SigningTimelineAttribute) return SigningStatusTextFormatter.map(context, attribute.status);
    throw ('Unsupported attribute: $attribute');
  }
}

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

class SigningStatusTextFormatter {
  static String map(BuildContext context, SigningStatus status) {
    switch (status) {
      case SigningStatus.success:
        return context.l10n.cardHistoryTimelineSigningSuccess;
      case SigningStatus.rejected:
        return context.l10n.cardHistoryTimelineSigningRejected;
    }
  }
}
