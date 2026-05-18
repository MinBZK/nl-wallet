import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/util/mapper/notification/app_notification_mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late AppNotificationMapper mapper;
  late MockMapper<core.NotificationType, NotificationType> typeMapper;
  late MockMapper<core.DisplayTarget, NotificationDisplayTarget> displayTargetMapper;

  setUp(() {
    provideDummy<NotificationDisplayTarget>(const NotificationDisplayTarget.dashboard());
    typeMapper = MockMapper<core.NotificationType, NotificationType>();
    displayTargetMapper = MockMapper<core.DisplayTarget, NotificationDisplayTarget>();
    mapper = AppNotificationMapper(typeMapper, displayTargetMapper);
  });

  group('AppNotificationMapper', () {
    test('maps AppNotification correctly', () {
      final coreType = core.NotificationType.cardExpired(card: MockAttestationPresentation());
      const coreTarget = core.DisplayTarget.dashboard();
      final input = core.AppNotification(
        id: 1,
        typ: coreType,
        targets: [coreTarget],
      );

      final domainType = NotificationType.cardExpired(card: MockWalletCard());
      const domainTarget = NotificationDisplayTarget.dashboard();

      when(typeMapper.map(coreType)).thenReturn(domainType);
      when(displayTargetMapper.mapList([coreTarget])).thenReturn([domainTarget]);

      final result = mapper.map(input);

      expect(result.id, 1);
      expect(result.type, domainType);
      expect(result.displayTargets, [domainTarget]);

      verify(typeMapper.map(coreType)).called(1);
      verify(displayTargetMapper.mapList([coreTarget])).called(1);
    });
  });
}
