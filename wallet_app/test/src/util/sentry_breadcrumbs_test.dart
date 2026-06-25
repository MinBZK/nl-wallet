import 'package:flutter_test/flutter_test.dart';
import 'package:sentry_flutter/sentry_flutter.dart';
import 'package:wallet/src/util/sentry_breadcrumbs.dart';

void main() {
  group('SentryBreadcrumbs', () {
    test('beforeBreadcrumb keeps only curated wallet breadcrumbs and scrubs payload data', () {
      final breadcrumb = Breadcrumb(
        category: 'wallet.flow',
        message: 'route.push.dashboard',
        data: {'route': '/dashboard'},
        level: SentryLevel.warning,
        type: 'navigation',
      );

      final result = SentryBreadcrumbs.beforeBreadcrumb(breadcrumb, Hint());

      expect(result, same(breadcrumb));
      expect(result?.data, isNull);
      expect(result?.level, SentryLevel.info);
      expect(result?.type, 'default');
    });

    test('beforeBreadcrumb drops non-wallet and invalid breadcrumbs', () {
      expect(
        SentryBreadcrumbs.beforeBreadcrumb(Breadcrumb(category: 'http', message: 'request'), Hint()),
        isNull,
      );
      expect(
        SentryBreadcrumbs.beforeBreadcrumb(Breadcrumb(category: 'wallet.flow', message: 'Route push'), Hint()),
        isNull,
      );
      expect(SentryBreadcrumbs.beforeBreadcrumb(null, Hint()), isNull);
    });

    test('filterEventBreadcrumbs keeps only curated wallet breadcrumbs and returns null when none remain', () {
      final breadcrumbs = [
        Breadcrumb(category: 'wallet.native', message: 'rust.error.unexpected', data: {'details': 'sensitive'}),
        Breadcrumb(category: 'wallet.flow', message: 'issuance.fail.start', level: SentryLevel.error),
        Breadcrumb(category: 'wallet.flow', message: 'Issuance.start'),
        Breadcrumb(category: 'http', message: 'request'),
      ];

      final result = SentryBreadcrumbs.filterEventBreadcrumbs(breadcrumbs);

      expect(result, hasLength(2));
      expect(result?.map((breadcrumb) => breadcrumb.message), [
        'rust.error.unexpected',
        'issuance.fail.start',
      ]);
      expect(result?.every((breadcrumb) => breadcrumb.data == null), isTrue);
      expect(result?.every((breadcrumb) => breadcrumb.level == SentryLevel.info), isTrue);
      expect(result?.every((breadcrumb) => breadcrumb.type == 'default'), isTrue);

      expect(SentryBreadcrumbs.filterEventBreadcrumbs([Breadcrumb(category: 'http', message: 'request')]), isNull);
      expect(SentryBreadcrumbs.filterEventBreadcrumbs(null), isNull);
    });
  });
}
