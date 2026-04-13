import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet/src/util/mapper/event/wallet_event_mapper.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../mocks/core_mock_data.dart';
import '../../../mocks/wallet_mock_data.dart';
import '../../../mocks/wallet_mocks.dart';

void main() {
  late MockMapper<core.AttestationPresentation, WalletCard> cardMapper;
  late MockMapper<core.Organization, Organization> organizationMapper;
  late MockMapper<core.RequestPolicy, Policy> policyMapper;
  late MockMapper<List<core.LocalizedString>, LocalizedText> localizedStringMapper;
  late MockMapper<core.DisclosureType, DisclosureType> disclosureTypeMapper;
  late WalletEventMapper mapper;

  setUpAll(() {
    provideDummy<WalletCard>(WalletMockData.card);
    provideDummy<LocalizedText>(const {});
    provideDummy<DisclosureType>(DisclosureType.regular);
  });

  setUp(() {
    cardMapper = MockMapper<core.AttestationPresentation, WalletCard>();
    organizationMapper = MockMapper<core.Organization, Organization>();
    policyMapper = MockMapper<core.RequestPolicy, Policy>();
    localizedStringMapper = MockMapper<List<core.LocalizedString>, LocalizedText>();
    disclosureTypeMapper = MockMapper<core.DisclosureType, DisclosureType>();

    when(cardMapper.map(any)).thenReturn(WalletMockData.card);
    when(organizationMapper.map(any)).thenReturn(WalletMockData.organization);
    when(policyMapper.map(any)).thenReturn(WalletMockData.policy);
    when(localizedStringMapper.map(any)).thenReturn(const {});
    when(disclosureTypeMapper.map(any)).thenReturn(DisclosureType.regular);
    when(cardMapper.mapList(any)).thenReturn([WalletMockData.card]);

    mapper = WalletEventMapper(
      cardMapper,
      organizationMapper,
      policyMapper,
      localizedStringMapper,
      disclosureTypeMapper,
    );
  });

  group('WalletEvent_Deletion', () {
    test('maps to a successful DeletionEvent with the mapped card', () {
      const input = core.WalletEvent_Deletion(
        id: 'evt-1',
        dateTime: '2024-05-01T00:00:00.000Z',
        attestation: CoreMockData.attestation,
      );

      final result = mapper.map(input);

      expect(result, isA<DeletionEvent>());
      final deletion = result as DeletionEvent;
      expect(deletion.dateTime, DateTime.parse('2024-05-01T00:00:00.000Z').toLocal());
      expect(deletion.status, EventStatus.success);
      expect(deletion.card, WalletMockData.card);
      verify(cardMapper.map(CoreMockData.attestation)).called(1);
    });
  });

  group('WalletEvent_Disclosure', () {
    test('maps a successful disclosure to a DisclosureEvent with the relying party', () {
      const input = core.WalletEvent_Disclosure(
        id: 'evt-2',
        dateTime: '2024-03-01T00:00:00.000Z',
        relyingParty: CoreMockData.organization,
        purpose: [],
        sharedAttestations: [CoreMockData.attestation],
        requestPolicy: CoreMockData.policy,
        status: core.DisclosureStatus.Success,
        typ: core.DisclosureType.Regular,
      );

      final result = mapper.map(input);

      expect(result, isA<DisclosureEvent>());
      final disclosure = result as DisclosureEvent;
      expect(disclosure.status, EventStatus.success);
      expect(disclosure.relyingParty, WalletMockData.organization);
    });
  });

  group('WalletEvent_Issuance', () {
    test('maps a non-renewed issuance to IssuanceEvent.cardIssued', () {
      const input = core.WalletEvent_Issuance(
        id: 'evt-3',
        dateTime: '2024-01-01T00:00:00.000Z',
        attestation: CoreMockData.attestation,
        renewed: false,
      );

      final result = mapper.map(input);

      expect(result, isA<IssuanceEvent>());
      final issuance = result as IssuanceEvent;
      expect(issuance.eventType, IssuanceEventType.cardIssued);
      expect(issuance.status, EventStatus.success);
      expect(issuance.card, WalletMockData.card);
    });

    test('maps a renewed issuance to IssuanceEvent.cardRenewed', () {
      const input = core.WalletEvent_Issuance(
        id: 'evt-4',
        dateTime: '2024-01-01T00:00:00.000Z',
        attestation: CoreMockData.attestation,
        renewed: true,
      );

      final result = mapper.map(input) as IssuanceEvent;
      expect(result.eventType, IssuanceEventType.cardRenewed);
    });
  });
}
