import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/feature/common/widget/menu_item.dart';
import 'package:wallet/src/feature/error/error_page.dart';
import 'package:wallet/src/feature/help/bloc/help_overview_bloc.dart';
import 'package:wallet/src/feature/help/help_overview_screen.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

class MockHelpOverviewBloc extends MockBloc<HelpOverviewEvent, HelpOverviewState> implements HelpOverviewBloc {}

const _loadedState = HelpOverviewLoadSuccess(WalletMockData.helpCategories);

Future<void> _pumpOverview(
  WidgetTester tester,
  HelpOverviewState state, {
  Brightness brightness = Brightness.light,
  Size surfaceSize = iphoneXSize,
}) {
  return tester.pumpWidgetWithAppWrapper(
    const HelpOverviewScreen().withState<HelpOverviewBloc, HelpOverviewState>(MockHelpOverviewBloc(), state),
    brightness: brightness,
    surfaceSize: surfaceSize,
  );
}

void main() {
  group('widgets', () {
    testWidgets('renders a MenuItem for every loaded category with title and subtitle', (tester) async {
      await _pumpOverview(tester, _loadedState);

      for (final category in WalletMockData.helpCategories) {
        expect(find.text(category.title), findsOneWidget);
        expect(find.text(category.subtitle), findsOneWidget);
      }
    });

    testWidgets('always renders the tour and contact entries alongside categories', (tester) async {
      await _pumpOverview(tester, _loadedState);
      final l10n = await TestUtils.englishLocalizations;

      expect(find.text(l10n.menuScreenTourCta), findsOneWidget);
      expect(find.text(l10n.contactScreenTitle), findsOneWidget);
    });

    testWidgets('shows only the centered loading indicator while the bloc is loading', (tester) async {
      await _pumpOverview(tester, const HelpOverviewLoadInProgress());

      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
      // Tour + contact are hidden during load (full-body loading like other screens).
      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.menuScreenTourCta), findsNothing);
      expect(find.text(l10n.contactScreenTitle), findsNothing);
    });

    testWidgets('renders an ErrorPage on load failure', (tester) async {
      await _pumpOverview(
        tester,
        const HelpOverviewLoadFailure(GenericError('yaml missing', sourceError: 'yaml missing')),
      );

      // The full-body error page replaces the category list, tour and contact entries.
      expect(find.byType(ErrorPage), findsOneWidget);
      for (final category in WalletMockData.helpCategories) {
        expect(find.text(category.title), findsNothing);
      }
    });

    testWidgets('tapping a category pushes helpCategoryRoute', (tester) async {
      await _pumpOverview(tester, _loadedState);

      await tester.tap(find.text(WalletMockData.helpCategories.first.title));
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.helpCategoryRoute), findsOneWidget);
    });

    testWidgets('tapping the contact row pushes contactRoute', (tester) async {
      await _pumpOverview(tester, _loadedState);
      final l10n = await TestUtils.englishLocalizations;

      await tester.tap(find.text(l10n.contactScreenTitle));
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.contactRoute), findsOneWidget);
    });

    testWidgets('tapping the tour row pushes tourOverviewRoute', (tester) async {
      await _pumpOverview(tester, _loadedState);
      final l10n = await TestUtils.englishLocalizations;

      await tester.tap(find.text(l10n.menuScreenTourCta));
      await tester.pumpAndSettle();

      expect(find.text(WalletRoutes.tourOverviewRoute), findsOneWidget);
    });

    testWidgets('renders one MenuItem per category plus tour and contact', (tester) async {
      await _pumpOverview(tester, _loadedState);

      expect(find.byType(MenuItem), findsNWidgets(WalletMockData.helpCategories.length + 2));
    });
  });

  group('goldens', () {
    testGoldens('HelpOverviewScreen light', (tester) async {
      await _pumpOverview(tester, _loadedState);
      await screenMatchesGolden('overview.light');
    });

    testGoldens('HelpOverviewScreen dark', (tester) async {
      await _pumpOverview(tester, _loadedState, brightness: Brightness.dark);
      await screenMatchesGolden('overview.dark');
    });

    testGoldens('HelpOverviewScreen light - landscape', (tester) async {
      await _pumpOverview(tester, _loadedState, surfaceSize: iphoneXSizeLandscape);
      await screenMatchesGolden('overview.light.landscape');
    });
  });
}
