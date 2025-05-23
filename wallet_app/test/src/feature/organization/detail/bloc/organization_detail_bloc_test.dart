import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/organization/detail/bloc/organization_detail_bloc.dart';

import '../../../../mocks/wallet_mock_data.dart';

void main() {
  setUp(() {});

  blocTest(
    'verify initial state',
    build: OrganizationDetailBloc.new,
    verify: (bloc) => expect(bloc.state, OrganizationDetailInitial()),
  );

  blocTest(
    'MenuLockWalletPressed invokes the lock usecase',
    build: OrganizationDetailBloc.new,
    act: (bloc) => bloc
        .add(OrganizationProvided(sharedDataWithOrganizationBefore: true, organization: WalletMockData.organization)),
  );

  group('states', () {
    test('OrganizationDetailInitial equals works', () {
      final a = OrganizationDetailInitial();
      final b = OrganizationDetailInitial();
      expect(a, b, reason: 'OrganizationDetailInitial instances should be equal');
    });

    test('OrganizationDetailSuccess equals works', () {
      final a = OrganizationDetailSuccess(
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: true,
      );
      final b = OrganizationDetailSuccess(
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: true,
      );
      final c = OrganizationDetailSuccess(
        organization: WalletMockData.organization,
        sharedDataWithOrganizationBefore: false,
      );
      expect(a, b, reason: 'OrganizationDetailSuccess instances should a & b should be equal');
      expect(a, isNot(c), reason: 'OrganizationDetailSuccess instances should a & c should not be equal');
    });

    test('OrganizationDetailFailure equals works', () {
      final a = OrganizationDetailFailure(organizationId: '1');
      final b = OrganizationDetailFailure(organizationId: '1');
      final c = OrganizationDetailFailure(organizationId: 'other');
      expect(a, b, reason: 'OrganizationDetailFailure instances should a & b should be equal');
      expect(a, isNot(c), reason: 'OrganizationDetailFailure instances should a & c should not be equal');
    });
  });

  group('events', () {
    test('OrganizationProvided equals works', () {
      final a = OrganizationProvided(sharedDataWithOrganizationBefore: true, organization: WalletMockData.organization);
      final b = OrganizationProvided(sharedDataWithOrganizationBefore: true, organization: WalletMockData.organization);
      final c =
          OrganizationProvided(sharedDataWithOrganizationBefore: false, organization: WalletMockData.organization);
      expect(a, b, reason: 'OrganizationProvided instances should a & b should be equal');
      expect(a, isNot(c), reason: 'OrganizationProvided instances should a & c should not be equal');
    });
  });
}
