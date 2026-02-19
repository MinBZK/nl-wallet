import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_state.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/domain/usecase/maintenance/impl/observe_maintenance_state_usecase_impl.dart';

import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late MockConfigurationRepository mockConfigurationRepository;

  late ObserveMaintenanceStateUseCaseImpl maintenanceStateUseCase;

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();

    maintenanceStateUseCase = ObserveMaintenanceStateUseCaseImpl(
      mockConfigurationRepository,
    );
  });

  test('emits noMaintenance when configuration has no maintenance window', () async {
    when(
      mockConfigurationRepository.observeAppConfiguration,
    ).thenAnswer(
      (_) => Stream.value(WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: null)),
    );

    final result = await maintenanceStateUseCase.invoke().first;

    expect(result, const MaintenanceState.noMaintenance());
  });

  test('emits inMaintenance when currently in maintenance window', () async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        when(
          mockConfigurationRepository.observeAppConfiguration,
        ).thenAnswer(
          (_) => Stream.value(
            WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: maintenanceWindow),
          ),
        );

        final result = await maintenanceStateUseCase.invoke().first;

        expect(result, MaintenanceState.inMaintenance(maintenanceWindow));
      },
    );
  });

  test('emits noMaintenance when maintenance window exists but is not active', () async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();
        final maintenanceWindow = MaintenanceWindow(
          startDateTime: now.add(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 2)),
        );

        when(
          mockConfigurationRepository.observeAppConfiguration,
        ).thenAnswer(
          (_) => Stream.value(
            WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: maintenanceWindow),
          ),
        );

        final result = await maintenanceStateUseCase.invoke().first;

        expect(result, const MaintenanceState.noMaintenance());
      },
    );
  });

  test('emits state changes when configuration stream emits multiple times', () async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();

        // Active maintenance window
        final activeMaintenanceWindow = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        // No maintenance window
        final MaintenanceWindow? noMaintenanceWindow = null;

        final config1 = WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: activeMaintenanceWindow);
        final config2 = WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: noMaintenanceWindow);
        final config3 = WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: activeMaintenanceWindow);

        when(
          mockConfigurationRepository.observeAppConfiguration,
        ).thenAnswer(
          (_) => Stream.fromIterable([config1, config2, config3]),
        );

        await expectLater(
          maintenanceStateUseCase.invoke(),
          emitsInOrder(
            [
              MaintenanceState.inMaintenance(activeMaintenanceWindow),
              const MaintenanceState.noMaintenance(),
              MaintenanceState.inMaintenance(activeMaintenanceWindow),
            ],
          ),
        );
      },
    );
  });

  test('uses distinct() to prevent duplicate emissions of the same state', () async {
    await withClock(
      Clock.fixed(DateTime(2025, 1, 15, 10, 30)),
      () async {
        final now = clock.now();

        // Same maintenance window emitted multiple times
        final maintenanceWindow1 = MaintenanceWindow(
          startDateTime: now.subtract(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 1)),
        );

        // No maintenance window
        final maintenanceWindow2 = MaintenanceWindow(
          startDateTime: now.add(const Duration(hours: 1)),
          endDateTime: now.add(const Duration(hours: 2)),
        );

        final config1 = WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: maintenanceWindow1);
        final config2 = WalletMockData.flutterAppConfiguration.copyWith(maintenanceWindow: maintenanceWindow2);

        when(
          mockConfigurationRepository.observeAppConfiguration,
        ).thenAnswer(
          // Emit config1 three times, then config2 once
          (_) => Stream.fromIterable([config1, config1, config1, config2]),
        );

        // Because of distinct(), we should only get 2 emissions:
        // - inMaintenance(window1) once (despite config1 being emitted 3 times)
        // - noMaintenance() once (window2 is not active yet)
        await expectLater(
          maintenanceStateUseCase.invoke(),
          emitsInOrder(
            [
              MaintenanceState.inMaintenance(maintenanceWindow1),
              const MaintenanceState.noMaintenance(),
            ],
          ),
        );
      },
    );
  });
}
