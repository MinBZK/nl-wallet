import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:provider/single_child_widget.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/service/navigation_service.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/usecase/session/cancel_session_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/observe_wallet_locked_usecase.dart';
import 'package:wallet/src/feature/recover_session/recover_session_screen.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mocks.mocks.dart';
import '../../test_util/golden_utils.dart';

void main() {
  late MockCancelSessionUseCase cancelSessionUseCase;
  late MockNavigationService navigationService;
  late MockObserveWalletLockedUseCase observeWalletLockedUseCase;
  late BehaviorSubject<bool> isLockedStream;

  setUp(() {
    cancelSessionUseCase = MockCancelSessionUseCase();
    when(cancelSessionUseCase.invoke()).thenAnswer((_) async => const Result.success(null));

    navigationService = MockNavigationService();
    when(navigationService.hasQueuedRequest).thenReturn(false);
    when(navigationService.processQueue()).thenAnswer((_) async {});

    isLockedStream = BehaviorSubject.seeded(false);
    observeWalletLockedUseCase = MockObserveWalletLockedUseCase();
    when(observeWalletLockedUseCase.invoke()).thenAnswer((_) => isLockedStream);
  });

  List<SingleChildWidget> providers() => [
    RepositoryProvider<CancelSessionUseCase>.value(value: cancelSessionUseCase),
    RepositoryProvider<NavigationService>.value(value: navigationService),
    RepositoryProvider<ObserveWalletLockedUseCase>.value(value: observeWalletLockedUseCase),
  ];

  group('goldens', () {
    testGoldens('RecoverSessionScreen light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );
      await screenMatchesGolden('recover_session.light');
    });

    testGoldens('RecoverSessionScreen dark', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        brightness: Brightness.dark,
        providers: providers(),
      );
      await screenMatchesGolden('recover_session.dark');
    });
  });

  group('widgets', () {
    testWidgets('title and description are shown', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );
      final l10nFinder = find.text('Continue in your browser');
      expect(l10nFinder, findsWidgets);
    });

    testWidgets('pressing stop cancels the session', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );

      await tester.tap(find.text('Stop'));
      await tester.pumpAndSettle();

      verify(cancelSessionUseCase.invoke()).called(1);
    });

    testWidgets('unlocking without a queued request cancels the session', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );

      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();

      verify(cancelSessionUseCase.invoke()).called(1);
      verifyNever(navigationService.processQueue());
    });

    testWidgets('unlocking with a queued request processes the queue instead of cancelling', (tester) async {
      when(navigationService.hasQueuedRequest).thenReturn(true);

      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );

      isLockedStream.add(true);
      await tester.pumpAndSettle();
      isLockedStream.add(false);
      await tester.pumpAndSettle();

      verify(navigationService.processQueue()).called(1);
      verifyNever(cancelSessionUseCase.invoke());
    });

    testWidgets('locking the app does not cancel the session or navigate', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const RecoverSessionScreen(),
        providers: providers(),
      );

      isLockedStream.add(true);
      await tester.pumpAndSettle();

      verifyNever(cancelSessionUseCase.invoke());
      verifyNever(navigationService.processQueue());
    });
  });
}
