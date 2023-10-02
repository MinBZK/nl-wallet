import 'package:flutter/cupertino.dart';

import '../../domain/model/timeline/interaction_timeline_attribute.dart';
import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../../domain/model/timeline/timeline_attribute.dart';
import 'interaction_status_description_text_formatter.dart';
import 'operation_status_description_text_formatter.dart';
import 'signing_status_description_text_formatter.dart';

class TimelineAttributeStatusDescriptionTextFormatter {
  static String map(BuildContext context, TimelineAttribute input) {
    if (input is InteractionTimelineAttribute) {
      return InteractionStatusDescriptionTextFormatter.map(context, input);
    }
    if (input is OperationTimelineAttribute) {
      return OperationStatusDescriptionTextFormatter.map(context, input.status);
    }
    if (input is SigningTimelineAttribute) return SigningStatusDescriptionTextFormatter.map(context, input);
    throw ('Unsupported attribute: $input');
  }
}
