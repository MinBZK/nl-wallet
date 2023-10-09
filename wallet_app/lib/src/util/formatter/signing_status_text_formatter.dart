import 'package:flutter/material.dart';

import '../../domain/model/timeline/signing_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

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
