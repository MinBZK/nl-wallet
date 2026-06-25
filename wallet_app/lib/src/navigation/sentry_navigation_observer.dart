import 'dart:async';

import 'package:flutter/widgets.dart';

import '../util/sentry_breadcrumbs.dart';
import 'wallet_routes.dart';

class SentryNavigationObserver extends NavigatorObserver {
  @override
  void didPush(Route<dynamic> route, Route<dynamic>? previousRoute) {
    _record('push', route);
  }

  @override
  void didPop(Route<dynamic> route, Route<dynamic>? previousRoute) {
    _record('pop', previousRoute);
  }

  @override
  void didReplace({Route<dynamic>? newRoute, Route<dynamic>? oldRoute}) {
    _record('replace', newRoute);
  }

  void _record(String action, Route<dynamic>? route) {
    final routeName = route?.settings.name;
    if (!WalletRoutes.isKnownRoute(routeName)) return;

    unawaited(SentryBreadcrumbs.flow('route.$action.${_toRouteCode(routeName!)}'));
  }

  static String _toRouteCode(String route) {
    if (route == WalletRoutes.splashRoute) return 'splash';
    return route.substring(1).replaceAll('/', '.');
  }
}
