import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/navigation/secured_page_route.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

void main() {
  test('.dashboardRoute should be a `SecuredPageRoute`', () {
    const routeSettings = RouteSettings(name: WalletRoutes.dashboardRoute);
    final Route homeRoute = WalletRoutes.routeFactory(routeSettings);
    expect(homeRoute, isA<SecuredPageRoute>());
  });

  test('.publicRoutes should not be a `SecuredPageRoute`', () {
    for (final routeName in WalletRoutes.publicRoutes) {
      final Route route = WalletRoutes.routeFactory(RouteSettings(name: routeName));
      expect(route, isNot(isA<SecuredPageRoute>()));
    }
  });
}
