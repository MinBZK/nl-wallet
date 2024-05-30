import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_screen.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../wallet_app_test_widget.dart';
import '../../util/test_utils.dart';
import 'pin_page_test.dart';

void main() {
  group('widgets', () {
    testWidgets('PinScreen shows the correct title for PinEntryInProgress state', (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: PinScreen(
            onUnlock: (returnUrl) {},
          ).withState<PinBloc, PinState>(
            MockPinBloc(),
            const PinEntryInProgress(0),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.pinScreenHeader);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('PinScreen shows the no internet error for PinValidateNetworkError(hasInternet=false) state',
        (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: PinScreen(
            onUnlock: (returnUrl) {},
          ).withState<PinBloc, PinState>(
            MockPinBloc(),
            const PinValidateNetworkError(
              error: CoreNetworkError('no internet'),
              hasInternet: false,
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'no internet' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenNoInternetHeadline);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);
    });

    testWidgets('PinScreen shows the generic error for PinValidateGenericError state', (tester) async {
      await tester.pumpWidget(
        WalletAppTestWidget(
          child: PinScreen(
            onUnlock: (returnUrl) {},
          ).withState<PinBloc, PinState>(
            MockPinBloc(),
            const PinValidateGenericError(
              error: CoreGenericError('generic'),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry);
      expect(tryAgainCtaFinder, findsOneWidget);
    });
  });
}
