import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../extension/build_context_extension.dart';

class HistoryDetailsTimeFormatter {
  static String format(BuildContext context, DateTime dateTime) {
    final DateFormat dateTimeFormat = DateFormat('d MMMM y, HH:mm', context.l10n.localeName);
    return dateTimeFormat.format(dateTime);
  }
}
