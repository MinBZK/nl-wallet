import 'package:flutter/cupertino.dart';
import 'package:intl/intl.dart';

import '../extension/build_context_extension.dart';

class OperationIssuedTimeFormatter {
  static String format(BuildContext context, DateTime dateTime) {
    DateFormat dateTimeFormat = DateFormat(DateFormat.MONTH_DAY, context.l10n.localeName).add_Hm();
    return dateTimeFormat.format(dateTime);
  }
}
