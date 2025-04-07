import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/feature/pin/bloc/pin_bloc.dart';
import 'package:wallet/src/feature/pin/pin_screen.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/test_utils.dart';
import 'pin_page_test.dart';

void main() {
  group('widgets', () {
    testWidgets('PinScreen shows the correct title for PinEntryInProgress state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinScreen(
          onUnlock: () {},
        ).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinEntryInProgress(0),
        ),
        providers: [
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
        ],
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the title is shown
      final titleFinder = find.text(l10n.pinScreenHeader, findRichText: true);
      expect(titleFinder, findsOneWidget);
    });

    testWidgets('PinScreen shows the server error for PinValidateNetworkError(hasInternet=true) state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinScreen(
          onUnlock: () {},
        ).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateNetworkError(
            error: NetworkError(hasInternet: true, sourceError: 'test'),
            hasInternet: true,
          ),
        ),
        providers: [
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
        ],
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'server error' title is shown
      final noInternetHeadlineFinder = find.text(l10n.errorScreenServerHeadline, findRichText: true);
      expect(noInternetHeadlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry, findRichText: true);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });

    testWidgets('PinScreen shows the generic error for PinValidateGenericError state', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        PinScreen(
          onUnlock: () {},
        ).withState<PinBloc, PinState>(
          MockPinBloc(),
          const PinValidateGenericError(
            error: GenericError('generic', sourceError: 'test'),
          ),
        ),
        providers: [
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
        ],
      );

      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;

      // Verify the 'something went wrong' title is shown
      final headlineFinder = find.text(l10n.errorScreenGenericHeadline, findRichText: true);
      expect(headlineFinder, findsAtLeastNWidgets(1));

      // Verify the 'try again' cta is shown
      final tryAgainCtaFinder = find.text(l10n.generalRetry, findRichText: true);
      expect(tryAgainCtaFinder, findsOneWidget);

      // Verify the 'show details' cta is shown
      final showDetailsCtaFinder = find.text(l10n.generalShowDetailsCta);
      expect(showDetailsCtaFinder, findsOneWidget);
    });
  });
}
