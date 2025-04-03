import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/wallet/personalize/page/wallet_personalize_confirm_pin_page.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.dart';
import '../../../../test_util/golden_utils.dart';
import '../../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('WalletPersonalizeConfirmPinPage light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPidAccepted: (_) {},
          onAcceptPidFailed: (context, state) {},
        ),
      );
      await screenMatchesGolden('wallet_personalize_confirm_pin/light');
    });

    testGoldens('WalletPersonalizeConfirmPinPage dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPidAccepted: (_) {},
          onAcceptPidFailed: (context, state) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('wallet_personalize_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('WalletPersonalizeConfirmPinPage renders the correct title', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        WalletPersonalizeConfirmPinPage(
          onPidAccepted: (_) {},
          onAcceptPidFailed: (context, state) {},
          bloc: PinBloc(Mocks.create()),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.walletPersonalizeConfirmPinPageTitle, findRichText: true);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
    });
  });
}
