import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/core/core_pid_repository.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/data/store/active_locale_provider.dart';
import 'package:wallet/src/data/store/impl/active_localization_delegate.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_label_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_mapper.dart';
import 'package:wallet/src/util/mapper/card/attribute/card_attribute_value_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_front_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_mapper.dart';
import 'package:wallet/src/util/mapper/card/card_subtitle_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late TypedWalletCore core;
  late ActiveLocaleProvider localeProvider;
  late PidRepository pidRepository;

  setUp(() {
    core = Mocks.create();
    localeProvider = ActiveLocalizationDelegate(); // Defaults to 'en'
    //FIXME: Mock mappers
    pidRepository = CorePidRepository(
      core,
      CardMapper(
        CardFrontMapper(CardSubtitleMapper(CardAttributeValueMapper())),
        CardAttributeMapper(CardAttributeLabelMapper(), CardAttributeValueMapper()),
      ),
      localeProvider,
    );
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await pidRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });
  });
}
