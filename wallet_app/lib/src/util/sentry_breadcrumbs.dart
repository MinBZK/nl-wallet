import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet_core/core.dart' as core;

class SentryBreadcrumbs {
  static const int maxBreadcrumbs = 25;

  static const String _flowCategory = 'wallet.flow';
  static const String _nativeCategory = 'wallet.native';

  static final RegExp _messagePattern = RegExp(r'^[a-z0-9_]+(\.[a-z0-9_]+)*$');
  static final Set<String> _allowedCategories = {
    _flowCategory,
    _nativeCategory,
  };

  static Future<void> flow(String message) => _add(_flowCategory, message);

  static Future<void> native(String message) => _add(_nativeCategory, message);

  static Breadcrumb? beforeBreadcrumb(Breadcrumb? breadcrumb, Hint hint) {
    if (breadcrumb == null || !_isCurated(breadcrumb)) return null;

    breadcrumb.data = null;
    breadcrumb.level = SentryLevel.info;
    breadcrumb.type = 'default';
    return breadcrumb;
  }

  static List<Breadcrumb>? filterEventBreadcrumbs(List<Breadcrumb>? breadcrumbs) {
    if (breadcrumbs == null) return null;

    final filtered = breadcrumbs.where(_isCurated).toList();
    if (filtered.isEmpty) return null;

    for (final breadcrumb in filtered) {
      breadcrumb.data = null;
      breadcrumb.level = SentryLevel.info;
      breadcrumb.type = 'default';
    }

    return filtered;
  }

  static Future<void> installRustForwarding() async {
    await core.setSentryBreadcrumbCallback(
      callback: native,
    );
  }

  static Future<void> _add(String category, String message) async {
    if (!_isAllowedMessage(message)) return;

    await Sentry.addBreadcrumb(
      Breadcrumb(
        category: category,
        message: message,
        type: 'default',
      ),
    );
  }

  static bool _isCurated(Breadcrumb breadcrumb) =>
      _allowedCategories.contains(breadcrumb.category) && _isAllowedMessage(breadcrumb.message);

  static bool _isAllowedMessage(String? message) => message != null && _messagePattern.hasMatch(message);
}
