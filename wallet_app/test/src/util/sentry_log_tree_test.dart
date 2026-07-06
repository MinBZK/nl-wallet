import 'dart:async';

import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:test/test.dart';
import 'package:wallet/src/util/sentry_log_tree.dart';

void main() {
  test('registers all Fimber levels', () {
    expect(SentryLogTree(logger: _RecordingSentryLogger()).getLevels(), [
      'V',
      'D',
      'I',
      'W',
      'E',
    ]);
  });

  test('maps Fimber levels to Sentry log levels', () {
    final logger = _RecordingSentryLogger();
    final tree = SentryLogTree(logger: logger);

    tree
      ..log('V', 'verbose')
      ..log('D', 'debug')
      ..log('I', 'info')
      ..log('W', 'warning')
      ..log('E', 'error')
      ..log('?', 'fallback');

    expect(logger.records.map((record) => record.level), [
      SentryLogLevel.trace,
      SentryLogLevel.debug,
      SentryLogLevel.info,
      SentryLogLevel.warn,
      SentryLogLevel.error,
      SentryLogLevel.info,
    ]);
  });

  test('preserves message details and logger tag', () {
    final logger = _RecordingSentryLogger();
    final tree = SentryLogTree(logger: logger);

    tree.log(
      'E',
      'failed',
      tag: 'wallet.core',
      ex: StateError('boom'),
      stacktrace: StackTrace.fromString('stack line'),
    );

    final record = logger.records.single;
    expect(record.body, contains('failed'));
    expect(record.body, contains('Bad state: boom'));
    expect(record.body, contains('stack line'));
    expect(record.attributes?['logger.name']?.value, 'wallet.core');
  });
}

class _LogRecord {
  const _LogRecord(this.level, this.body, this.attributes);

  final SentryLogLevel level;
  final String body;
  final Map<String, SentryAttribute>? attributes;
}

class _RecordingSentryLogger implements SentryLogger {
  final records = <_LogRecord>[];

  @override
  SentryLoggerFormatter get fmt => throw UnimplementedError();

  @override
  FutureOr<void> trace(
    String body, {
    Map<String, SentryAttribute>? attributes,
  }) {
    records.add(_LogRecord(SentryLogLevel.trace, body, attributes));
  }

  @override
  FutureOr<void> debug(
    String body, {
    Map<String, SentryAttribute>? attributes,
  }) {
    records.add(_LogRecord(SentryLogLevel.debug, body, attributes));
  }

  @override
  FutureOr<void> info(String body, {Map<String, SentryAttribute>? attributes}) {
    records.add(_LogRecord(SentryLogLevel.info, body, attributes));
  }

  @override
  FutureOr<void> warn(String body, {Map<String, SentryAttribute>? attributes}) {
    records.add(_LogRecord(SentryLogLevel.warn, body, attributes));
  }

  @override
  FutureOr<void> error(
    String body, {
    Map<String, SentryAttribute>? attributes,
  }) {
    records.add(_LogRecord(SentryLogLevel.error, body, attributes));
  }

  @override
  FutureOr<void> fatal(
    String body, {
    Map<String, SentryAttribute>? attributes,
  }) {
    records.add(_LogRecord(SentryLogLevel.fatal, body, attributes));
  }
}
