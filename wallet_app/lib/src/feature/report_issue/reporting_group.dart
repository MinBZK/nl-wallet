import 'reporting_option.dart';

class ReportingGroup {
  static const List<ReportingOption> disclosureCheckOrganizationForLogin = [
    ReportingOption.unknownOrganization,
    ReportingOption.requestNotInitiated,
    ReportingOption.suspiciousOrganization,
    ReportingOption.impersonatingOrganization,
  ];

  static const List<ReportingOption> disclosureConfirm = [
    ReportingOption.untrusted,
    ReportingOption.overAskingOrganization,
    ReportingOption.suspiciousOrganization,
    ReportingOption.unreasonableTerms,
  ];

  static const List<ReportingOption> disclosureMissingAttributes = [
    ReportingOption.overAskingOrganization,
    ReportingOption.suspiciousOrganization,
  ];

  static const List<ReportingOption> issuance = [
    ReportingOption.unreasonableTerms,
    ReportingOption.suspiciousOrganization,
    ReportingOption.untrusted,
  ];
}
