// This is an example unit test.
//
// A unit test tests a single function, method, or class. To learn more about
// writing unit tests, visit
// https://flutter.dev/docs/cookbook/testing/unit/introduction

import 'package:flutter/cupertino.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/wallet_routes.dart';

void main() {
  group('Route Security', () {
    test('dashboard route should be secured', () {
      const routeSettings = RouteSettings(name: WalletRoutes.dashboardRoute);
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
