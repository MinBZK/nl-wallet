import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/core/core_pid_repository.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/claim_display_metadata_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/localized_labels_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/card/metadata_mapper.dart';
import 'package:wallet/src/util/mapper/card/status/card_status_mapper.dart';
import 'package:wallet/src/util/mapper/configuration/flutter_app_configuration_mapper.dart';
import 'package:wallet/src/util/mapper/configuration/maintenance_window_mapper.dart';
import 'package:wallet/src/util/mapper/configuration/pid_attestation_mapper.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet/src/util/mapper/organization/organization_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late MockTypedWalletCore core;
  late PidRepository pidRepository;
  late CardMapper cardMapper;
  late FlutterAppConfigurationMapper configMapper;

  const kMockPidRenewalUrl = 'mock_pid_renewal_url';

  setUp(() {
    core = Mocks.create<TypedWalletCore>() as MockTypedWalletCore;
    when(core.observeConfig()).thenAnswer((_) => Stream.value(CoreMockData.flutterConfiguration));
    cardMapper = CardMapper(
      CardAttributeMapper(CardAttributeValueMapper(), ClaimDisplayMetadataMapper()),
      OrganizationMapper(LocalizedLabelsMapper(), ImageMapper()),
      DisplayMetadataMapper(ImageMapper()),
      CardStatusMapper(),
    );
    configMapper = FlutterAppConfigurationMapper(MaintenanceWindowMapper(), PidAttestationMapper());
    pidRepository = CorePidRepository(core, cardMapper, configMapper);
  });

  tearDown(() => reset(core));

  group('Pid issuance', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await pidRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });

    test('continue pid issuance should be propagated to the core', () async {
      const mockContinueUri = 'mock_continue_issuance_url';
      final testAttestation = AttestationPresentation(
        identity: const AttestationIdentity_Ephemeral(),
        attestationType: 'pid',
        format: .SdJwt,
        displayMetadata: [CoreMockData.enDisplayMetadata],
        issuer: CoreMockData.organization,
        attributes: CoreMockData.attestation.attributes,
        validityStatus: const ValidityStatus_Valid(validUntil: null),
      );
      final expectedAttributes = cardMapper.map(testAttestation).attributes;

      when(core.continueIssuance(mockContinueUri)).thenAnswer((realInvocation) async => [testAttestation]);
      expect(await pidRepository.continuePidIssuance(mockContinueUri), expectedAttributes);
      verify(core.continueIssuance(mockContinueUri));
    });

    test('accept offered pid should be propagated to the core', () async {
      const samplePin = '000000';
      await pidRepository.acceptIssuance(samplePin);
      verify(core.acceptPidIssuance(samplePin));
    });

    test('accept offered pid should propagate errors from the core as WalletInstructionError', () async {
      const samplePin = '000000';
      when(core.acceptPidIssuance(samplePin)).thenAnswer(
        (_) async => const PidIssuanceResult_InstructionError(
          error: WalletInstructionError.incorrectPin(attemptsLeftInRound: 3, isFinalRound: false),
        ),
      );

      expect(() async => pidRepository.acceptIssuance(samplePin), throwsA(isA<WalletInstructionError>()));
      verify(core.acceptPidIssuance(samplePin));
    });

    test('renewal url should be fetched through the wallet_core', () async {
      when(core.createPidRenewalRedirectUri()).thenAnswer((_) async => kMockPidRenewalUrl);
      expect(await pidRepository.getPidRenewalUrl(), kMockPidRenewalUrl);
      verify(core.createPidRenewalRedirectUri());
    });
  });
}
