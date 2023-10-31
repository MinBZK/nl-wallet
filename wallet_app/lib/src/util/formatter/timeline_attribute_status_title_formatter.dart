import 'package:flutter/cupertino.dart';

import '../../domain/model/attribute/attribute.dart';
import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';
import 'interaction_status_text_formatter.dart';
import 'signing_status_text_formatter.dart';

class TimelineAttributeStatusTitleFormatter {
  static String map(BuildContext context, TimelineAttribute input) {
    if (input is InteractionTimelineAttribute) return InteractionStatusTextFormatter.map(context, input.status);
    if (input is OperationTimelineAttribute) return input.cardTitle.l10nValue(context);
    if (input is SigningTimelineAttribute) return SigningStatusTextFormatter.map(context, input.status);
    throw ('Unsupported attribute: $input');
  }
}
