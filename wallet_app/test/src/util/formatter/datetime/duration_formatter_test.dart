import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/util/formatter/datetime/duration_formatter.dart';

import '../../../test_util/test_utils.dart';

void main() {
  late AppLocalizations l10n;

  setUpAll(() async {
    l10n = await TestUtils.englishLocalizations;
  });

  group('DurationFormatter', () {
    final now = DateTime(2024, 1, 1, 12);

    group('prettyPrintTimeAgo', () {
      test('should return "less then a minute ago" when time is now', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now),
            'less then a minute ago',
          );
        });
      });

      test('should return "less then a minute ago" when time is in the future', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.add(const Duration(minutes: 5))),
            'less then a minute ago',
          );
        });
      });

      test('should return "less then a minute ago" when time is 30 seconds ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(seconds: 30))),
            'less then a minute ago',
          );
        });
      });

      test('should return "1 minute ago" when time is 1 minute ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(minutes: 1))),
            '1 minute ago',
          );
        });
      });

      test('should return "2 minutes ago" when time is 2 minutes ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(minutes: 2))),
            '2 minutes ago',
          );
        });
      });

      test('should return "1 hour ago" when time is 1 hour ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(hours: 1))),
            '1 hour ago',
          );
        });
      });

      test('should return "2 hours ago" when time is 2 hours ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(hours: 2))),
            '2 hours ago',
          );
        });
      });

      test('should return "1 day ago" when time is 1 day ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(days: 1))),
            '1 day ago',
          );
        });
      });

      test('should return "6 days ago" when time is 6 days ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(days: 6))),
            '6 days ago',
          );
        });
      });

      test('should return "more than a week ago" when time is 7 days ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(days: 7))),
            'more than a week ago',
          );
        });
      });

      test('should return "more than a week ago" when time is 8 days ago', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeAgo(l10n, now.subtract(const Duration(days: 8))),
            'more than a week ago',
          );
        });
      });
    });

    group('prettyPrintTimeDifference', () {
      test('should return "less then a minute" when no difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now),
            'less then a minute',
          );
        });
      });

      test('should return "less then a minute" when 30 seconds difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(seconds: 30))),
            'less then a minute',
          );
        });
      });

      test('should return "1 minute" when 1 minute difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(minutes: 1))),
            '1 minute',
          );
        });
      });

      test('should return "2 minutes" when 2 minutes difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(minutes: 2))),
            '2 minutes',
          );
        });
      });

      test('should return "1 hour" when 1 hour difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(hours: 1))),
            '1 hour',
          );
        });
      });

      test('should return "2 hours" when 2 hours difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(hours: 2))),
            '2 hours',
          );
        });
      });

      test('should return "1 day" when 1 day difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(days: 1))),
            '1 day',
          );
        });
      });

      test('should return "2 days" when 2 days difference', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.add(const Duration(days: 2))),
            '2 days',
          );
        });
      });

      test('should work for past dates as well (absolute difference)', () {
        withClock(Clock.fixed(now), () {
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, now.subtract(const Duration(days: 2))),
            '2 days',
          );
        });
      });

      test('should return "365 days" when 1 year difference', () {
        withClock(Clock.fixed(now), () {
          final past = DateTime(2023, 1, 1, 12);
          expect(
            DurationFormatter.prettyPrintTimeDifference(l10n, past),
            '365 days',
          );
        });
      });
    });
  });
}
