import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/data/service/announcement_service.dart';

import '../../test_util/test_utils.dart';

void main() {
  group('AnnouncementService', () {
    late AppLocalizations l10n;

    setUp(() async {
      l10n = await TestUtils.englishLocalizations;
    });

    testWidgets('announceEnteredDigits does not throw', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: true),
          child: Builder(
            builder: (BuildContext context) {
              final service = AnnouncementService(context);
              service.announceEnteredDigits(l10n, 2);
              return const Placeholder();
            },
          ),
        ),
      );
    });

    testWidgets('announce does not throw', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: true),
          child: Builder(
            builder: (BuildContext context) {
              final service = AnnouncementService(context);
              service.announce('test_announcement');
              return const Placeholder();
            },
          ),
        ),
      );
    });

    testWidgets('announcementsEnabled is true (is fetched through MediaQuery)', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: true),
          child: Builder(
            builder: (BuildContext context) {
              final service = AnnouncementService(context);
              expect(service.announcementsEnabled, isTrue);
              return const Placeholder();
            },
          ),
        ),
      );
    });

    testWidgets('announcementsEnabled is false (is fetched through MediaQuery)', (tester) async {
      await tester.pumpWidget(
        MediaQuery(
          data: const MediaQueryData(accessibleNavigation: false),
          child: Builder(
            builder: (BuildContext context) {
              final service = AnnouncementService(context);
              expect(service.announcementsEnabled, isFalse);
              return const Placeholder();
            },
          ),
        ),
      );
    });
  });
}
