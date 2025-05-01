import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/info/info_screen.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  group('widgets', () {
    testWidgets('provided title and description are shown', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(const InfoScreen(title: 'title', description: 'description'));

      expect(find.text('title'), findsAtLeast(1));
      expect(find.text('description'), findsOneWidget);
    });

    testWidgets('showDetailsIncorrect shows the expected copy', (tester) async {
      final btnKey = ValueKey('btn');
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return TextButton(
              onPressed: () => InfoScreen.showDetailsIncorrect(context),
              child: Text('btn', key: btnKey),
            );
          },
        ),
        providers: [
          RepositoryProvider<WalletRepository>(
            create: (c) {
              final mock = MockWalletRepository();
              when(mock.isLockedStream).thenAnswer((_) => Stream.value(false));
              return mock;
            },
          ),
          RepositoryProvider<IsWalletInitializedUseCase>(create: (c) => MockIsWalletInitializedUseCase()),
          RepositoryProvider<IsBiometricLoginEnabledUseCase>(create: (c) => MockIsBiometricLoginEnabledUseCase()),
          RepositoryProvider<BiometricUnlockManager>(create: (c) => MockBiometricUnlockManager()),
          RepositoryProvider<UnlockWalletWithPinUseCase>(create: (c) => MockUnlockWalletWithPinUseCase()),
        ],
      );
      await tester.tap(find.byKey(btnKey));
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      expect(find.text(l10n.detailsIncorrectScreenTitle), findsAtLeast(1));
      l10n.detailsIncorrectScreenDescription.split('\n\n').forEach((paragraph) {
        expect(find.text(paragraph), findsOneWidget);
      });
    });
  });

  group('goldens', () {
    testGoldens('InfoScreen', (tester) async {
      final l10n = await TestUtils.englishLocalizations;
      await tester.pumpWidgetWithAppWrapper(
        InfoScreen(
          title: l10n.detailsIncorrectScreenTitle,
          description: l10n.detailsIncorrectScreenDescription,
        ),
      );
      await screenMatchesGolden('info_screen');
    });
  });
}
