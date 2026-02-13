import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/configuration/maintenance_window.dart';
import 'package:wallet/src/util/mapper/configuration/maintenance_window_mapper.dart';
import 'package:wallet/src/util/mapper/mapper.dart';

void main() {
  late Mapper<(String, String)?, MaintenanceWindow?> mapper;

  setUp(() {
    mapper = MaintenanceWindowMapper();
  });

  group('map', () {
    test('maps valid tuple with RFC 3339 datetime strings to MaintenanceWindow', () {
      final startDateTime = '2025-01-15T10:00:00Z';
      final endDateTime = '2025-01-15T11:00:00Z';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNotNull);
      expect(result?.startDateTime, DateTime.parse(startDateTime));
      expect(result?.endDateTime, DateTime.parse(endDateTime));
    });

    test('maps valid tuple with RFC 3339 datetime strings (with timezone) to MaintenanceWindow', () {
      final startDateTime = '2025-01-15T10:00:00+01:00';
      final endDateTime = '2025-01-15T11:00:00+01:00';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNotNull);
      expect(result?.startDateTime, DateTime.parse(startDateTime));
      expect(result?.endDateTime, DateTime.parse(endDateTime));
    });

    test('maps null input to null', () {
      final result = mapper.map(null);

      expect(result, isNull);
    });

    test('returns null when start datetime parsing fails', () {
      final startDateTime = 'invalid-date';
      final endDateTime = '2025-01-15T11:00:00Z';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNull);
    });

    test('returns null when end datetime parsing fails', () {
      final startDateTime = '2025-01-15T10:00:00Z';
      final endDateTime = 'invalid-date';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNull);
    });

    test('returns null when maintenance window is invalid (start after end)', () {
      final startDateTime = '2025-01-15T11:00:00Z';
      final endDateTime = '2025-01-15T10:00:00Z';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNull);
    });

    test('returns null when start equals end', () {
      final dateTime = '2025-01-15T10:00:00Z';
      final input = (dateTime, dateTime);

      final result = mapper.map(input);

      expect(result, isNull);
    });

    test('maps multi-day maintenance window correctly', () {
      final startDateTime = '2025-01-15T22:00:00Z';
      final endDateTime = '2025-01-16T04:00:00Z';
      final input = (startDateTime, endDateTime);

      final result = mapper.map(input);

      expect(result, isNotNull);
      expect(result?.startDateTime, DateTime.parse(startDateTime));
      expect(result?.endDateTime, DateTime.parse(endDateTime));
      expect(result?.isValid, isTrue);
    });
  });
}
