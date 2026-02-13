import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:provider/provider.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_state.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/domain/usecase/maintenance/observe_maintenance_state_usecase.dart';
import 'package:wallet/src/domain/usecase/wallet/lock_wallet_usecase.dart';
import 'package:wallet/src/feature/maintenance/maintenance_checker.dart';
import 'package:wallet/src/feature/maintenance/maintenance_screen.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  late MockObserveMaintenanceStateUseCase mockObserveMaintenanceStateUseCase;
  late MockLockWalletUseCase mockLockWalletUseCase;

  setUp(() {
    mockObserveMaintenanceStateUseCase = MockObserveMaintenanceStateUseCase();
    mockLockWalletUseCase = MockLockWalletUseCase();
  });

  tearDown(() {
    reset(mockObserveMaintenanceStateUseCase);
    reset(mockLockWalletUseCase);
  });

  Widget buildTestWidget({
    MaintenanceState? maintenanceState,
    Stream<MaintenanceState>? stream,
    required Widget child,
  }) {
    // Set up the mock with either a custom stream or a static value
    if (stream != null) {
      when(mockObserveMaintenanceStateUseCase.invoke()).thenAnswer((_) => stream);
    } else {
      when(mockObserveMaintenanceStateUseCase.invoke()).thenAnswer(
        (_) => Stream.value(maintenanceState ?? const MaintenanceState.noMaintenance()),
      );
    }

    // Mock LockWalletUseCase to do nothing
    when(mockLockWalletUseCase.invoke()).thenAnswer((_) async {});

    // Wrap child in MaterialApp (not MaintenanceChecker) to avoid nested MaterialApps
    // When in maintenance: MaintenanceChecker returns MinimalWalletApp (ignores child)
    // When not in maintenance: MaintenanceChecker returns child (needs MaterialApp)
    return MultiProvider(
      providers: [
        Provider<ObserveMaintenanceStateUseCase>.value(value: mockObserveMaintenanceStateUseCase),
        Provider<LockWalletUseCase>.value(value: mockLockWalletUseCase),
      ],
      child: MaintenanceChecker(
        child: MaterialApp(home: child),
      ),
    );
  }

  testWidgets('shows child widget when not in maintenance', (tester) async {
    await tester.pumpWidget(
      buildTestWidget(
        maintenanceState: const MaintenanceState.noMaintenance(),
        child: const Text('Child Widget'),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.text('Child Widget'), findsOneWidget);
    expect(find.byType(MaintenanceScreen), findsNothing);
  });

  testWidgets('shows maintenance screen when in maintenance window', (tester) async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        await tester.pumpWidget(
          buildTestWidget(
            maintenanceState: MaintenanceState.inMaintenance(maintenanceWindow),
            child: const Text('Child Widget'),
          ),
        );

        await tester.pumpAndSettle();

        expect(find.byType(MaintenanceScreen), findsOneWidget);
        expect(find.text('Child Widget'), findsNothing);
      },
    );
  });

  testWidgets('shows maintenance screen when maintenance window starts now', (tester) async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now,
          endDateTime: now.add(const Duration(hours: 2)),
        );

        await tester.pumpWidget(
          buildTestWidget(
            maintenanceState: MaintenanceState.inMaintenance(maintenanceWindow),
            child: const Text('Child Widget'),
          ),
        );

        await tester.pumpAndSettle();

        expect(find.byType(MaintenanceScreen), findsOneWidget);
      },
    );
  });

  testWidgets('shows child when not in maintenance state', (tester) async {
    await tester.pumpWidget(
      buildTestWidget(
        maintenanceState: const MaintenanceState.noMaintenance(),
        child: const Text('Child Widget'),
      ),
    );

    await tester.pumpAndSettle();

    expect(find.byType(MaintenanceScreen), findsNothing);
    expect(find.text('Child Widget'), findsOneWidget);
  });

  testWidgets('transitions from maintenance to child on stream update', (tester) async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        final subject = BehaviorSubject<MaintenanceState>.seeded(
          MaintenanceState.inMaintenance(maintenanceWindow),
        );

        await tester.pumpWidget(
          buildTestWidget(
            stream: subject.stream,
            child: const Text('Child Widget'),
          ),
        );

        await tester.pumpAndSettle();
        expect(find.byType(MaintenanceScreen), findsOneWidget);

        subject.add(const MaintenanceState.noMaintenance());
        await tester.pumpAndSettle();
        expect(find.text('Child Widget'), findsOneWidget);

        await subject.close();
      },
    );
  });

  testWidgets('locks wallet when entering maintenance mode', (tester) async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        await tester.pumpWidget(
          buildTestWidget(
            maintenanceState: MaintenanceState.inMaintenance(maintenanceWindow),
            child: const Text('Child Widget'),
          ),
        );

        await tester.pumpAndSettle();

        // Verify wallet was locked when maintenance screen is shown
        verify(mockLockWalletUseCase.invoke()).called(1);
      },
    );
  });
}
