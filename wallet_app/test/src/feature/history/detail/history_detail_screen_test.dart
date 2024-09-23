import 'dart:ui';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/policy/organization_policy.dart';
import 'package:wallet/src/feature/history/detail/bloc/history_detail_bloc.dart';
import 'package:wallet/src/feature/history/detail/history_detail_screen.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_disclose_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_issue_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_login_page.dart';
import 'package:wallet/src/feature/history/detail/widget/page/history_detail_sign_page.dart';
import 'package:wallet/src/util/mapper/context_mapper.dart';
import 'package:wallet/src/util/mapper/policy/policy_body_text_mapper.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

class MockHistoryDetailBloc extends MockBloc<HistoryDetailEvent, HistoryDetailState> implements HistoryDetailBloc {}

void main() {
  group('goldens', () {
    testGoldens('HistoryDetailLoadSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
              create: (c) => PolicyBodyTextMapper(),
            ),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('HistoryDetailLoadSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            ),
          ),
        wrapper: walletAppWrapper(
          brightness: Brightness.dark,
          providers: [
            RepositoryProvider<ContextMapper<OrganizationPolicy, String>>(
              create: (c) => PolicyBodyTextMapper(),
            ),
          ],
        ),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('HistoryDetailLoadInProgress light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadInProgress(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('Error state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          const HistoryDetailLoadFailure(),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('Disclose event is rendered with DisclosePage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.disclosureEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.byType(HistoryDetailDisclosePage), findsOneWidget);
    });

    testWidgets('Issuance event is rendered with IssuePage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen().withState<HistoryDetailBloc, HistoryDetailState>(
          MockHistoryDetailBloc(),
          HistoryDetailLoadSuccess(WalletMockData.issuanceEvent, [WalletMockData.card]),
        ),
      );

      expect(find.byType(HistoryDetailIssuePage), findsOneWidget);
    });

    testWidgets('Login event is rendered with LoginPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.loginEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.byType(HistoryDetailLoginPage), findsOneWidget);
    });

    testWidgets('Sign event is rendered with SignPage', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              HistoryDetailLoadSuccess(WalletMockData.signEvent, [WalletMockData.card]),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      expect(find.byType(HistoryDetailSignPage), findsOneWidget);
    });

    testWidgets('Error screen is rendered for HistoryDetailLoadFailure', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const HistoryDetailScreen()
            .withState<HistoryDetailBloc, HistoryDetailState>(
              MockHistoryDetailBloc(),
              const HistoryDetailLoadFailure(),
            )
            .withDependency<ContextMapper<OrganizationPolicy, String>>((context) => PolicyBodyTextMapper()),
      );

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.historyDetailScreenTitle), findsAtLeast(1));
      expect(find.text(l10n.errorScreenGenericDescription), findsOneWidget);
      expect(find.text(l10n.generalRetry), findsOneWidget); // CTA to retry is visible
    });
  });
}
