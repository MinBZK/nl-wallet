import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/core/core_pid_repository.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/claim_display_metadata_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/localized_labels_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_config_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/card/metadata_mapper.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet/src/util/mapper/organization/organization_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/core_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late TypedWalletCore core;
  late PidRepository pidRepository;
  late CardMapper cardMapper;

  setUp(() {
    core = Mocks.create();
    cardMapper = CardMapper(
      CardConfigMapper(),
      CardAttributeMapper(CardAttributeValueMapper(), ClaimDisplayMetadataMapper()),
      OrganizationMapper(LocalizedLabelsMapper(), ImageMapper()),
      DisplayMetadataMapper(ImageMapper()),
    );
    pidRepository = CorePidRepository(core, cardMapper);
  });

  group('Pid issuance', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await pidRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });

    test('continue pid issuance should be propagated to the core', () async {
      const mockContinueUri = 'mock_continue_issuance_url';
      final testAttestation = Attestation(
        identity: const AttestationIdentity_Ephemeral(),
        attestationType: kPidDocType,
        displayMetadata: [CoreMockData.enDisplayMetadata],
        issuer: CoreMockData.organization,
        attributes: CoreMockData.attestation.attributes,
      );
      final expectedAttributes = cardMapper.map(testAttestation).attributes;

      when(core.continuePidIssuance(mockContinueUri)).thenAnswer((realInvocation) async => [testAttestation]);
      expect(await pidRepository.continuePidIssuance(mockContinueUri), expectedAttributes);
      verify(core.continuePidIssuance(mockContinueUri));
    });

    test('cancel pid issuance should be propagated to the core', () async {
      await pidRepository.cancelPidIssuance();
      verify(core.cancelPidIssuance());
    });

    test('accept offered pid should be propagated to the core', () async {
      const samplePin = '000000';
      await pidRepository.acceptOfferedPid(samplePin);
      verify(core.acceptOfferedPid(samplePin));
    });
  });
}
