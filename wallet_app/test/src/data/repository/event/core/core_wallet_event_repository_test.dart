import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/event/core/core_wallet_event_repository.dart';
import 'package:wallet/src/domain/model/app_image_data.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/card/metadata/card_display_metadata.dart';
import 'package:wallet/src/domain/model/card/status/card_status.dart';
import 'package:wallet/src/domain/model/card/wallet_card.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/event/wallet_event.dart';
import 'package:wallet/src/domain/model/localized_text.dart';
import 'package:wallet/src/domain/model/organization.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet/src/domain/model/policy/policy.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockTypedWalletCore walletCore;
  late MockMapper<core.WalletEvent, WalletEvent> walletEventMapper;
  late MockMapper<core.FlutterConfiguration, FlutterAppConfiguration> configMapper;
  late CoreWalletEventRepository repository;

  setUp(() {
    walletCore = MockTypedWalletCore();
    walletEventMapper = MockMapper<core.WalletEvent, WalletEvent>();
    configMapper = MockMapper<core.FlutterConfiguration, FlutterAppConfiguration>();
    repository = CoreWalletEventRepository(walletCore, walletEventMapper, configMapper);
  });

  final now = DateTime(2023, 1, 1);

  WalletCard createCard({required String attestationType, required AttestationFormat format}) {
    return WalletCard(
      attestationId: 'id-$format',
      attestationType: attestationType,
      format: format,
      attributes: [],
      issuer: const Organization(
        id: 'issuer-id',
        legalName: '',
        displayName: '',
        category: {},
        description: {},
        logo: SvgImage(''),
        countryCode: '',
      ),
      metadata: [
        const CardDisplayMetadata(
          language: Locale('en'),
          name: 'Card',
        ),
      ],
      status: const CardStatus.valid(validUntil: null),
    );
  }

  FlutterAppConfiguration createConfig(List<PidAttestation> pidAttestations) {
    return FlutterAppConfiguration(
      idleLockTimeout: Duration.zero,
      idleWarningTimeout: Duration.zero,
      backgroundLockTimeout: Duration.zero,
      staticAssetsBaseUrl: '',
      pidAttestations: pidAttestations,
      maintenanceWindow: null,
      version: '1.0.0',
      environment: 'test',
    );
  }

  group('CoreWalletEventRepository', () {
    test('getEvents should fetch, map, and filter events', () async {
      final coreEvents = [
        core.WalletEvent_Issuance(
          id: '1',
          dateTime: now.toIso8601String(),
          attestation: const core.AttestationPresentation(
            identity: core.AttestationIdentity.ephemeral(),
            attestationType: 'pid',
            format: core.Format.SdJwt,
            displayMetadata: [],
            issuer: core.Organization(
              legalName: '',
              displayName: '',
              description: [],
              identifier: '',
              category: [],
              countryCode: '',
            ),
            attributes: [],
            validityStatus: core.ValidityStatus.valid(validUntil: null),
          ),
          renewed: false,
        ),
      ];

      final mappedEvents = [
        WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: createCard(attestationType: 'other', format: AttestationFormat.sdJwt),
          eventType: IssuanceEventType.cardIssued,
        ),
      ];

      when(
        walletCore.getHistory(page: anyNamed('page'), pageSize: anyNamed('pageSize')),
      ).thenAnswer((_) async => coreEvents);
      when(walletEventMapper.mapList(coreEvents)).thenReturn(mappedEvents);

      final config = createConfig([]);
      when(walletCore.observeConfig()).thenAnswer(
        (_) => Stream.value(
          const core.FlutterConfiguration(
            inactiveWarningTimeout: 0,
            inactiveLockTimeout: 0,
            backgroundLockTimeout: 0,
            pidAttestations: [],
            staticAssetsBaseUrl: '',
            version: '',
            environment: '',
          ),
        ),
      );
      when(configMapper.map(any)).thenReturn(config);

      final result = await repository.getEvents();

      expect(result, mappedEvents);
      verify(walletCore.getHistory(page: anyNamed('page'), pageSize: anyNamed('pageSize'))).called(1);
    });

    test('getEvents with removeDuplicatePidEvents: false should return all events without filtering', () async {
      final coreEvents = [
        core.WalletEvent_Issuance(
          id: '1',
          dateTime: now.toIso8601String(),
          attestation: const core.AttestationPresentation(
            identity: core.AttestationIdentity.ephemeral(),
            attestationType: 'pid',
            format: core.Format.SdJwt,
            displayMetadata: [],
            issuer: core.Organization(
              legalName: '',
              displayName: '',
              description: [],
              identifier: '',
              category: [],
              countryCode: '',
            ),
            attributes: [],
            validityStatus: core.ValidityStatus.valid(validUntil: null),
          ),
          renewed: false,
        ),
      ];

      final mappedEvents = [
        WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: createCard(attestationType: 'pid', format: AttestationFormat.sdJwt),
          eventType: IssuanceEventType.cardIssued,
        ),
      ];

      when(
        walletCore.getHistory(page: anyNamed('page'), pageSize: anyNamed('pageSize')),
      ).thenAnswer((_) async => coreEvents);
      when(walletEventMapper.mapList(coreEvents)).thenReturn(mappedEvents);

      final result = await repository.getEvents(page: 0, pageSize: 100, removeDuplicatePidEvents: false);

      expect(result, mappedEvents);
      verify(walletCore.getHistory(page: anyNamed('page'), pageSize: anyNamed('pageSize'))).called(1);
      verifyNever(walletCore.observeConfig()); // Filtering relies on config, so make sure it's not fetched.
    });

    test('getEvents throws when pagination is combined with removeDuplicatePidEvents: true', () async {
      expect(
        () => repository.getEvents(page: 0, pageSize: 100, removeDuplicatePidEvents: true),
        throwsArgumentError,
      );
      expect(
        () => repository.getEvents(page: 0, pageSize: 100),
        throwsArgumentError,
      );
    });

    test('getEvents with removeDuplicatePidEvents: true should filter events', () async {
      final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
      final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

      final eventSdJwt = WalletEvent.issuance(
        dateTime: now,
        status: EventStatus.success,
        card: pidSdJwt,
        eventType: IssuanceEventType.cardIssued,
      );
      final eventMdoc = WalletEvent.issuance(
        dateTime: now,
        status: EventStatus.success,
        card: pidMdoc,
        eventType: IssuanceEventType.cardIssued,
      );

      final mappedEvents = [eventSdJwt, eventMdoc];

      when(walletCore.getHistory(page: anyNamed('page'), pageSize: anyNamed('pageSize'))).thenAnswer((_) async => []);
      when(walletEventMapper.mapList(any)).thenReturn(mappedEvents);

      final config = createConfig([const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt)]);
      when(walletCore.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(CoreMockData.flutterConfiguration)).thenReturn(config);

      final result = await repository.getEvents(removeDuplicatePidEvents: true);

      expect(result, [eventSdJwt]);
    });

    test('getEventsForCard should fetch and map events for a card', () async {
      final coreEvents = [
        core.WalletEvent_Issuance(
          id: 'some-id',
          dateTime: now.toIso8601String(),
          attestation: const core.AttestationPresentation(
            identity: core.AttestationIdentity.ephemeral(),
            attestationType: 'pid',
            format: core.Format.SdJwt,
            displayMetadata: [],
            issuer: core.Organization(
              legalName: '',
              displayName: '',
              description: [],
              identifier: '',
              category: [],
              countryCode: '',
            ),
            attributes: [],
            validityStatus: core.ValidityStatus.valid(validUntil: null),
          ),
          renewed: false,
        ),
      ];

      final mappedEvents = [
        WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: createCard(attestationType: 'pid', format: AttestationFormat.sdJwt),
          eventType: IssuanceEventType.cardIssued,
        ),
      ];

      when(walletCore.getHistoryForCard('some-id')).thenAnswer((_) async => coreEvents);
      when(walletEventMapper.mapList(coreEvents)).thenReturn(mappedEvents);

      final result = await repository.getEventsForCard('some-id');

      expect(result, mappedEvents);
      verify(walletCore.getHistoryForCard('some-id')).called(1);
    });

    test('readMostRecentDisclosureEvent should return the most recent disclosure event with given status', () async {
      final card = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
      final mostRecentDisclosureEvent = WalletEvent.disclosure(
        dateTime: now,
        status: EventStatus.success,
        relyingParty: const Organization(
          id: 'rp-id',
          legalName: '',
          displayName: '',
          category: {},
          description: {},
          logo: SvgImage(''),
          countryCode: '',
        ),
        purpose: {},
        cards: [card],
        policy: const Policy(dataIsShared: false, deletionCanBeRequested: false, privacyPolicyUrl: ''),
        type: DisclosureType.regular,
      );
      final events = [
        WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: createCard(attestationType: 'pid', format: .sdJwt),
          eventType: .cardIssued,
        ),
        mostRecentDisclosureEvent,
      ];

      when(walletCore.getHistoryForCard('some-id')).thenAnswer((_) async => []);
      when(walletEventMapper.mapList(any)).thenReturn(events);

      final result = await repository.readMostRecentDisclosureEvent('some-id', EventStatus.success);

      expect(result, mostRecentDisclosureEvent);
    });

    test('observeRecentEvents should stream mapped and filtered events', () async {
      final coreEvents = [
        core.WalletEvent_Issuance(
          id: '1',
          dateTime: now.toIso8601String(),
          attestation: const core.AttestationPresentation(
            identity: core.AttestationIdentity.ephemeral(),
            attestationType: 'pid',
            format: core.Format.SdJwt,
            displayMetadata: [],
            issuer: core.Organization(
              legalName: '',
              displayName: '',
              description: [],
              identifier: '',
              category: [],
              countryCode: '',
            ),
            attributes: [],
            validityStatus: core.ValidityStatus.valid(validUntil: null),
          ),
          renewed: false,
        ),
      ];

      final mappedEvents = [
        WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: createCard(attestationType: 'pid', format: AttestationFormat.sdJwt),
          eventType: IssuanceEventType.cardIssued,
        ),
      ];

      when(walletCore.observeRecentHistory()).thenAnswer((_) => Stream.value(coreEvents));
      when(walletEventMapper.mapList(coreEvents)).thenReturn(mappedEvents);

      final config = createConfig([]);
      when(walletCore.observeConfig()).thenAnswer(
        (_) => Stream.value(
          const core.FlutterConfiguration(
            inactiveWarningTimeout: 0,
            inactiveLockTimeout: 0,
            backgroundLockTimeout: 0,
            pidAttestations: [],
            staticAssetsBaseUrl: '',
            version: '',
            environment: '',
          ),
        ),
      );
      when(configMapper.map(any)).thenReturn(config);

      final result = await repository.observeRecentEvents().first;

      expect(result, mappedEvents);
    });

    test(
      'observeRecentEvents with removeDuplicatePidEvents: false should stream mapped events without filtering',
      () async {
        final coreEvents = [
          core.WalletEvent_Issuance(
            id: '1',
            dateTime: now.toIso8601String(),
            attestation: const core.AttestationPresentation(
              identity: core.AttestationIdentity.ephemeral(),
              attestationType: 'pid',
              format: core.Format.SdJwt,
              displayMetadata: [],
              issuer: core.Organization(
                legalName: '',
                displayName: '',
                description: [],
                identifier: '',
                category: [],
                countryCode: '',
              ),
              attributes: [],
              validityStatus: core.ValidityStatus.valid(validUntil: null),
            ),
            renewed: false,
          ),
        ];

        final mappedEvents = [
          WalletEvent.issuance(
            dateTime: now,
            status: EventStatus.success,
            card: createCard(attestationType: 'pid', format: AttestationFormat.sdJwt),
            eventType: IssuanceEventType.cardIssued,
          ),
        ];

        when(walletCore.observeRecentHistory()).thenAnswer((_) => Stream.value(coreEvents));
        when(walletEventMapper.mapList(coreEvents)).thenReturn(mappedEvents);

        final result = await repository.observeRecentEvents(removeDuplicatePidEvents: false).first;

        expect(result, mappedEvents);
        verifyNever(walletCore.observeConfig()); // Filtering relies on config, make sure it's not fetched.
      },
    );

    test('observeRecentEvents with removeDuplicatePidEvents: true should filter events', () async {
      final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
      final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

      final eventSdJwt = WalletEvent.issuance(
        dateTime: now,
        status: EventStatus.success,
        card: pidSdJwt,
        eventType: IssuanceEventType.cardIssued,
      );
      final eventMdoc = WalletEvent.issuance(
        dateTime: now,
        status: EventStatus.success,
        card: pidMdoc,
        eventType: IssuanceEventType.cardIssued,
      );
      final nonPidEvent = WalletEvent.issuance(
        dateTime: now,
        status: .success,
        card: WalletMockData.altCard,
        eventType: .cardIssued,
      );

      final mappedEvents = [eventSdJwt, eventMdoc, nonPidEvent];

      when(walletCore.observeRecentHistory()).thenAnswer((_) => Stream.value([]));
      when(walletEventMapper.mapList(any)).thenReturn(mappedEvents);

      final config = createConfig([const PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc)]);
      when(walletCore.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(CoreMockData.flutterConfiguration)).thenReturn(config);

      final result = await repository.observeRecentEvents(removeDuplicatePidEvents: true).first;

      expect(result, [eventMdoc, nonPidEvent]);
    });

    group('filterDuplicatePidEvents', () {
      test('should return all events if no PID events are present', () async {
        final events = [
          WalletEvent.issuance(
            dateTime: now,
            status: EventStatus.success,
            card: createCard(attestationType: 'not-pid', format: AttestationFormat.sdJwt),
            eventType: IssuanceEventType.cardIssued,
          ),
        ];

        final config = createConfig([
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
        ]);
        when(walletCore.observeConfig()).thenAnswer(
          (_) => Stream.value(
            const core.FlutterConfiguration(
              inactiveWarningTimeout: 0,
              inactiveLockTimeout: 0,
              backgroundLockTimeout: 0,
              pidAttestations: [],
              staticAssetsBaseUrl: '',
              version: '',
              environment: '',
            ),
          ),
        );
        when(configMapper.map(any)).thenReturn(config);

        final result = await repository.filterDuplicatePidEvents(events);

        expect(result, events);
      });

      test('should filter duplicate PID events based on priority', () async {
        final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

        final eventSdJwt = WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: pidSdJwt,
          eventType: IssuanceEventType.cardIssued,
        );
        final eventMdoc = WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: pidMdoc,
          eventType: IssuanceEventType.cardIssued,
        );

        final events = [eventSdJwt, eventMdoc];

        // SD-JWT has higher priority
        final config = createConfig([
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc),
        ]);

        when(walletCore.observeConfig()).thenAnswer(
          (_) => Stream.value(
            const core.FlutterConfiguration(
              inactiveWarningTimeout: 0,
              inactiveLockTimeout: 0,
              backgroundLockTimeout: 0,
              pidAttestations: [],
              staticAssetsBaseUrl: '',
              version: '',
              environment: '',
            ),
          ),
        );
        when(configMapper.map(any)).thenReturn(config);

        final result = await repository.filterDuplicatePidEvents(events);

        expect(result, [eventSdJwt]);
      });

      test('should keep the other event if priority is reversed', () async {
        final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

        final eventSdJwt = WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: pidSdJwt,
          eventType: IssuanceEventType.cardIssued,
        );
        final eventMdoc = WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: pidMdoc,
          eventType: IssuanceEventType.cardIssued,
        );

        final events = [eventSdJwt, eventMdoc];

        // MDOC has higher priority
        final config = createConfig([
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc),
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
        ]);

        when(walletCore.observeConfig()).thenAnswer(
          (_) => Stream.value(
            const core.FlutterConfiguration(
              inactiveWarningTimeout: 0,
              inactiveLockTimeout: 0,
              backgroundLockTimeout: 0,
              pidAttestations: [],
              staticAssetsBaseUrl: '',
              version: '',
              environment: '',
            ),
          ),
        );
        when(configMapper.map(any)).thenReturn(config);

        final result = await repository.filterDuplicatePidEvents(events);

        expect(result, [eventMdoc]);
      });

      test('should not filter events that do not match in time', () async {
        final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

        final eventSdJwt = WalletEvent.issuance(
          dateTime: now,
          status: EventStatus.success,
          card: pidSdJwt,
          eventType: IssuanceEventType.cardIssued,
        );
        final eventMdoc = WalletEvent.issuance(
          dateTime: now.add(const Duration(seconds: 1)),
          status: EventStatus.success,
          card: pidMdoc,
          eventType: IssuanceEventType.cardIssued,
        );

        final events = [eventSdJwt, eventMdoc];

        final config = createConfig([
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.mdoc),
        ]);

        when(walletCore.observeConfig()).thenAnswer(
          (_) => Stream.value(
            const core.FlutterConfiguration(
              inactiveWarningTimeout: 0,
              inactiveLockTimeout: 0,
              backgroundLockTimeout: 0,
              pidAttestations: [],
              staticAssetsBaseUrl: '',
              version: '',
              environment: '',
            ),
          ),
        );
        when(configMapper.map(any)).thenReturn(config);

        final result = await repository.filterDuplicatePidEvents(events);

        expect(result, containsAll([eventSdJwt, eventMdoc]));
      });

      test('should filter duplicate deletion events', () async {
        final pidSdJwt = createCard(attestationType: 'pid', format: AttestationFormat.sdJwt);
        final pidMdoc = createCard(attestationType: 'pid', format: AttestationFormat.mdoc);

        final eventSdJwt = WalletEvent.deletion(
          dateTime: now,
          status: EventStatus.success,
          card: pidSdJwt,
        );
        final eventMdoc = WalletEvent.deletion(
          dateTime: now,
          status: EventStatus.success,
          card: pidMdoc,
        );

        final events = [eventSdJwt, eventMdoc];

        final config = createConfig([
          const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt),
        ]);

        when(walletCore.observeConfig()).thenAnswer(
          (_) => Stream.value(
            const core.FlutterConfiguration(
              inactiveWarningTimeout: 0,
              inactiveLockTimeout: 0,
              backgroundLockTimeout: 0,
              pidAttestations: [],
              staticAssetsBaseUrl: '',
              version: '',
              environment: '',
            ),
          ),
        );
        when(configMapper.map(any)).thenReturn(config);

        final result = await repository.filterDuplicatePidEvents(events);

        expect(result, [eventSdJwt]);
      });
    });
  });
}
