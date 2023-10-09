import 'package:flutter/material.dart';

import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class OperationStatusTextFormatter {
  static String map(BuildContext context, OperationStatus status) {
    switch (status) {
      case OperationStatus.issued:
        return context.l10n.cardHistoryTimelineOperationIssued;
      case OperationStatus.renewed:
        return context.l10n.cardHistoryTimelineOperationRenewed;
      case OperationStatus.expired:
        return context.l10n.cardHistoryTimelineOperationExpired;
    }
  }
}
