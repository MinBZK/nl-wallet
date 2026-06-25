import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/card/impl/wallet_card_repository_impl.dart';
import 'package:wallet/src/domain/model/card/format/attestation_format.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/pid/pid_attestation.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart' as core;

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late WalletCardRepositoryImpl repository;
  late MockWalletDataSource dataSource;
  late MockTypedWalletCore coreService;
  late MockMapper<core.FlutterConfiguration, FlutterAppConfiguration> configMapper;

  setUp(() {
    dataSource = MockWalletDataSource();
    coreService = Mocks.create<TypedWalletCore>() as MockTypedWalletCore;
    configMapper = MockMapper();
    repository = WalletCardRepositoryImpl(dataSource, coreService, configMapper);
  });

  tearDown(() {
    reset(dataSource);
    reset(coreService);
    reset(configMapper);
  });

  group('WalletCardRepositoryImpl', () {
    test('observeWalletCards should call dataSource and return cards', () async {
      final cards = [WalletMockData.card];
      when(dataSource.observeCards()).thenAnswer((_) => Stream.value(cards));

      when(coreService.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(any)).thenReturn(WalletMockData.flutterAppConfiguration.copyWith(pidAttestations: []));

      final result = await repository.observeWalletCards().first;
      expect(result, cards);
      verify(dataSource.observeCards());
    });

    test('observeWalletCards with filterDuplicatePids: false should return all cards without filtering', () async {
      final cards = [WalletMockData.card];
      when(dataSource.observeCards()).thenAnswer((_) => Stream.value(cards));

      final result = await repository.observeWalletCards(filterDuplicatePids: false).first;
      expect(result, cards);
      verify(dataSource.observeCards());
      verifyNever(coreService.observeConfig()); // Card filtering depends on config, so make sure it isn't fetched.
    });

    test('observeWalletCards with filterDuplicatePids: true should filter cards', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: 'pid', attestationId: 'id1', format: .mdoc);
      final pidCard2 = WalletMockData.card.copyWith(attestationType: 'pid', attestationId: 'id2', format: .sdJwt);
      final cards = [pidCard1, pidCard2];

      when(dataSource.observeCards()).thenAnswer((_) => Stream.value(cards));
      when(coreService.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(CoreMockData.flutterConfiguration)).thenReturn(
        WalletMockData.flutterAppConfiguration.copyWith(
          pidAttestations: [const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt)],
        ),
      );

      final result = await repository.observeWalletCards(filterDuplicatePids: true).first;

      expect(result, [pidCard2]);
      verify(dataSource.observeCards());
    });

    test('exists should return true if card exists in dataSource', () async {
      when(dataSource.read('id')).thenAnswer((_) async => WalletMockData.card);
      expect(await repository.exists('id'), true);
      verify(dataSource.read('id'));
    });

    test('exists should return false if card does not exist in dataSource', () async {
      when(dataSource.read('id')).thenAnswer((_) async => null);
      expect(await repository.exists('id'), false);
      verify(dataSource.read('id'));
    });

    test('readAll should call dataSource and return cards', () async {
      final cards = [WalletMockData.card];
      when(dataSource.readAll()).thenAnswer((_) async => cards);
      when(coreService.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(any)).thenReturn(WalletMockData.flutterAppConfiguration.copyWith(pidAttestations: []));

      final result = await repository.readAll();
      expect(result, cards);
      verify(dataSource.readAll());
    });

    test('readAll with filterDuplicatePids: false should return all cards without filtering', () async {
      final cards = [WalletMockData.card];
      when(dataSource.readAll()).thenAnswer((_) async => cards);

      final result = await repository.readAll(filterDuplicatePids: false);
      expect(result, cards);
      verify(dataSource.readAll());
      verifyNever(coreService.observeConfig());
    });

    test('readAll with filterDuplicatePids: true should filter cards', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: 'pid', attestationId: 'id1', format: .mdoc);
      final pidCard2 = WalletMockData.card.copyWith(attestationType: 'pid', attestationId: 'id2', format: .sdJwt);
      final altCard = WalletMockData.card.copyWith(attestationType: 'non-pid', attestationId: 'id3', format: .mdoc);
      final cards = [pidCard1, pidCard2, altCard];

      when(dataSource.readAll()).thenAnswer((_) async => cards);
      when(coreService.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(any)).thenReturn(
        WalletMockData.flutterAppConfiguration.copyWith(
          pidAttestations: [const PidAttestation(attestationType: 'pid', format: AttestationFormat.sdJwt)],
        ),
      );

      final result = await repository.readAll(filterDuplicatePids: true);

      expect(result, [pidCard2, altCard]);
      verify(dataSource.readAll());
    });

    test('read should return card from dataSource', () async {
      when(dataSource.read('id')).thenAnswer((_) async => WalletMockData.card);
      expect(await repository.read('id'), WalletMockData.card);
      verify(dataSource.read('id'));
    });

    test('delete should call dataSource and handle Ok result', () async {
      when(dataSource.delete('pin', 'id')).thenAnswer((_) async => const core.WalletInstructionResult_Ok());
      await repository.delete('pin', 'id');
      verify(dataSource.delete('pin', 'id'));
    });

    test('delete should throw error on InstructionError', () async {
      const error = core.WalletInstructionError.incorrectPin(attemptsLeftInRound: 3, isFinalRound: false);
      when(
        dataSource.delete('pin', 'id'),
      ).thenAnswer((_) async => const core.WalletInstructionResult_InstructionError(error: error));

      expect(() => repository.delete('pin', 'id'), throwsA(error));
    });
  });

  group('filterDuplicatePidCards', () {
    test('should keep only the first matching PID card based on configuration ordering', () async {
      final pidCard1 = WalletMockData.card.copyWith(attestationType: 'type1', attestationId: 'id1', format: .mdoc);
      final pidCard2 = WalletMockData.card.copyWith(attestationType: 'type2', attestationId: 'id2', format: .sdJwt);
      final otherCard = WalletMockData.card.copyWith(attestationType: 'other', attestationId: 'id3');

      final cards = [pidCard1, pidCard2, otherCard];

      when(coreService.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(any)).thenReturn(
        WalletMockData.flutterAppConfiguration.copyWith(
          pidAttestations: [
            const PidAttestation(attestationType: 'type2', format: AttestationFormat.sdJwt),
            const PidAttestation(attestationType: 'type1', format: AttestationFormat.mdoc),
          ],
        ),
      );

      final result = await repository.filterDuplicatePidCards(cards);

      // type2 comes first in config, so it should be maintained.
      // type1 is a pid type, so it should be filtered out.
      // otherCard is NOT a pid type, so it should be kept.
      expect(result, containsAll([pidCard2, otherCard]));
      expect(result, isNot(contains(pidCard1)));
    });

    test('should return all cards if no PID cards are present', () async {
      final cards = [WalletMockData.card.copyWith(attestationType: 'other', attestationId: 'id1')];

      when(
        coreService.observeConfig(),
      ).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
      when(configMapper.map(any)).thenReturn(
        WalletMockData.flutterAppConfiguration.copyWith(
          pidAttestations: [const PidAttestation(attestationType: 'pid_type', format: AttestationFormat.sdJwt)],
        ),
      );

      final result = await repository.filterDuplicatePidCards(cards);
      expect(result, cards);
    });
  });
}
