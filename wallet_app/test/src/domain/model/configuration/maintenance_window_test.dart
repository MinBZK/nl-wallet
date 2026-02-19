import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';

void main() {
  group('isCurrentlyInMaintenance', () {
    test('returns true when current time is within maintenance window', () {
      final fixedTime = DateTime.utc(2025, 1, 15, 10, 30);
      final startTime = DateTime.utc(2025, 1, 15, 10, 0);
      final endTime = DateTime.utc(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isTrue);
    });

    test('returns false when current time is before maintenance window', () {
      final fixedTime = DateTime.utc(2025, 1, 15, 9, 30);
      final startTime = DateTime.utc(2025, 1, 15, 10, 0);
      final endTime = DateTime.utc(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isFalse);
    });

    test('returns false when current time is after maintenance window', () {
      final fixedTime = DateTime.utc(2025, 1, 15, 11, 30);
      final startTime = DateTime.utc(2025, 1, 15, 10, 0);
      final endTime = DateTime.utc(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isFalse);
    });

    test('returns true when current time equals start time', () {
      final fixedTime = DateTime.utc(2025, 1, 15, 10, 0);
      final startTime = DateTime.utc(2025, 1, 15, 10, 0);
      final endTime = DateTime.utc(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isTrue);
    });

    test('returns false when current time equals end time (exclusive)', () {
      final fixedTime = DateTime.utc(2025, 1, 15, 11, 0);
      final startTime = DateTime.utc(2025, 1, 15, 10, 0);
      final endTime = DateTime.utc(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isFalse);
    });

    test('handles timezone-aware datetime correctly', () {
      final fixedTime = DateTime(2025, 1, 15, 10, 30).toUtc();
      final startTime = DateTime(2025, 1, 15, 10, 0).toUtc();
      final endTime = DateTime(2025, 1, 15, 11, 0).toUtc();

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final result = withClock(Clock.fixed(fixedTime), () {
        return window.isCurrentlyInMaintenance;
      });

      expect(result, isTrue);
    });
  });

  group('isValid', () {
    test('returns true when start is before end', () {
      final startTime = DateTime(2025, 1, 15, 10, 0);
      final endTime = DateTime(2025, 1, 15, 11, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      expect(window.isValid, isTrue);
    });

    test('returns false when start equals end', () {
      final time = DateTime(2025, 1, 15, 10, 0);

      final window = MaintenanceWindow(
        startDateTime: time,
        endDateTime: time,
      );

      expect(window.isValid, isFalse);
    });

    test('returns false when start is after end', () {
      final startTime = DateTime(2025, 1, 15, 11, 0);
      final endTime = DateTime(2025, 1, 15, 10, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      expect(window.isValid, isFalse);
    });

    test('returns true for multi-day maintenance window', () {
      final startTime = DateTime(2025, 1, 15, 22, 0);
      final endTime = DateTime(2025, 1, 16, 4, 0);

      final window = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      expect(window.isValid, isTrue);
    });
  });

  group('equality', () {
    test('two windows with same times are equal', () {
      final startTime = DateTime(2025, 1, 15, 10, 0);
      final endTime = DateTime(2025, 1, 15, 11, 0);

      final window1 = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      final window2 = MaintenanceWindow(
        startDateTime: startTime,
        endDateTime: endTime,
      );

      expect(window1, equals(window2));
    });

    test('two windows with different times are not equal', () {
      final startTime1 = DateTime(2025, 1, 15, 10, 0);
      final endTime1 = DateTime(2025, 1, 15, 11, 0);
      final startTime2 = DateTime(2025, 1, 15, 10, 30);
      final endTime2 = DateTime(2025, 1, 15, 11, 30);

      final window1 = MaintenanceWindow(
        startDateTime: startTime1,
        endDateTime: endTime1,
      );

      final window2 = MaintenanceWindow(
        startDateTime: startTime2,
        endDateTime: endTime2,
      );

      expect(window1, isNot(equals(window2)));
    });
  });
}
