import 'package:flutter/cupertino.dart';

import '../../navigation/wallet_routes.dart';

extension NavigatorStateExtension<T> on NavigatorState {
  Future<T?> resetToDashboard() {
    return pushNamedAndRemoveUntil(
      WalletRoutes.dashboardRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }

  Future<T?> resetToSplash() {
    return pushNamedAndRemoveUntil(
      WalletRoutes.splashRoute,
      ModalRoute.withName(WalletRoutes.splashRoute),
    );
  }
}
