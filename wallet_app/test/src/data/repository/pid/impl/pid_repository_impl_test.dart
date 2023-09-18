import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/pid/impl/pid_repository_impl.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/data/store/active_locale_provider.dart';
import 'package:wallet/src/data/store/impl/active_localization_delegate.dart';
import 'package:wallet/src/util/mapper/card/card_attribute_label_mapper.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet/src/wallet_core/error/core_error_mapper.dart';
import 'package:wallet/src/wallet_core/error/flutter_api_error.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late TypedWalletCore core;
  late ActiveLocaleProvider localeProvider;
  late PidRepository pidRepository;

  setUp(() {
    core = Mocks.create();
    localeProvider = ActiveLocalizationDelegate(); // Defaults to 'en'
    pidRepository = PidRepositoryImpl(core, CoreErrorMapper(), CardAttributeLabelMapper(), localeProvider);
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the wallet_core', () async {
      expect(await pidRepository.getPidIssuanceUrl(), kMockPidIssuanceUrl);
      verify(core.createPidIssuanceRedirectUri());
    });
  });

  group('DigiD State Updates', () {
    test('Stream updates when notified', () async {
      expectLater(pidRepository.observePidIssuanceStatus(), emitsInOrder([PidIssuanceAuthenticating()]));
      pidRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
    });

    test('Stream reflects success state and returns back to idle', () async {
      expectLater(pidRepository.observePidIssuanceStatus(),
          emitsInOrder([PidIssuanceAuthenticating(), PidIssuanceSuccess(List.empty()), PidIssuanceIdle()]));
      pidRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
      pidRepository.notifyPidIssuanceStateUpdate(PidIssuanceEvent.success(previewCards: List.empty()));
    });

    test('Stream reflects unknown error state and returns back to idle', () async {
      expectLater(pidRepository.observePidIssuanceStatus(),
          emitsInOrder([PidIssuanceAuthenticating(), PidIssuanceError(RedirectError.unknown), PidIssuanceIdle()]));
      pidRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
      pidRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.error(data: 'data'));
    });

    test('Stream reflects accessDenied error state and returns back to idle', () async {
      expectLater(pidRepository.observePidIssuanceStatus(),
          emitsInOrder([PidIssuanceAuthenticating(), PidIssuanceError(RedirectError.accessDenied), PidIssuanceIdle()]));
      pidRepository.notifyPidIssuanceStateUpdate(const PidIssuanceEvent.authenticating());
      pidRepository.notifyPidIssuanceStateUpdate(
        PidIssuanceEvent.error(
          data: jsonEncode(
            FlutterApiError(
              type: FlutterApiErrorType.redirectUri,
              description: '',
              data: {'redirect_error': 'access_denied'},
            ),
          ),
        ),
      );
    });

    test('Stream returns back to idle when receiving a null status', () async {
      expectLater(pidRepository.observePidIssuanceStatus(), emitsInOrder([PidIssuanceIdle()]));
      pidRepository.notifyPidIssuanceStateUpdate(null);
    });
  });
}
