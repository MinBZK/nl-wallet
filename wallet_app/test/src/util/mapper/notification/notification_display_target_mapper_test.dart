import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/util/mapper/notification/notification_display_target_mapper.dart';
import 'package:wallet_core/core.dart' as core;

void main() {
  late NotificationDisplayTargetMapper mapper;

  setUp(() {
    mapper = NotificationDisplayTargetMapper();
  });

  group('NotificationDisplayTargetMapper', () {
    test('maps DisplayTarget_Os correctly', () {
      final now = DateTime(2024, 1, 1, 12, 0);
      final input = core.DisplayTarget_Os(notifyAt: now.toIso8601String());

      final result = mapper.map(input);

      expect(result, isA<Os>());
      expect((result as Os).notifyAt, now.toLocal());
    });

    test('maps DisplayTarget_Dashboard correctly', () {
      const input = core.DisplayTarget_Dashboard();

      final result = mapper.map(input);

      expect(result, const NotificationDisplayTarget.dashboard());
    });
  });
}
