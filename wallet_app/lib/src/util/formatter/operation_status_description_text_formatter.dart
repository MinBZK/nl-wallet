import 'package:flutter/material.dart';

import '../../domain/model/timeline/operation_timeline_attribute.dart';
import '../extension/build_context_extension.dart';

class OperationStatusDescriptionTextFormatter {
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
