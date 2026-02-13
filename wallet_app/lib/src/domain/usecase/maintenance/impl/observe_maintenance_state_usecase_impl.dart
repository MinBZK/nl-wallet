import 'package:rxdart/rxdart.dart';

import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../model/configuration/maintenance_state.dart';
import '../observe_maintenance_state_usecase.dart';

const Duration _kMaintenanceCheckInterval = Duration(minutes: 1);

class ObserveMaintenanceStateUseCaseImpl extends ObserveMaintenanceStateUseCase {
  final ConfigurationRepository _configurationRepository;

  ObserveMaintenanceStateUseCaseImpl(this._configurationRepository);

  @override
  Stream<MaintenanceState> invoke() {
    // Combine configuration changes with periodic checks (every minute)
    // to ensure we emit when maintenance windows start/end naturally.
    // distinct() works efficiently because MaintenanceState is a sealed class:
    // it will only emit when transitioning between inMaintenance/noMaintenance states,
    // or when the maintenance window object itself changes.
    return Rx.combineLatest2(
      _configurationRepository.observeAppConfiguration.map((event) => event.maintenanceWindow),
      Stream.periodic(_kMaintenanceCheckInterval).startWith(null),
      (window, _) {
        if (window == null || !window.isCurrentlyInMaintenance) {
          return const MaintenanceState.noMaintenance();
        }
        return MaintenanceState.inMaintenance(window);
      },
    ).distinct();
  }
}
