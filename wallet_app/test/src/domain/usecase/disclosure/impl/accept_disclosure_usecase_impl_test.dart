import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/model/result/application_error.dart';
import 'package:wallet/src/domain/usecase/disclosure/accept_disclosure_usecase.dart';
import 'package:wallet/src/domain/usecase/disclosure/impl/accept_disclosure_usecase_impl.dart';
import 'package:wallet/src/wallet_core/error/core_error.dart';
import 'package:wallet_core/core.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late AcceptDisclosureUseCase usecase;
  final mockRepo = Mocks.create<DisclosureRepository>() as MockDisclosureRepository;

  setUp(() {
    provideDummy<AcceptDisclosureResult>(AcceptDisclosureResult.ok());
    usecase = AcceptDisclosureUseCaseImpl(mockRepo);
  });

  test('when acceptDisclosure throws, result is error', () async {
    when(mockRepo.acceptDisclosure(any)).thenAnswer((_) async => throw const CoreGenericError('expected error'));
    final result = await usecase.invoke('123123');
    expect(result.hasError, isTrue);
  });

  test('when acceptDisclosure returns instruction error, result is error', () async {
    when(mockRepo.acceptDisclosure(any)).thenThrow(
      WalletInstructionError.incorrectPin(attemptsLeftInRound: 3, isFinalRound: false),
    );
    final result = await usecase.invoke('123123');
    expect(result.hasError, isTrue);
    expect(result.error, isA<CheckPinError>());
  });

  test('when acceptDisclosure succeeds, result is ok', () async {
    when(mockRepo.acceptDisclosure(any)).thenAnswer((_) async => 'https://example.org');
    final result = await usecase.invoke('123123');
    expect(result.hasError, isFalse);
    expect(result.value, 'https://example.org');
  });
}
