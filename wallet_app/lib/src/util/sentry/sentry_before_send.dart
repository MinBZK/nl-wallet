import 'dart:async';

import 'package:sentry_flutter/sentry_flutter.dart';

import 'sentry_breadcrumbs.dart';

/// `flutter_error_details` (added by `FlutterErrorIntegration` on a render error)
/// is a free-text dump of the widget subtree and can contain personal data. Every
/// other context is vetted SDK telemetry, so this is the only one we drop.
const _freeFormContext = 'flutter_error_details';

/// Scrubs personal data from every outgoing Sentry event; stacktraces are kept.
FutureOr<SentryEvent?> beforeSend(SentryEvent event, Hint hint) async {
  event.user
    ?..geo = null
    ..ipAddress = null;
  event.breadcrumbs = SentryBreadcrumbs.filterEventBreadcrumbs(event.breadcrumbs);
  event.exceptions?.forEach((exception) => exception.value = null);
  event.contexts.remove(_freeFormContext);
  event.request = null;
  event.transaction = null;
  event.message = null;
  // ignore: deprecated_member_use
  event.extra = null;
  return event;
}
