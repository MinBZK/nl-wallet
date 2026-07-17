import 'package:flutter_test/flutter_test.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet/src/util/sentry/sentry_before_send.dart';

/// Builds a fresh event resembling what the (still active) `FlutterErrorIntegration`
/// produces for a render/build error on a screen that shows personal data, e.g.
/// the card detail screen. The context set mirrors a real Flutter error event:
/// only `flutter_error_details` is free-form, the rest is vetted SDK telemetry.
/// [beforeSend] mutates in place, so each test gets its own event.
SentryEvent buildRenderErrorEvent() {
  final contexts =
      Contexts(
          device: SentryDevice(name: 'iPhone'),
          operatingSystem: SentryOperatingSystem(name: 'iOS'),
          app: SentryApp(name: 'wallet'),
          culture: SentryCulture(locale: 'nl_NL'),
          // `trace` is a typed context key, so it must hold a SentryTraceContext: a raw
          // map passes in memory but makes Contexts.toJson throw while serialising.
          trace: SentryTraceContext(operation: 'ui.load'),
        )
        ..['flutter'] = <String, String>{'name': 'Flutter', 'type': 'runtime'}
        ..['accessibility'] = <String, bool>{'bold_text': false}
        ..['flutter_error_details'] = <String, String>{
          'context': 'thrown building CardDetailScreen(dirty)',
          // The information collector can dump widget diagnostics containing rendered
          // attribute values — exactly the personal data we must not send.
          'information': "Text('BSN 1234.56.789')\nText('Anna de Vries')",
        };

  final event = SentryEvent(
    throwable: StateError('boom'),
    level: SentryLevel.error,
    exceptions: [
      SentryException(type: 'RenderFlexOverflowError', value: 'A RenderFlex overflowed by 42 pixels'),
      SentryException(type: '_CardAttributeRow', value: 'value: Anna de Vries, BSN 1234.56.789'),
    ],
    message: SentryMessage('rendering failed for Anna de Vries'),
    transaction: '/card/detail',
    request: SentryRequest(url: 'https://issuer.example/pid?token=secret'),
    contexts: contexts,
    user: SentryUser(
      ipAddress: '82.14.203.5',
      geo: SentryGeo(city: 'Utrecht', countryCode: 'NL'),
    ),
    breadcrumbs: [
      Breadcrumb(category: 'wallet.flow', message: 'card.detail.open'),
      Breadcrumb(category: 'http', message: 'GET /pid', data: {'body': 'BSN 1234.56.789'}),
    ],
  );
  // ignore: deprecated_member_use
  event.extra = {'attribute': 'BSN 1234.56.789'};
  return event;
}

void main() {
  // Guards the fixture's own claim of resembling a real event: every other test
  // here asserts in memory, so a context that cannot be serialised would go
  // unnoticed while breaking any event that is actually sent.
  test('fixture serialises the way a real outgoing event would', () {
    expect(() => buildRenderErrorEvent().toJson(), returnsNormally);
  });

  group('beforeSend', () {
    test('removes the free-form flutter_error_details context (the render-time leak)', () async {
      final result = (await beforeSend(buildRenderErrorEvent(), Hint()))!;

      expect(result.contexts.containsKey('flutter_error_details'), isFalse);
    });

    test('keeps vetted SDK contexts and does not over-strip', () async {
      final result = (await beforeSend(buildRenderErrorEvent(), Hint()))!;

      // Structured, non-personal telemetry must be retained for diagnostics.
      expect(result.contexts.containsKey('device'), isTrue);
      expect(result.contexts.containsKey('os'), isTrue);
      expect(result.contexts.containsKey('app'), isTrue);
      expect(result.contexts.containsKey('culture'), isTrue);
      expect(result.contexts.containsKey('flutter'), isTrue);
      expect(result.contexts.containsKey('trace'), isTrue);
      expect(result.contexts.containsKey('accessibility'), isTrue);
    });

    test('nulls exception values, message, transaction, request and extra', () async {
      final result = (await beforeSend(buildRenderErrorEvent(), Hint()))!;

      expect(result.exceptions, isNotNull);
      for (final exception in result.exceptions!) {
        expect(exception.value, isNull);
      }
      expect(result.message, isNull);
      expect(result.transaction, isNull);
      expect(result.request, isNull);
      // ignore: deprecated_member_use
      expect(result.extra, isNull);
    });

    test('scrubs user ip/geo and keeps only curated wallet breadcrumbs', () async {
      final result = (await beforeSend(buildRenderErrorEvent(), Hint()))!;

      expect(result.user?.ipAddress, isNull);
      expect(result.user?.geo, isNull);

      expect(result.breadcrumbs, hasLength(1));
      final breadcrumb = result.breadcrumbs!.single;
      expect(breadcrumb.category, 'wallet.flow');
      expect(breadcrumb.message, 'card.detail.open');
      expect(breadcrumb.data, isNull);
    });
  });
}
