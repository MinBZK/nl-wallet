import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/sign/page/sign_confirm_pin_page.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('SignConfirmPinPage light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        SignConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
        ),
      );
      await screenMatchesGolden('sign_confirm_pin/light');
    });

    testGoldens('SignConfirmPinPage dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        SignConfirmPinPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('sign_confirm_pin/dark');
    });
  });

  group('widgets', () {
    testWidgets('SignConfirmPinPage renders the correct title & subtitle', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        SignConfirmPinPage(
          onPinValidated: (_) {},
          bloc: PinBloc(Mocks.create()),
        ),
      );

      // Setup finders
      final titleFinder = find.text(l10n.signConfirmPinPageTitle);
      final descriptionFinder = find.text(l10n.signConfirmPinPageDescription);

      // Verify all expected widgets show up once
      expect(titleFinder, findsOneWidget);
      expect(descriptionFinder, findsOneWidget);
    });
  });
}
