import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:wallet/src/feature/issuance/page/issuance_confirm_pin_to_retrieve_cards_page.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  group('goldens', () {
    testGoldens('Confirm page light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinToRetrieveCardsPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
      );
      await screenMatchesGolden('issuance_confirm_pin_to_retrieve_cards/light');
    });

    testGoldens('Confirm page dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinToRetrieveCardsPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('issuance_confirm_pin_to_retrieve_cards/dark');
    });
  });

  group('widgets', () {
    testWidgets('renders the correct title', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        IssuanceConfirmPinToRetrieveCardsPage(
          bloc: PinBloc(Mocks.create()),
          onPinValidated: (_) {},
          onConfirmWithPinFailed: (_, _) {},
        ),
      );

      expect(find.text(l10n.issuanceConfirmPinPageTitle, findRichText: true), findsOneWidget);
    });
  });
}
