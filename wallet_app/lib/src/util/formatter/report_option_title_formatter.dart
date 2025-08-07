import 'package:flutter/cupertino.dart';

import '../../feature/report_issue/reporting_option.dart';
import '../extension/build_context_extension.dart';

class ReportOptionTitleFormatter {
  static String map(BuildContext context, ReportingOption input) {
    switch (input) {
      case ReportingOption.unknownOrganization:
        return context.l10n.reportOptionUnknownOrganization;
      case ReportingOption.requestNotInitiated:
        return context.l10n.reportOptionRequestNotInitiated;
      case ReportingOption.impersonatingOrganization:
        return context.l10n.reportOptionImpersonatingOrganization;
      case ReportingOption.untrusted:
        return context.l10n.reportOptionUntrusted;
      case ReportingOption.overAskingOrganization:
        return context.l10n.reportOptionOverAskingOrganization;
      case ReportingOption.suspiciousOrganization:
        return context.l10n.reportOptionSuspiciousOrganization;
      case ReportingOption.unreasonableTerms:
        return context.l10n.reportOptionUnreasonableTerms;
    }
  }
}
