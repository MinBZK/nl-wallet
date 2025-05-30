import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.mocks.dart';

// Mock AppLocalizations with just the methods we need for testing
class AppLocalizationsMock extends AppLocalizations {
  AppLocalizationsMock(super.locale);

  @override
  String disclosureConfirmDataAttributesPageSharedNotStoredSubtitle(String organization) =>
      'SHARED_NOT_STORED: $organization';

  @override
  String disclosureConfirmDataAttributesPageSharedAndStoredSubtitle(int months, String organization) =>
      'SHARED_AND_STORED: $organization for $months months';

  @override
  String disclosureConfirmDataAttributesPageNotSharedNotStoredSubtitle(String organization) =>
      'NOT_SHARED_NOT_STORED: $organization';

  @override
  String disclosureConfirmDataAttributesPageNotSharedButStoredSubtitle(int months, String organization) =>
      'NOT_SHARED_BUT_STORED: $organization for $months months';

  @override
  noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  group('PolicyBodyTextMapper', () {
    late PolicyBodyTextMapper mapper;
    late MockBuildContext context;

    setUp(() {
      mapper = PolicyBodyTextMapper(appLocalizations: AppLocalizationsMock('en'));
      context = MockBuildContext();
    });

    test('handles data shared but not stored', () {
      final policy = const Policy(
        dataIsShared: true,
        storageDuration: null, // Not stored
        deletionCanBeRequested: false,
        privacyPolicyUrl: 'https://example.com',
      );

      final orgPolicy = OrganizationPolicy(organization: WalletMockData.organization, policy: policy);

      final result = mapper.map(context, orgPolicy);

      expect(result, 'SHARED_NOT_STORED: Organization Display Name');
    });

    test('handles data shared and stored', () {
      final policy = const Policy(
        dataIsShared: true,
        storageDuration: Duration(days: 180), // 6 months
        deletionCanBeRequested: true,
        privacyPolicyUrl: 'https://example.com',
      );

      final orgPolicy = OrganizationPolicy(organization: WalletMockData.organization, policy: policy);

      final result = mapper.map(context, orgPolicy);

      expect(result, 'SHARED_AND_STORED: Organization Display Name for 6 months');
    });

    test('handles data not shared and not stored', () {
      final policy = const Policy(
        dataIsShared: false,
        storageDuration: null,
        deletionCanBeRequested: false,
        privacyPolicyUrl: 'https://example.com',
      );

      final orgPolicy = OrganizationPolicy(organization: WalletMockData.organization, policy: policy);

      final result = mapper.map(context, orgPolicy);

      expect(result, 'NOT_SHARED_NOT_STORED: Organization Display Name');
    });

    test('handles data not shared but stored', () {
      final policy = const Policy(
        dataIsShared: false,
        storageDuration: Duration(days: 180),
        deletionCanBeRequested: true,
        privacyPolicyUrl: 'https://example.com',
      );

      final orgPolicy = OrganizationPolicy(organization: WalletMockData.organization, policy: policy);

      final result = mapper.map(context, orgPolicy);

      expect(result, 'NOT_SHARED_BUT_STORED: Organization Display Name for 6 months');
    });
  });
}
