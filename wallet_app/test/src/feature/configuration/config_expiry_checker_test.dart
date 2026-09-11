import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:provider/provider.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/usecase/configuration/observe_configuration_expired_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/lock_wallet_usecase.dart';
import 'package:wallet/src/feature/configuration/config_expiry_checker.dart';
import 'package:wallet/src/feature/error/error_page.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockObserveConfigurationExpiredUseCase mockObserveConfigurationExpiredUseCase;
  late MockLockWalletUseCase mockLockWalletUseCase;

  setUp(() {
    mockObserveConfigurationExpiredUseCase = MockObserveConfigurationExpiredUseCase();
    mockLockWalletUseCase = MockLockWalletUseCase();
  });

  tearDown(() {
    reset(mockObserveConfigurationExpiredUseCase);
    reset(mockLockWalletUseCase);
  });

  Widget buildTestWidget({
    bool? isExpired,
    Stream<bool>? stream,
    required Widget child,
  }) {
    if (stream != null) {
      when(mockObserveConfigurationExpiredUseCase.invoke()).thenAnswer((_) => stream);
    } else {
      when(mockObserveConfigurationExpiredUseCase.invoke()).thenAnswer((_) => Stream.value(isExpired ?? false));
    }

    // Wrap child in MaterialApp (not ConfigExpiryChecker) to avoid nested MaterialApps.
    // When expired: ConfigExpiryChecker returns MinimalWalletApp (ignores child).
    // When not expired: ConfigExpiryChecker returns child (needs MaterialApp).
    return MultiProvider(
      providers: [
        Provider<ObserveConfigurationExpiredUseCase>.value(value: mockObserveConfigurationExpiredUseCase),
        Provider<LockWalletUseCase>.value(value: mockLockWalletUseCase),
      ],
      child: ConfigExpiryChecker(
        child: MaterialApp(home: child),
      ),
    );
  }

  testWidgets('shows child widget when config is not expired', (tester) async {
    await tester.pumpWidget(
      buildTestWidget(
        isExpired: false,
        child: const Text('Child Widget'),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('Child Widget'), findsOneWidget);
    expect(find.byType(ErrorPage), findsNothing);
  });

  testWidgets('shows blocking error page and locks the wallet when config is expired', (tester) async {
    await tester.pumpWidget(
      buildTestWidget(
        isExpired: true,
        child: const Text('Child Widget'),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.byType(ErrorPage), findsOneWidget);
    expect(find.text('Child Widget'), findsNothing);
    verify(mockLockWalletUseCase.invoke()).called(1);
  });

  testWidgets('transitions from blocked to child on stream update', (tester) async {
    final subject = BehaviorSubject<bool>.seeded(true);

    await tester.pumpWidget(
      buildTestWidget(
        stream: subject.stream,
        child: const Text('Child Widget'),
      ),
    );

    await tester.pumpAndSettle();
    expect(find.byType(ErrorPage), findsOneWidget);
    verify(mockLockWalletUseCase.invoke()).called(1);

    subject.add(false);
    await tester.pumpAndSettle();
    expect(find.text('Child Widget'), findsOneWidget);

    subject.add(true);
    await tester.pumpAndSettle();
    expect(find.byType(ErrorPage), findsOneWidget);
    verify(mockLockWalletUseCase.invoke()).called(1);

    await subject.close();
  });
}
