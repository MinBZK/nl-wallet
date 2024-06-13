import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../extension/build_context_extension.dart';

class CardValidUntilTimeFormatter {
  static String format(BuildContext context, DateTime dateTime) {
    final DateFormat dateTimeFormat = DateFormat(DateFormat.YEAR_MONTH_DAY, context.l10n.localeName);
    return dateTimeFormat.format(dateTime);
  }
}
