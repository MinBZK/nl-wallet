import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rxdart/rxdart.dart';
import 'package:wallet/src/data/repository/authentication/digid_auth_repository.dart';
import 'package:wallet/src/domain/usecase/auth/impl/observe_digid_auth_status_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/auth/observe_digid_auth_status_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late ObserveDigidAuthStatusUseCase usecase;
  final mockRepo = Mocks.create<DigidAuthRepository>() as MockDigidAuthRepository;
  late BehaviorSubject<DigidAuthStatus> mockStatusSubject;

  setUp(() {
    mockStatusSubject = BehaviorSubject<DigidAuthStatus>();
    when(mockRepo.observeAuthStatus()).thenAnswer((_) => mockStatusSubject);
    usecase = ObserveDigidAuthStatusUseCaseImpl(mockRepo);
  });

  group('DigiD Status Verification', () {
    test('Nothing should be emitted when the repo is not notified', () async {
      expectLater(usecase.invoke(), emitsInOrder([]));
    });

    test('The stream should be updated after notifying the repository', () async {
      expectLater(usecase.invoke(), emitsInOrder([DigidAuthStatus.authenticating]));
      mockStatusSubject.add(DigidAuthStatus.authenticating);
    });

    test('The idle state is expected to be consumed by the usecase', () async {
      expectLater(usecase.invoke(), emitsInOrder([DigidAuthStatus.authenticating, DigidAuthStatus.success]));
      mockStatusSubject.add(DigidAuthStatus.authenticating);
      mockStatusSubject.add(DigidAuthStatus.success);
      mockStatusSubject.add(DigidAuthStatus.idle);
    });
  });
}
