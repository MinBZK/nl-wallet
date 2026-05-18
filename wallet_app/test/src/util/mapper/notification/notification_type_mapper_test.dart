import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/notification/app_notification.dart';
import 'package:wallet/src/util/mapper/notification/notification_type_mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late NotificationTypeMapper mapper;
  late MockMapper<core.AttestationPresentation, WalletCard> cardMapper;

  setUp(() {
    cardMapper = MockMapper<core.AttestationPresentation, WalletCard>();
    mapper = NotificationTypeMapper(cardMapper);
    provideDummy<WalletCard>(
      WalletCard(
        attestationId: '',
        attestationType: '',
        issuer: WalletMockData.organization,
        status: .valid(validUntil: DateTime(2100)),
        attributes: [],
      ),
    );
  });

  final walletCard = MockWalletCard();

  group('NotificationTypeMapper', () {
    test('maps NotificationType_CardExpired correctly', () {
      final coreCard = MockAttestationPresentation();
      final input = core.NotificationType.cardExpired(card: coreCard);
      when(cardMapper.map(coreCard)).thenReturn(walletCard);

      final result = mapper.map(input);

      expect(result, NotificationType.cardExpired(card: walletCard));
      verify(cardMapper.map(coreCard)).called(1);
    });

    test('maps NotificationType_CardExpiresSoon correctly', () {
      final coreCard = MockAttestationPresentation();
      final expiresAt = DateTime(2024, 1, 1, 12, 0);
      final input = core.NotificationType.cardExpiresSoon(
        card: coreCard,
        expiresAt: expiresAt.toIso8601String(),
      );
      when(cardMapper.map(coreCard)).thenReturn(walletCard);

      final result = mapper.map(input);

      expect(result, NotificationType.cardExpiresSoon(card: walletCard, expiresAt: expiresAt.toLocal()));
      verify(cardMapper.map(coreCard)).called(1);
    });

    test('maps NotificationType_Revoked correctly', () {
      final coreCard = MockAttestationPresentation();
      final input = core.NotificationType.revoked(card: coreCard);
      when(cardMapper.map(coreCard)).thenReturn(walletCard);

      final result = mapper.map(input);

      expect(result, NotificationType.cardRevoked(card: walletCard));
      verify(cardMapper.map(coreCard)).called(1);
    });
  });
}
