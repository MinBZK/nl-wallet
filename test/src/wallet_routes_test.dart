import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_routes.dart';

void main() {
  group('Route Security', () {
    test('home route should be secured', () {
      const routeSettings = RouteSettings(name: WalletRoutes.homeRoute);
      final Route dashboardRoute = WalletRoutes.routeFactory(routeSettings);
      expect(dashboardRoute, isA<SecuredPageRoute>());
    });

    test('public routes should not be secured', () {
      for (var routeName in WalletRoutes.publicRoutes) {
        final Route route = WalletRoutes.routeFactory(RouteSettings(name: routeName));
        expect(route, isNot(isA<SecuredPageRoute>()));
      }
    });
  });
}
