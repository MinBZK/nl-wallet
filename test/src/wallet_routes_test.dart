import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_routes.dart';

void main() {
  test('.homeRoute should be a `SecuredPageRoute`', () {
    const routeSettings = RouteSettings(name: WalletRoutes.homeRoute);
    final Route homeRoute = WalletRoutes.routeFactory(routeSettings);
    expect(homeRoute, isA<SecuredPageRoute>());
  });

  test('.publicRoutes should not be a `SecuredPageRoute`', () {
    for (var routeName in WalletRoutes.publicRoutes) {
      final Route route = WalletRoutes.routeFactory(RouteSettings(name: routeName));
      expect(route, isNot(isA<SecuredPageRoute>()));
    }
  });
}
