import 'package:mockito/mockito.dart';
import 'package:test/test.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_session_type.dart';
import 'package:wallet/src/domain/model/disclosure/disclosure_type.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/disclosure/impl/start_disclosure_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/disclosure/start_disclosure_usecase.dart';
import 'package:wallet/src/util/extension/string_extension.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  final repository = MockDisclosureRepository();
  late StartDisclosureUseCase usecase;

  setUp(() {
    usecase = StartDisclosureUseCaseImpl(repository);
  });

  test('Verify ReadyToDisclose is returned when all is good', () async {
    final readyToDiscloseResult = StartDisclosureReadyToDisclose(
      relyingParty: WalletMockData.organization,
      originUrl: 'http://origin.org',
      requestPurpose: 'requestPurpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      type: DisclosureType.login,
      policy: WalletMockData.policy,
      sharedDataWithOrganizationBefore: false,
      requestedAttributes: {},
    );
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => readyToDiscloseResult,
    );

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isFalse);
    expect(result.value, readyToDiscloseResult);
  });

  test('Verify MissingAttributes is returned when attributes are missing', () async {
    final missingAttributesResult = StartDisclosureMissingAttributes(
      relyingParty: WalletMockData.organization,
      originUrl: 'http://origin.org',
      requestPurpose: 'requestPurpose'.untranslated,
      sessionType: DisclosureSessionType.crossDevice,
      sharedDataWithOrganizationBefore: false,
      missingAttributes: [],
    );
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode'))).thenAnswer(
      (_) async => missingAttributesResult,
    );

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isFalse);
    expect(result.value, missingAttributesResult);
  });

  test('Verify ExternalScannerError is returned when CoreDisclosureSourceMismatchError is thrown for crossDevice',
      () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreDisclosureSourceMismatchError('', isCrossDevice: true));

    final result = await usecase.invoke('disclosureUri', isQrCode: true);
    expect(result.hasError, isTrue);
    expect(result.error, isA<ExternalScannerError>());
  });

  test('Verify ExternalScannerError is NOT returned when CoreDisclosureSourceMismatchError is thrown for sameDevice',
      () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreDisclosureSourceMismatchError('', isCrossDevice: false));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(result.error, isA<GenericError>());
  });

  test('Verify ExpiredError is returned when CoreExpiredSessionError is thrown (retry=true)', () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreExpiredSessionError('expired', canRetry: true));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(
      result.error,
      isA<SessionError>()
          .having((error) => error.state, 'state', SessionState.expired)
          .having((error) => error.canRetry, 'canRetry', isTrue),
    );
  });

  test('Verify ExpiredError is returned when CoreExpiredSessionError is thrown (retry=false)', () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreExpiredSessionError('expired', canRetry: false));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(
      result.error,
      isA<SessionError>()
          .having((error) => error.state, 'state', SessionState.expired)
          .having((error) => error.canRetry, 'canRetry', isFalse),
    );
  });

  test('Verify ExpiredError is returned when CoreCancelledSessionError is thrown ', () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreCancelledSessionError('cancelled'));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(result.error, isA<SessionError>().having((error) => error.state, 'state', SessionState.cancelled));
  });

  test('Verify NetworkError is returned when CoreNetworkError is thrown ', () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreNetworkError('server'));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(result.error, isA<NetworkError>());
  });

  test('Verify GenericError includes returnUrl if CoreGenericError contains it', () async {
    when(repository.startDisclosure(any, isQrCode: anyNamed('isQrCode')))
        .thenAnswer((_) async => throw const CoreGenericError('generic', data: {'return_url': 'https://example.org'}));

    final result = await usecase.invoke('disclosureUri');
    expect(result.hasError, isTrue);
    expect(
      result.error,
      isA<GenericError>().having(
        (error) => error.redirectUrl,
        'redirectUrl matches that from the original error',
        'https://example.org',
      ),
    );
  });
}
