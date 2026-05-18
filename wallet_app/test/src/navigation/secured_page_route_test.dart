import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/wallet/wallet_repository.dart';
import 'package:wallet/src/domain/usecase/app/check_is_app_initialized_usecase.dart';
import 'package:wallet/src/domain/usecase/biometrics/is_biometric_login_enabled_usecase.dart';
import 'package:wallet/src/domain/usecase/pin/unlock_wallet_with_pin_usecase.dart';
import 'package:wallet/src/feature/pin/pin_overlay.dart';
import 'package:wallet/src/feature/pin/pin_screen.dart';
import 'package:wallet/src/navigation/secured_page_route.dart';
import 'package:wallet/src/util/manager/biometric_unlock_manager.dart';

import '../../wallet_app_test_widget.dart';
import '../mocks/wallet_mocks.dart';

void main() {
  late MockWalletRepository mockWalletRepository;
  late MockIsWalletInitializedUseCase mockIsWalletInitializedUseCase;

  late StreamController<bool> lockedStreamController;

  setUp(() {
    SecuredPageRoute.overrideDurationOfNextTransition(null);
    mockWalletRepository = MockWalletRepository();
    mockIsWalletInitializedUseCase = MockIsWalletInitializedUseCase();

    lockedStreamController = StreamController();
    when(mockWalletRepository.isLockedStream).thenAnswer((_) => lockedStreamController.stream);
    when(mockIsWalletInitializedUseCase.invoke()).thenAnswer((_) async => true);
  });

  group('SecuredPageRoute', () {
    test('default transition is platform', () {
      final route = SecuredPageRoute(builder: (context) => Container());
      expect(route.transition, SecuredPageTransition.platform);
    });

    test('transitionDuration returns 500ms for slideInFromBottom', () {
      final route = SecuredPageRoute(
        builder: (context) => Container(),
        transition: SecuredPageTransition.slideInFromBottom,
      );
      expect(route.transitionDuration, const Duration(milliseconds: 500));
    });

    test('overrideDurationOfNextTransition overrides duration once', () {
      const customDuration = Duration(seconds: 1);
      SecuredPageRoute.overrideDurationOfNextTransition(customDuration);

      final route1 = SecuredPageRoute(
        builder: (context) => Container(),
        transition: SecuredPageTransition.slideInFromBottom,
      );
      expect(route1.transitionDuration, customDuration);

      final route2 = SecuredPageRoute(
        builder: (context) => Container(),
        transition: SecuredPageTransition.slideInFromBottom,
      );
      expect(route2.transitionDuration, isNot(customDuration));
    });

    test('overrideDurationOfNextTransition(null) resets override', () {
      const customDuration = Duration(seconds: 1);
      SecuredPageRoute.overrideDurationOfNextTransition(customDuration);
      SecuredPageRoute.overrideDurationOfNextTransition(null);

      final route = SecuredPageRoute(
        builder: (context) => Container(),
        transition: SecuredPageTransition.slideInFromBottom,
      );
      expect(route.transitionDuration, isNot(customDuration));
    });

    testWidgets('wraps content with PinOverlay and reacts to lock-state', (WidgetTester tester) async {
      await tester.pumpWidgetWithAppWrapper(
        Builder(
          builder: (context) {
            return ElevatedButton(
              onPressed: () => Navigator.of(context).push(
                SecuredPageRoute(
                  builder: (context) => const Text('Secured Page'),
                  transition: .slideInFromBottom,
                ),
              ),
              child: const Text('Go'),
            );
          },
        ),
        providers: [
          RepositoryProvider<WalletRepository>.value(value: mockWalletRepository),
          RepositoryProvider<IsWalletInitializedUseCase>.value(value: mockIsWalletInitializedUseCase),
          RepositoryProvider<UnlockWalletWithPinUseCase>.value(value: Mocks.create()),
          RepositoryProvider<IsBiometricLoginEnabledUseCase>.value(value: Mocks.create()),
          RepositoryProvider<BiometricUnlockManager>.value(value: Mocks.create()),
        ],
      );

      lockedStreamController.add(true); // Start out locked.

      await tester.tap(find.text('Go'));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 500));

      expect(find.byType(PinOverlay), findsOneWidget);
      expect(find.byType(PinScreen), findsOneWidget); // Above Secured content
      expect(find.text('Secured Page'), findsOneWidget);

      lockedStreamController.add(false); // Unlock app, PinScreen should disappear
      await tester.pump();
      expect(find.byType(PinOverlay), findsOneWidget);
      expect(find.byType(PinScreen), findsNothing); // No longer above Secured content
      expect(find.text('Secured Page'), findsOneWidget);
    });
  });
}
