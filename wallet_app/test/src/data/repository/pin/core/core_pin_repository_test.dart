import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/pin/core/core_pin_repository.dart';
import 'package:wallet/src/domain/model/pin/pin_validation_error.dart';
import 'package:wallet/src/util/mapper/pin/pin_validation_error_mapper.dart';
import 'package:wallet/src/wallet_core/typed/typed_wallet_core.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

const _kValidPin = '112233';

void main() {
  late CorePinRepository repo;
  final MockTypedWalletCore core = Mocks.create<TypedWalletCore>() as MockTypedWalletCore;
  late BehaviorSubject<bool> mockLockedStream;

  setUp(() {
    var registered = false;
    mockLockedStream = BehaviorSubject.seeded(true);
    when(core.isLocked).thenAnswer((_) => mockLockedStream);
    when(core.isRegistered()).thenAnswer((_) async => registered);
    when(core.register(any)).thenAnswer((_) async {
      registered = true;
      mockLockedStream.add(false);
    });
    when(core.unlockWallet(any)).thenAnswer((realInvocation) async {
      final pinIsValid = realInvocation.positionalArguments.first == _kValidPin;
      mockLockedStream.add(!pinIsValid);
      if (pinIsValid) return const WalletInstructionResult.ok();
      return const WalletInstructionResult.instructionError(
        error: WalletInstructionError.incorrectPin(
          attemptsLeftInRound: 3,
          isFinalRound: false,
        ),
      );
    });
    when(core.lockWallet()).thenAnswer((realInvocation) async => mockLockedStream.add(true));
    repo = CorePinRepository(core, PinValidationErrorMapper());
  });
  group('pin validation', () {
    test('checking invalid pin results in a thrown PinValidationError', () async {
      when(core.isValidPin(any)).thenAnswer((realInvocation) async => PinValidationResult.TooFewUniqueDigits);
      expect(() async => repo.validatePin('000000'), throwsA(isA<PinValidationError>()));
    });

    test('checking a valid pin completes without throwing', () async {
      when(core.isValidPin(any)).thenAnswer((realInvocation) async => PinValidationResult.Ok);
      expect(repo.validatePin('112233'), completes);
    });
  });

  test('call to checkPin is passed through to the core', () async {
    when(core.isRegistered()).thenAnswer((_) async => true);
    await repo.checkPin('123123');
    verify(core.checkPin('123123')).called(1);
  });

  test('call to checkPin throws when not registered', () async {
    await expectLater(() async => repo.checkPin('143245'), throwsA(isA<StateError>()));
  });

  test('call to changePin is passed through to the core', () async {
    when(core.isRegistered()).thenAnswer((_) async => true);
    await repo.changePin('123123', '321321');
    verify(core.changePin('123123', '321321')).called(1);
  });

  test('call to changePin throws when not registered', () async {
    await expectLater(() async => repo.changePin('143242', '324942'), throwsA(isA<StateError>()));
  });

  test('call to continueChangePin is passed through to the core', () async {
    when(core.isRegistered()).thenAnswer((_) async => true);
    await repo.continueChangePin('321321');
    verify(core.continueChangePin('321321')).called(1);
  });

  test('call to continueChangePin throws when not registered', () async {
    await expectLater(() async => repo.continueChangePin('324942'), throwsA(isA<StateError>()));
  });
}
