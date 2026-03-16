import 'package:flutter/cupertino.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/wallet/is_wallet_initialized_with_pid_usecase.dart';
import 'package:wallet/src/feature/forgot_pin/argument/forgot_pin_screen_argument.dart';
import 'package:wallet/src/feature/forgot_pin/forgot_pin_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';
import '../../test_util/test_utils.dart';

void main() {
  late MockIsWalletInitializedWithPidUseCase mockIsWalletInitializedWithPidUseCase;

  setUp(() {
    mockIsWalletInitializedWithPidUseCase = MockIsWalletInitializedWithPidUseCase();
    when(mockIsWalletInitializedWithPidUseCase.invoke()).thenAnswer((_) async => true);
  });

  group('goldens', () {
    testGoldens('ltc41 forgot pin light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(),
        providers: [
          RepositoryProvider<IsWalletInitializedWithPidUseCase>(create: (_) => mockIsWalletInitializedWithPidUseCase),
        ],
      );
      await screenMatchesGolden('light');
    });

    testGoldens('ltc41 forgot pin light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(useCloseButton: true),
        providers: [
          RepositoryProvider<IsWalletInitializedWithPidUseCase>(create: (_) => mockIsWalletInitializedWithPidUseCase),
        ],
      );
      await screenMatchesGolden('light.close_variant');
    });

    testGoldens('ltc41 forgot pin dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(),
        brightness: Brightness.dark,
        providers: [
          RepositoryProvider<IsWalletInitializedWithPidUseCase>(create: (_) => mockIsWalletInitializedWithPidUseCase),
        ],
      );
      await screenMatchesGolden('dark');
    });
  });

  group('widgets', () {
    testWidgets('ltc41 clear wallet button can be found', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const ForgotPinScreen(),
        providers: [
          RepositoryProvider<IsWalletInitializedWithPidUseCase>(create: (_) => mockIsWalletInitializedWithPidUseCase),
        ],
      );
      final l10n = await TestUtils.englishLocalizations;
      final clearWalletButton = find.text(l10n.forgotPinScreenCta, findRichText: true);
      expect(clearWalletButton, findsOneWidget);
    });
  });

  group('getArgument', () {
    test('returns argument when valid map with useCloseButton is provided as true', () {
      const settings = RouteSettings(arguments: {'useCloseButton': true});
      final result = ForgotPinScreen.getArgument(settings);

      expect(result, const ForgotPinScreenArgument(useCloseButton: true));
    });

    test('returns argument when valid map with useCloseButton is provided as false', () {
      const settings = RouteSettings(arguments: {'useCloseButton': false});
      final result = ForgotPinScreen.getArgument(settings);

      expect(result, const ForgotPinScreenArgument(useCloseButton: false));
    });

    test('returns argument with useCloseButton false as default when no arguments are provided', () {
      const settings = RouteSettings(arguments: <String, dynamic>{});
      final result = ForgotPinScreen.getArgument(settings);

      expect(result, const ForgotPinScreenArgument(useCloseButton: false));
    });

    test('returns ForgotPinScreenArgument(useCloseButton: false) when arguments are null', () {
      const settings = RouteSettings(arguments: null);
      final result = ForgotPinScreen.getArgument(settings);

      expect(result, const ForgotPinScreenArgument(useCloseButton: false));
    });

    test('returns ForgotPinScreenArgument(useCloseButton: false) when reason is invalid', () {
      const settings = RouteSettings(arguments: {'useCloseButton': '_invalid_value_'});
      final result = ForgotPinScreen.getArgument(settings);

      expect(result, const ForgotPinScreenArgument(useCloseButton: false));
    });
  });
}
