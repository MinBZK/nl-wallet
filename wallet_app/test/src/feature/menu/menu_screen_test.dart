import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/menu/menu_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/test_utils.dart';

void main() {
  testWidgets('expected menu items are visible', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const MenuScreen(showDesignSystemRow: true),
      providers: [
        Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
      ],
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.menuScreenHelpCta), findsOneWidget);
    expect(find.text(l10n.menuScreenScanQrCta), findsOneWidget);
    expect(find.text(l10n.menuScreenHistoryCta), findsOneWidget);
    expect(find.text(l10n.menuScreenSettingsCta), findsOneWidget);
    expect(find.text(l10n.menuScreenFeedbackCta), findsOneWidget);
    expect(find.text(l10n.menuScreenAboutCta), findsOneWidget);
    expect(find.text(l10n.menuScreenDesignCta), findsOneWidget);
  });

  testWidgets('design system menu item is hidden when disabled', (WidgetTester tester) async {
    await tester.pumpWidgetWithAppWrapper(
      const MenuScreen(showDesignSystemRow: false),
      providers: [
        Provider<ObserveWalletLockedUseCase>(create: (_) => MockObserveWalletLockedUseCase()),
      ],
    );

    final l10n = await TestUtils.englishLocalizations;
    expect(find.text(l10n.menuScreenDesignCta), findsNothing);
  });
}
