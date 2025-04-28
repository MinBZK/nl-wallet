import 'dart:convert';

import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet/src/wallet_core/error/flutter_api_error.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart';

import '../../mocks/core_mock_data.dart';
import '../../mocks/wallet_mocks.dart';

const _kSamplePin = '112233';
const _kSampleIssuer = CoreMockData.organization;

void main() {
  final WalletCoreApi core = Mocks.create();
  WalletCore.initMock(api: core);

  late MockMapper<String, CoreError> errorMapper;
  late TypedWalletCore typedWalletCore;

  setUp(() async {
    if (!await isInitialized()) {
      await core.crateApiFullInit();
    }

    errorMapper = MockMapper();
    typedWalletCore = TypedWalletCore(errorMapper);

    // Setup default error mock
    when(errorMapper.map(any)).thenAnswer((invocation) {
      return CoreGenericError(invocation.positionalArguments.first);
    });
  });

  group('isValidPin', () {
    test('pin validation is passed on to core', () async {
      when(core.crateApiFullIsValidPin(pin: _kSamplePin)).thenAnswer((realInvocation) async => PinValidationResult.Ok);
      final result = await typedWalletCore.isValidPin(_kSamplePin);
      expect(result, PinValidationResult.Ok);
      verify(core.crateApiFullIsValidPin(pin: _kSamplePin)).called(1);
    });
  });

  group('register', () {
    test('register is passed on to core', () async {
      await typedWalletCore.register(_kSamplePin);
      verify(core.crateApiFullRegister(pin: _kSamplePin)).called(1);
    });
  });

  group('isRegistered', () {
    test('registration check is passed on to core', () async {
      await typedWalletCore.isRegistered();
      verify(core.crateApiFullHasRegistration()).called(1);
    });
  });

  group('lockWallet', () {
    test('lock wallet is passed on to core', () async {
      await typedWalletCore.lockWallet();
      verify(core.crateApiFullLockWallet()).called(1);
    });
  });

  group('unlockWallet', () {
    test('unlock wallet is passed on to core', () async {
      await typedWalletCore.unlockWallet(_kSamplePin);
      verify(core.crateApiFullUnlockWallet(pin: _kSamplePin)).called(1);
    });
  });

  group('isLocked', () {
    test('locked state is fetched through core by setting the lock stream', () async {
      // Verify we don't observe the stream pre-emptively
      verifyNever(core.crateApiFullSetLockStream());
      // But make sure we do call into the core once we check the isLocked stream
      await typedWalletCore.isLocked.first;
      verify(core.crateApiFullSetLockStream()).called(1);
    });
  });

  group('createdPidIssuanceUri', () {
    test('create pid issuance redirect uri is passed on to core', () async {
      await typedWalletCore.createPidIssuanceRedirectUri();
      verify(core.crateApiFullCreatePidIssuanceRedirectUri()).called(1);
    });
  });

  group('identifyUri', () {
    test('identify uri is passed on to core', () async {
      const uri = 'https://example.org';
      await typedWalletCore.identifyUri(uri);
      verify(core.crateApiFullIdentifyUri(uri: uri)).called(1);
    });
  });

  group('cancelPidIssuance', () {
    test('cancel pid issuance is passed on to core', () async {
      await typedWalletCore.cancelPidIssuance();
      verify(core.crateApiFullCancelIssuance()).called(1);
    });
  });

  group('observeConfig', () {
    test('configuration is fetched through core by setting the configuration stream', () async {
      when(core.crateApiFullSetConfigurationStream()).thenAnswer(
        (_) => Stream.value(
          FlutterConfiguration(
            inactiveWarningTimeout: 0,
            inactiveLockTimeout: 0,
            backgroundLockTimeout: 0,
            version: BigInt.zero,
          ),
        ),
      );
      // Verify we don't observe the stream pre-emptively
      verifyNever(core.crateApiFullSetConfigurationStream());
      // But make sure we do call into the core once we check the configuration stream
      await typedWalletCore.observeConfig().first;
      verify(core.crateApiFullSetConfigurationStream()).called(1);
    });
  });

  group('observeVersionState', () {
    test('version state is fetched through core by setting the version state stream', () async {
      when(core.crateApiFullSetVersionStateStream()).thenAnswer(
        (_) => Stream.value(const FlutterVersionState.ok()),
      );
      // Verify we don't observe the stream pre-emptively
      verifyNever(core.crateApiFullSetVersionStateStream());
      // But make sure we do call into the core once we check the version state stream
      await typedWalletCore.observeVersionState().first;
      verify(core.crateApiFullSetVersionStateStream()).called(1);
    });
  });

  group('acceptOfferedPid', () {
    test('accept offered pid is passed on to core', () async {
      await typedWalletCore.acceptOfferedPid(_kSamplePin);
      verify(core.crateApiFullAcceptIssuance(pin: _kSamplePin)).called(1);
    });
  });

  group('resetWallet', () {
    test('reset wallet pid is passed on to core', () async {
      await typedWalletCore.resetWallet();
      verify(core.crateApiFullResetWallet()).called(1);
    });
  });

  group('observeAttestations', () {
    test('observeAttestations should fetch attestation through WalletCore', () {
      final List<Attestation> mockAttestations = [
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
      ];
      when(core.crateApiFullSetAttestationsStream()).thenAnswer((realInvocation) => Stream.value(mockAttestations));

      expect(
        TypedWalletCore(errorMapper).observeCards(),
        emitsInOrder([hasLength(mockAttestations.length)]),
      );
    });

    test('observeAttestations should emit a new value when WalletCore exposes new attestations', () {
      final List<Attestation> initialCards = [
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
      ];
      final List<Attestation> updatedCards = [
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
      ];
      when(core.crateApiFullSetAttestationsStream())
          .thenAnswer((realInvocation) => Stream.fromIterable([[], initialCards, updatedCards]));

      expect(
        TypedWalletCore(errorMapper).observeCards(),
        emitsInOrder([hasLength(0), hasLength(initialCards.length), hasLength(updatedCards.length)]),
      );
    });

    test('observeAttestations should emit only the last value on a new subscription', () async {
      final List<Attestation> initialCards = [
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
      ];
      final List<Attestation> updatedCards = [
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
        const Attestation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
        ),
      ];
      when(core.crateApiFullSetAttestationsStream())
          .thenAnswer((realInvocation) => Stream.fromIterable([initialCards, updatedCards]));

      final typedCore = TypedWalletCore(errorMapper);

      /// This makes sure the observeCards() had a chance initialize
      await typedCore.observeCards().take(2).last;

      /// On a new subscription we now only expect to see the last value
      expect(typedCore.observeCards(), emitsInOrder([hasLength(updatedCards.length)]));
    });
  });

  ///Verify that methods convert potential [FfiException]s into the expected [CoreError]s
  group('handleCoreException', () {
    /// Create a [FfiException] that should be converted to a [CoreError]
    final flutterApiError = FlutterApiError(type: FlutterApiErrorType.generic, description: null, data: null);
    final ffiException = AnyhowException(jsonEncode(flutterApiError));

    test('isValidPin', () async {
      when(core.crateApiFullIsValidPin(pin: _kSamplePin)).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.isValidPin(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('register', () async {
      when(core.crateApiFullRegister(pin: _kSamplePin)).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.register(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('isRegistered', () async {
      when(core.crateApiFullHasRegistration()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.isRegistered(), throwsA(isA<CoreError>()));
    });

    test('lockWallet', () async {
      when(core.crateApiFullLockWallet()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.lockWallet(), throwsA(isA<CoreError>()));
    });

    test('unlockWallet', () async {
      when(core.crateApiFullUnlockWallet(pin: _kSamplePin)).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.unlockWallet(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('createPidIssuanceRedirectUri', () async {
      when(core.crateApiFullCreatePidIssuanceRedirectUri()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.createPidIssuanceRedirectUri(), throwsA(isA<CoreError>()));
    });

    test('identifyUri', () async {
      when(core.crateApiFullIdentifyUri(uri: 'https://example.org')).thenThrow(ffiException);
      expect(() => typedWalletCore.identifyUri('https://example.org'), throwsA(isA<CoreError>()));
    });

    test('cancelPidIssuance', () async {
      when(core.crateApiFullCancelIssuance()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.cancelPidIssuance(), throwsA(isA<CoreError>()));
    });

    test('acceptOfferedPid', () async {
      when(core.crateApiFullAcceptIssuance(pin: _kSamplePin)).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.acceptOfferedPid(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('resetWallet', () async {
      when(core.crateApiFullResetWallet()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.resetWallet(), throwsA(isA<CoreError>()));
    });
  });
}
