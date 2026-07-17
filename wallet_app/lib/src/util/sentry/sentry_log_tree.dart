import 'dart:async';

import 'package:fimber/fimber.dart';
import 'package:sentry_flutter/sentry_flutter.dart';

class SentryLogTree extends LogTree {
  SentryLogTree({SentryLogger? logger}) : _logger = logger ?? Sentry.logger;

  static const _levels = ['V', 'D', 'I', 'W', 'E'];

  final SentryLogger _logger;

  @override
  List<String> getLevels() => _levels;

  @override
  void log(
    String level,
    String message, {
    String? tag,
    dynamic ex,
    StackTrace? stacktrace,
  }) {
    final body = _formatBody(message, ex, stacktrace);
    final attributes = tag == null ? null : {'logger.name': SentryAttribute.string(tag)};

    final result = switch (level) {
      'V' => _logger.trace(body, attributes: attributes),
      'D' => _logger.debug(body, attributes: attributes),
      'I' => _logger.info(body, attributes: attributes),
      'W' => _logger.warn(body, attributes: attributes),
      'E' => _logger.error(body, attributes: attributes),
      _ => _logger.warn('Unexpected log level "$level": $body', attributes: attributes),
    };

    if (result is Future<void>) unawaited(result);
  }

  String _formatBody(String message, dynamic ex, StackTrace? stacktrace) {
    final body = StringBuffer(message);
    if (ex != null) body.write('\n$ex');
    if (stacktrace != null) body.write('\n$stacktrace');
    return body.toString();
  }
}
