import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/navigation/secured_page_route.dart';
import 'package:wallet/src/navigation/wallet_routes.dart';

void main() {
  group('WalletRoutes', () {
    test('.initialRoutes should return a list containing the splash route', () {
      final routes = WalletRoutes.initialRoutes('/');
      expect(routes, hasLength(1));
      expect(routes.first, isA<MaterialPageRoute>());
      expect(routes.first, isNot(isA<SecuredPageRoute>()));
    });

    test('.routeFactory should throw UnsupportedError for unknown route', () {
      expect(
        () => WalletRoutes.routeFactory(const RouteSettings(name: '/unknown')),
        throwsA(isA<UnsupportedError>()),
      );
    });

    group('.routeFactory', () {
      final allRoutes = WalletRoutes.allRoutes;

      for (final routeName in allRoutes) {
        testWidgets('should return a valid route for $routeName', (tester) async {
          final settings = RouteSettings(name: routeName);
          final route = WalletRoutes.routeFactory(settings);
          expect(route.settings.name, routeName);

          if (WalletRoutes.publicRoutes.contains(routeName)) {
            expect(route, isNot(isA<SecuredPageRoute>()), reason: '$routeName should be a public route');
            expect(route, isA<MaterialPageRoute>());
          } else {
            expect(route, isA<SecuredPageRoute>(), reason: '$routeName should be a secured route');
          }
        });
      }
    });
  });
}
