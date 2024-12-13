import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/version/version_state_repository.dart';
import 'package:wallet/src/domain/usecase/update/impl/observe_version_state_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/update/observe_version_state_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late BehaviorSubject<VersionState> mockVersionStateStream;
  late VersionStateRepository mockVersionStateRepository;

  late ObserveVersionStateUsecase usecase;

  setUp(() {
    mockVersionStateStream = BehaviorSubject<VersionState>();
    mockVersionStateRepository = MockVersionStateRepository();

    usecase = ObserveVersionStateUsecaseImpl(
      mockVersionStateRepository,
    );
  });

  group('invoke', () {
    test('should return version state on invoke', () async {
      mockVersionStateStream.add(VersionStateNotify());
      when(mockVersionStateRepository.observeVersionState()).thenAnswer((_) => mockVersionStateStream);

      final result = await usecase.invoke().first;
      expect(result, VersionStateNotify());

      verify(mockVersionStateRepository.observeVersionState()).called(1);
    });
  });
}
