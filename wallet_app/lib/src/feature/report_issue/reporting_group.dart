import 'reporting_option.dart';

class ReportingGroup {
  static const List<ReportingOption> disclosureCheckOrganization = [
    ReportingOption.unknownOrganization,
    ReportingOption.requestNotInitiated,
    ReportingOption.suspiciousOrganization,
    ReportingOption.impersonatingOrganization,
  ];

  static const List<ReportingOption> disclosureConfirmMultipleAttributes = [
    ReportingOption.overAskingOrganization,
    ReportingOption.unreasonableTerms,
    ReportingOption.suspiciousOrganization,
    ReportingOption.requestUntrusted,
  ];

  static const List<ReportingOption> disclosureConfirmSingleAttribute = [
    ReportingOption.irrelevantAskingOrganization,
    ReportingOption.unreasonableTerms,
    ReportingOption.suspiciousOrganization,
    ReportingOption.requestUntrusted,
  ];

  static const List<ReportingOption> issuanceConfirmCards = [
    ReportingOption.incorrectCardData,
    ReportingOption.unreasonableTerms,
    ReportingOption.suspiciousOrganization,
  ];

  static const List<ReportingOption> disclosureMissingAttributes = [
    ReportingOption.overAskingOrganization,
    ReportingOption.suspiciousOrganization,
  ];
}
