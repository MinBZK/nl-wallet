import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/core/core_pid_repository.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/localized_labels_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_config_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_subtitle_mapper.dart';
import 'package:wallet/src/util/mapper/image/image_mapper.dart';
import 'package:wallet/src/util/mapper/organization/organization_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late TypedWalletCore core;
  late PidRepository pidRepository;

  setUp(() {
    core = Mocks.create();
    //FIXME: Mock mappers
    pidRepository = CorePidRepository(
      core,
      CardMapper(
        CardFrontMapper(CardSubtitleMapper(CardAttributeValueMapper())),
        CardConfigMapper(),
        CardAttributeMapper(CardAttributeValueMapper(), LocalizedLabelsMapper()),
        OrganizationMapper(LocalizedLabelsMapper(), ImageMapper()),
      ),
    );
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await pidRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });
  });
}
