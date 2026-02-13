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
  /// WalletCore.init can only be called once, so setting up and assigning mock here.
  final MockWalletCoreApi core = MockWalletCoreApi();
  WalletCore.initMock(api: core);

  late MockMapper<String, CoreError> errorMapper;
  late TypedWalletCore typedWalletCore;

  setUp(() async {
    errorMapper = MockMapper();
    typedWalletCore = TypedWalletCore(errorMapper);

    // Setup default error mock
    when(errorMapper.map(any)).thenAnswer((invocation) {
      return CoreGenericError(invocation.positionalArguments.first);
    });
  });

  tearDown(() => clearInteractions(core));

  group('postInit', () {
    test('defaults to uninitialized and does not clear streams', () async {
      final initialized = await isInitialized(); // Call #1
      expect(initialized, isFalse);
      await postInit(); // Call #2
      verify(core.crateApiFullIsInitialized()).called(2);

      // Verify clear methods are NOT called
      verifyNoMoreInteractions(core);
    });

    test('streams are cleared when when postInit is called after previous initialization', () async {
      when(core.crateApiFullIsInitialized()).thenAnswer((_) async => true);
      final initialized = await isInitialized(); // Call #1
      expect(initialized, isTrue);
      await postInit(); // Call #2
      verify(core.crateApiFullIsInitialized()).called(2);

      // Verify clear stream methods are called
      verify(core.crateApiFullClearLockStream()).called(1);
      verify(core.crateApiFullClearConfigurationStream()).called(1);
      verify(core.crateApiFullClearVersionStateStream()).called(1);
      verify(core.crateApiFullClearAttestationsStream()).called(1);
      verify(core.crateApiFullClearRecentHistoryStream()).called(1);
      verify(core.crateApiFullClearScheduledNotificationsStream()).called(1);
      verify(core.crateApiFullClearDirectNotificationsCallback()).called(1);

      // Make sure wallet starts out locked
      verify(core.crateApiFullLockWallet()).called(1);
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
      await typedWalletCore.cancelIssuance();
      verify(core.crateApiFullCancelIssuance()).called(1);
    });
  });

  group('observeConfig', () {
    test('configuration is fetched through core by setting the configuration stream', () async {
      when(core.crateApiFullSetConfigurationStream()).thenAnswer(
        (_) => Stream.value(
          const FlutterConfiguration(
            inactiveWarningTimeout: 0,
            inactiveLockTimeout: 0,
            backgroundLockTimeout: 0,
            pidAttestationTypes: [],
            staticAssetsBaseUrl: '',
            version: '0',
            environment: 'test',
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
      await typedWalletCore.acceptIssuance(_kSamplePin);
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
      final List<AttestationPresentation> mockAttestations = [
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
      ];
      when(core.crateApiFullSetAttestationsStream()).thenAnswer((realInvocation) => Stream.value(mockAttestations));

      expect(
        TypedWalletCore(errorMapper).observeCards(),
        emitsInOrder([hasLength(mockAttestations.length)]),
      );
    });

    test('observeAttestations should emit a new value when WalletCore exposes new attestations', () {
      final List<AttestationPresentation> initialCards = [
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
      ];
      final List<AttestationPresentation> updatedCards = [
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
      ];
      when(
        core.crateApiFullSetAttestationsStream(),
      ).thenAnswer((realInvocation) => Stream.fromIterable([[], initialCards, updatedCards]));

      expect(
        TypedWalletCore(errorMapper).observeCards(),
        emitsInOrder([hasLength(0), hasLength(initialCards.length), hasLength(updatedCards.length)]),
      );
    });

    test('observeAttestations should emit only the last value on a new subscription', () async {
      final List<AttestationPresentation> initialCards = [
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
      ];
      final List<AttestationPresentation> updatedCards = [
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_id',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
        const AttestationPresentation(
          identity: AttestationIdentity.fixed(id: '0'),
          attestationType: 'pid_address',
          displayMetadata: [],
          attributes: [],
          issuer: _kSampleIssuer,
          validityStatus: ValidityStatus_Valid(validUntil: null),
        ),
      ];
      when(
        core.crateApiFullSetAttestationsStream(),
      ).thenAnswer((realInvocation) => Stream.fromIterable([initialCards, updatedCards]));

      final typedCore = TypedWalletCore(errorMapper);

      /// This makes sure the observeCards() had a chance initialize
      await typedCore.observeCards().take(2).last;

      /// On a new subscription we now only expect to see the last value
      expect(typedCore.observeCards(), emitsInOrder([hasLength(updatedCards.length)]));
    });
  });

  group('notifications', () {
    test('observeNotifications is passed on to core', () async {
      when(core.crateApiFullSetScheduledNotificationsStream()).thenAnswer((_) => Stream.value([]));
      await typedWalletCore.observeNotifications().first;
      verify(core.crateApiFullSetScheduledNotificationsStream()).called(1);
    });

    test('setupNotificationCallback is passed on to core', () {
      typedWalletCore.setupNotificationCallback((_) {});
      verify(core.crateApiFullClearDirectNotificationsCallback()).called(1);
      verify(core.crateApiFullSetDirectNotificationsCallback(callback: anyNamed('callback'))).called(1);
    });
  });

  group('history', () {
    test('getHistory is passed on to core', () async {
      await typedWalletCore.getHistory();
      verify(core.crateApiFullGetHistory()).called(1);
    });

    test('getHistoryForCard is passed on to core', () async {
      await typedWalletCore.getHistoryForCard('test-id');
      verify(core.crateApiFullGetHistoryForCard(attestationId: 'test-id')).called(1);
    });

    test('observeRecentHistory is passed on to core', () async {
      when(core.crateApiFullSetRecentHistoryStream()).thenAnswer((_) => Stream.value([]));
      await typedWalletCore.observeRecentHistory().first;
      verify(core.crateApiFullSetRecentHistoryStream()).called(1);
    });
  });

  group('pin management', () {
    test('checkPin is passed on to core', () async {
      await typedWalletCore.checkPin(_kSamplePin);
      verify(core.crateApiFullCheckPin(pin: _kSamplePin)).called(1);
    });

    test('changePin is passed on to core', () async {
      await typedWalletCore.changePin(_kSamplePin, 'new-pin');
      verify(core.crateApiFullChangePin(oldPin: _kSamplePin, newPin: 'new-pin')).called(1);
    });

    test('continueChangePin is passed on to core', () async {
      await typedWalletCore.continueChangePin(_kSamplePin);
      verify(core.crateApiFullContinueChangePin(pin: _kSamplePin)).called(1);
    });
  });

  group('biometrics', () {
    test('isBiometricLoginEnabled is passed on to core', () async {
      await typedWalletCore.isBiometricLoginEnabled();
      verify(core.crateApiFullIsBiometricUnlockEnabled()).called(1);
    });

    test('setBiometricUnlock is passed on to core', () async {
      await typedWalletCore.setBiometricUnlock(enabled: true);
      verify(core.crateApiFullSetBiometricUnlock(enable: true)).called(1);
    });

    test('unlockWithBiometrics is passed on to core', () async {
      await typedWalletCore.unlockWithBiometrics();
      verify(core.crateApiFullUnlockWalletWithBiometrics()).called(1);
    });
  });

  group('pid and issuance', () {
    test('createPidRenewalRedirectUri is passed on to core', () async {
      await typedWalletCore.createPidRenewalRedirectUri();
      verify(core.crateApiFullCreatePidRenewalRedirectUri()).called(1);
    });

    test('continuePidIssuance is passed on to core', () async {
      await typedWalletCore.continuePidIssuance('uri');
      verify(core.crateApiFullContinuePidIssuance(uri: 'uri')).called(1);
    });

    test('continueDisclosureBasedIssuance is passed on to core', () async {
      await typedWalletCore.continueDisclosureBasedIssuance(_kSamplePin, [1, 2]);
      verify(core.crateApiFullContinueDisclosureBasedIssuance(pin: _kSamplePin, selectedIndices: [1, 2])).called(1);
    });

    test('acceptPidIssuance is passed on to core', () async {
      await typedWalletCore.acceptPidIssuance(_kSamplePin);
      verify(core.crateApiFullAcceptPidIssuance(pin: _kSamplePin)).called(1);
    });
  });

  group('disclosure', () {
    test('startDisclosure is passed on to core', () async {
      await typedWalletCore.startDisclosure('uri', isQrCode: true);
      verify(core.crateApiFullStartDisclosure(uri: 'uri', isQrCode: true)).called(1);
    });

    test('cancelDisclosure is passed on to core', () async {
      await typedWalletCore.cancelDisclosure();
      verify(core.crateApiFullCancelDisclosure()).called(1);
    });

    test('acceptDisclosure is passed on to core', () async {
      await typedWalletCore.acceptDisclosure(_kSamplePin, [1, 2]);
      verify(core.crateApiFullAcceptDisclosure(pin: _kSamplePin, selectedIndices: [1, 2])).called(1);
    });
  });

  group('pin recovery', () {
    test('createPinRecoveryRedirectUri is passed on to core', () async {
      await typedWalletCore.createPinRecoveryRedirectUri();
      verify(core.crateApiFullCreatePinRecoveryRedirectUri()).called(1);
    });

    test('continuePinRecovery is passed on to core', () async {
      await typedWalletCore.continuePinRecovery('uri');
      verify(core.crateApiFullContinuePinRecovery(uri: 'uri')).called(1);
    });

    test('completePinRecovery is passed on to core', () async {
      await typedWalletCore.completePinRecovery(_kSamplePin);
      verify(core.crateApiFullCompletePinRecovery(pin: _kSamplePin)).called(1);
    });

    test('cancelPinRecovery is passed on to core', () async {
      await typedWalletCore.cancelPinRecovery();
      verify(core.crateApiFullCancelPinRecovery()).called(1);
    });
  });

  group('wallet transfer', () {
    test('initWalletTransfer is passed on to core', () async {
      await typedWalletCore.initWalletTransfer();
      verify(core.crateApiFullInitWalletTransfer()).called(1);
    });

    test('pairWalletTransfer is passed on to core', () async {
      await typedWalletCore.pairWalletTransfer('uri');
      verify(core.crateApiFullPairWalletTransfer(uri: 'uri')).called(1);
    });

    test('confirmWalletTransfer is passed on to core', () async {
      await typedWalletCore.confirmWalletTransfer(_kSamplePin);
      verify(core.crateApiFullConfirmWalletTransfer(pin: _kSamplePin)).called(1);
    });

    test('transferWallet is passed on to core', () async {
      await typedWalletCore.transferWallet();
      verify(core.crateApiFullTransferWallet()).called(1);
    });

    test('receiveWalletTransfer is passed on to core', () async {
      await typedWalletCore.receiveWalletTransfer();
      verify(core.crateApiFullReceiveWalletTransfer()).called(1);
    });

    test('cancelWalletTransfer is passed on to core', () async {
      await typedWalletCore.cancelWalletTransfer();
      verify(core.crateApiFullCancelWalletTransfer()).called(1);
    });

    test('getWalletTransferState is passed on to core', () async {
      await typedWalletCore.getWalletTransferState();
      verify(core.crateApiFullGetWalletTransferState()).called(1);
    });

    test('skipWalletTransfer is passed on to core', () async {
      await typedWalletCore.skipWalletTransfer();
      verify(core.crateApiFullSkipWalletTransfer()).called(1);
    });
  });

  group('misc', () {
    test('getVersionString is passed on to core', () async {
      await typedWalletCore.getVersionString();
      verify(core.crateApiFullGetVersionString()).called(1);
    });

    test('getWalletState is passed on to core', () async {
      await typedWalletCore.getWalletState();
      verify(core.crateApiFullGetWalletState()).called(1);
    });

    test('getRegistrationRevocationCode is passed on to core', () async {
      await typedWalletCore.getRegistrationRevocationCode();
      verify(core.crateApiFullGetRegistrationRevocationCode()).called(1);
    });

    test('getRevocationCode is passed on to core', () async {
      await typedWalletCore.getRevocationCode(_kSamplePin);
      verify(core.crateApiFullGetRevocationCode(pin: _kSamplePin)).called(1);
    });
  });

  ///Verify that methods convert potential [FfiException]s into the expected [CoreError]s
  group('handleCoreException', () {
    /// Create a [FfiException] that should be converted to a [CoreError]
    final flutterApiError = const FlutterApiError(type: FlutterApiErrorType.generic, description: null, data: null);
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
      expect(() async => typedWalletCore.cancelIssuance(), throwsA(isA<CoreError>()));
    });

    test('acceptOfferedPid', () async {
      when(core.crateApiFullAcceptIssuance(pin: _kSamplePin)).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.acceptIssuance(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('resetWallet', () async {
      when(core.crateApiFullResetWallet()).thenAnswer((_) async => throw ffiException);
      expect(() async => typedWalletCore.resetWallet(), throwsA(isA<CoreError>()));
    });

    test('checkPin', () async {
      when(core.crateApiFullCheckPin(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.checkPin(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('changePin', () async {
      when(core.crateApiFullChangePin(oldPin: _kSamplePin, newPin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.changePin(_kSamplePin, _kSamplePin), throwsA(isA<CoreError>()));
    });

    test('continueChangePin', () async {
      when(core.crateApiFullContinueChangePin(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.continueChangePin(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('createPidRenewalRedirectUri', () async {
      when(core.crateApiFullCreatePidRenewalRedirectUri()).thenThrow(ffiException);
      expect(() => typedWalletCore.createPidRenewalRedirectUri(), throwsA(isA<CoreError>()));
    });

    test('continuePidIssuance', () async {
      when(core.crateApiFullContinuePidIssuance(uri: 'https://example.org')).thenThrow(ffiException);
      expect(() => typedWalletCore.continuePidIssuance('https://example.org'), throwsA(isA<CoreError>()));
    });

    test('continueDisclosureBasedIssuance', () async {
      when(
        core.crateApiFullContinueDisclosureBasedIssuance(pin: _kSamplePin, selectedIndices: []),
      ).thenThrow(ffiException);
      expect(() => typedWalletCore.continueDisclosureBasedIssuance(_kSamplePin, []), throwsA(isA<CoreError>()));
    });

    test('acceptDisclosure', () async {
      when(core.crateApiFullAcceptDisclosure(pin: _kSamplePin, selectedIndices: [])).thenThrow(ffiException);
      expect(() => typedWalletCore.acceptDisclosure(_kSamplePin, []), throwsA(isA<CoreError>()));
    });

    test('startDisclosure', () async {
      when(core.crateApiFullStartDisclosure(uri: 'https://example.org', isQrCode: false)).thenThrow(ffiException);
      expect(() => typedWalletCore.startDisclosure('https://example.org'), throwsA(isA<CoreError>()));
    });

    test('getHistory', () async {
      when(core.crateApiFullGetHistory()).thenThrow(ffiException);
      expect(() => typedWalletCore.getHistory(), throwsA(isA<CoreError>()));
    });

    test('getHistoryForCard', () async {
      when(core.crateApiFullGetHistoryForCard(attestationId: '0')).thenThrow(ffiException);
      expect(() => typedWalletCore.getHistoryForCard('0'), throwsA(isA<CoreError>()));
    });

    test('isBiometricLoginEnabled', () async {
      when(core.crateApiFullIsBiometricUnlockEnabled()).thenThrow(ffiException);
      expect(() => typedWalletCore.isBiometricLoginEnabled(), throwsA(isA<CoreError>()));
    });

    test('setBiometricUnlock', () async {
      when(core.crateApiFullSetBiometricUnlock(enable: true)).thenThrow(ffiException);
      expect(() => typedWalletCore.setBiometricUnlock(enabled: true), throwsA(isA<CoreError>()));
    });

    test('unlockWithBiometrics', () async {
      when(core.crateApiFullUnlockWalletWithBiometrics()).thenThrow(ffiException);
      expect(() => typedWalletCore.unlockWithBiometrics(), throwsA(isA<CoreError>()));
    });

    test('getVersionString', () async {
      when(core.crateApiFullGetVersionString()).thenThrow(ffiException);
      expect(() => typedWalletCore.getVersionString(), throwsA(isA<CoreError>()));
    });

    test('createPinRecoveryRedirectUri', () async {
      when(core.crateApiFullCreatePinRecoveryRedirectUri()).thenThrow(ffiException);
      expect(() => typedWalletCore.createPinRecoveryRedirectUri(), throwsA(isA<CoreError>()));
    });

    test('continuePinRecovery', () async {
      when(core.crateApiFullContinuePinRecovery(uri: 'https://example.org')).thenThrow(ffiException);
      expect(() => typedWalletCore.continuePinRecovery('https://example.org'), throwsA(isA<CoreError>()));
    });

    test('completePinRecovery', () async {
      when(core.crateApiFullCompletePinRecovery(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.completePinRecovery(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('cancelPinRecovery', () async {
      when(core.crateApiFullCancelPinRecovery()).thenThrow(ffiException);
      expect(() => typedWalletCore.cancelPinRecovery(), throwsA(isA<CoreError>()));
    });

    test('initWalletTransfer', () async {
      when(core.crateApiFullInitWalletTransfer()).thenThrow(ffiException);
      expect(() => typedWalletCore.initWalletTransfer(), throwsA(isA<CoreError>()));
    });

    test('pairWalletTransfer', () async {
      when(core.crateApiFullPairWalletTransfer(uri: 'https://example.org')).thenThrow(ffiException);
      expect(() => typedWalletCore.pairWalletTransfer('https://example.org'), throwsA(isA<CoreError>()));
    });

    test('confirmWalletTransfer', () async {
      when(core.crateApiFullConfirmWalletTransfer(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.confirmWalletTransfer(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('transferWallet', () async {
      when(core.crateApiFullTransferWallet()).thenThrow(ffiException);
      expect(() => typedWalletCore.transferWallet(), throwsA(isA<CoreError>()));
    });

    test('receiveWalletTransfer', () async {
      when(core.crateApiFullReceiveWalletTransfer()).thenThrow(ffiException);
      expect(() => typedWalletCore.receiveWalletTransfer(), throwsA(isA<CoreError>()));
    });

    test('cancelWalletTransfer', () async {
      when(core.crateApiFullCancelWalletTransfer()).thenThrow(ffiException);
      expect(() => typedWalletCore.cancelWalletTransfer(), throwsA(isA<CoreError>()));
    });

    test('getWalletTransferState', () async {
      when(core.crateApiFullGetWalletTransferState()).thenThrow(ffiException);
      expect(() => typedWalletCore.getWalletTransferState(), throwsA(isA<CoreError>()));
    });

    test('skipWalletTransfer', () async {
      when(core.crateApiFullSkipWalletTransfer()).thenThrow(ffiException);
      expect(() => typedWalletCore.skipWalletTransfer(), throwsA(isA<CoreError>()));
    });

    test('getWalletState', () async {
      when(core.crateApiFullGetWalletState()).thenThrow(ffiException);
      expect(() => typedWalletCore.getWalletState(), throwsA(isA<CoreError>()));
    });

    test('acceptPidIssuance', () async {
      when(core.crateApiFullAcceptPidIssuance(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.acceptPidIssuance(_kSamplePin), throwsA(isA<CoreError>()));
    });

    test('cancelDisclosure', () async {
      when(core.crateApiFullCancelDisclosure()).thenThrow(ffiException);
      expect(() => typedWalletCore.cancelDisclosure(), throwsA(isA<CoreError>()));
    });

    test('getRegistrationRevocationCode', () async {
      when(core.crateApiFullGetRegistrationRevocationCode()).thenThrow(ffiException);
      expect(() => typedWalletCore.getRegistrationRevocationCode(), throwsA(isA<CoreError>()));
    });

    test('getRevocationCode', () async {
      when(core.crateApiFullGetRevocationCode(pin: _kSamplePin)).thenThrow(ffiException);
      expect(() => typedWalletCore.getRevocationCode(_kSamplePin), throwsA(isA<CoreError>()));
    });
  });
}
