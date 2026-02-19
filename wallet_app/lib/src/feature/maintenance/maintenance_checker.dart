import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../domain/model/configuration/maintenance_state.dart';
import '../../domain/usecase/maintenance/observe_maintenance_state_usecase.dart';
import '../../domain/usecase/wallet/lock_wallet_usecase.dart';
import '../common/widget/minimal_wallet_app.dart';
import 'maintenance_screen.dart';

/// This widget observes and processes the maintenance state of the app.
/// It intentionally lives above the [WalletApp] widget to make sure
/// no accidental navigation is possible if the app is in maintenance mode.
///
/// Uses a StreamBuilder to react to maintenance state changes.
/// When in maintenance mode, it destroys the child widget tree (which includes
/// the MaterialApp), effectively preventing any navigation. When maintenance
/// ends, the child is shown again.
class MaintenanceChecker extends StatelessWidget {
  final Widget child;

  const MaintenanceChecker({
    required this.child,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return StreamBuilder(
      stream: context.read<ObserveMaintenanceStateUseCase>().invoke(),
      builder: (context, snapshot) {
        final maintenanceState = snapshot.data;
        if (maintenanceState == null) return child;
        return maintenanceState.when(
          inMaintenance: (window) {
            // In maintenance mode: lock wallet & show maintenance screen
            context.read<LockWalletUseCase>().invoke();
            return MinimalWalletApp(
              child: MaintenanceScreen(maintenanceWindow: window),
            );
          },
          noMaintenance: () => child,
        );
      },
    );
  }
}
