import 'package:flutter_gen/gen_l10n/app_localizations.dart';

import '../../feature/report_issue/report_issue_screen.dart';

class ReportingOptionTitleMapper {
  static String map(AppLocalizations locale, ReportingOption option) {
    switch (option) {
      case ReportingOption.unknownOrganization:
        return locale.reportOptionUnknownOrganization;
      case ReportingOption.requestNotInitiated:
        return locale.reportOptionRequestNotInitiated;
      case ReportingOption.impersonatingOrganization:
        return locale.reportOptionImpersonatingOrganization;
      case ReportingOption.untrusted:
        return locale.reportOptionUntrusted;
      case ReportingOption.overAskingOrganization:
        return locale.reportOptionOverAskingOrganization;
      case ReportingOption.suspiciousOrganization:
        return locale.reportOptionSuspiciousOrganization;
      case ReportingOption.unreasonableTerms:
        return locale.reportOptionUnreasonableTerms;
    }
  }
}
