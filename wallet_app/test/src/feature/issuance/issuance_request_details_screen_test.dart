import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/issuance/bloc/issuance_bloc.dart';
import 'package:wallet/src/feature/issuance/issuance_request_details_screen.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';
import 'issuance_screen_test.dart';

void main() {
  testGoldens('RequestDetailsContent Light', (tester) async {
    // Test the _buildContent UI with valid issuance data
    await tester.pumpWidgetWithAppWrapper(
      const IssuanceRequestDetailsScreen().withState<IssuanceBloc, IssuanceState>(
        MockIssuanceBloc(),
        IssuanceCheckOrganization(
          organization: WalletMockData.organization,
          policy: WalletMockData.policy,
          purpose: 'sample purpose'.untranslated,
          cardRequests: [WalletMockData.discloseCardRequestSingleCard],
        ),
      ),
      providers: [
        RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (_) => MockContextMapper()),
      ],
    );
    await screenMatchesGolden('request_details/light');
  });

  testGoldens('RequestDetailsContent Dark', (tester) async {
    // Test the _buildContent UI with valid issuance data
    await tester.pumpWidgetWithAppWrapper(
      const IssuanceRequestDetailsScreen().withState<IssuanceBloc, IssuanceState>(
        MockIssuanceBloc(),
        IssuanceCheckOrganization(
          organization: WalletMockData.organization,
          policy: WalletMockData.policy,
          purpose: 'sample purpose'.untranslated,
          cardRequests: [WalletMockData.discloseCardRequestMultiCard],
        ),
      ),
      brightness: Brightness.dark,
      providers: [
        RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (_) => MockContextMapper()),
      ],
    );
    await screenMatchesGolden('request_details/dark');
  });

  testGoldens('RequestDetailsContent Dark', (tester) async {
    // Test the _buildContent UI with valid issuance data
    await tester.pumpWidgetWithAppWrapper(
      const IssuanceRequestDetailsScreen().withState<IssuanceBloc, IssuanceState>(
        MockIssuanceBloc(),
        IssuanceCheckOrganization(
          organization: WalletMockData.organization,
          policy: WalletMockData.policy,
          purpose: 'sample purpose'.untranslated,
          cardRequests: [WalletMockData.discloseCardRequestMultiCard],
        ),
      ),
      brightness: Brightness.dark,
      providers: [
        RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
          create: (_) {
            final mapper = MockContextMapper<OrganizationPolicy, String>();
            when(mapper.map(any, any)).thenReturn('Sample mapped policy string');
            return mapper;
          },
        ),
      ],
    );

    // Scroll to the bottom of the screen
    await tester.fling(find.byType(Scrollable).first, const Offset(0, -1000), 5000);
    await tester.pumpAndSettle();

    // Tap the 'swap card' button
    final l10n = await TestUtils.englishLocalizations;
    await tester.tap(find.text(l10n.checkAttributesScreenChangeCardCta));
    await tester.pumpAndSettle();

    // Verify 'change card sheet' is shown
    await screenMatchesGolden('request_details/select_card_sheet.dark');
  });

  testGoldens('RequestDetailsError Light', (tester) async {
    // Test the _buildError UI with generic error state
    await tester.pumpWidgetWithAppWrapper(
      const IssuanceRequestDetailsScreen().withState<IssuanceBloc, IssuanceState>(
        MockIssuanceBloc(),
        const IssuanceGenericError(
          error: GenericError('An error occurred', sourceError: CoreGenericError('test')),
        ),
      ),
      providers: [
        RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(create: (_) => MockContextMapper()),
      ],
    );
    await screenMatchesGolden('request_details/error.light');
  });
}
