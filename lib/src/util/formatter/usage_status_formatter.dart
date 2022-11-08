import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../domain/model/usage_attribute.dart';

class UsageStatusFormatter {
  static String format(AppLocalizations locale, UsageStatus status) {
    switch (status) {
      case UsageStatus.success:
        return locale.cardSummaryDataShareStatusSuccess;
      case UsageStatus.rejected:
        return locale.cardSummaryDataShareStatusRejected;
      case UsageStatus.failed:
        return locale.cardSummaryDataShareStatusFailed;
      default:
        throw ('Unsupported status: $status');
    }
  }
}
