import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/bridge_generated.dart';
import 'package:wallet/src/data/repository/authentication/digid_auth_repository.dart';
import 'package:wallet/src/domain/usecase/auth/impl/update_digid_auth_status_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/auth/update_digid_auth_status_usecase.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late UpdateDigidAuthStatusUseCase usecase;
  final DigidAuthRepository mockRepo = Mocks.create();

  setUp(() {
    usecase = UpdateDigidAuthStatusUseCaseImpl(mockRepo);
  });

  group('DigiD Status Updates', () {
    test('DigiD state should be passed on to the repository', () async {
      usecase.invoke(DigidState.Success);
      verify(mockRepo.notifyDigidStateUpdate(DigidState.Success));
    });
  });
}
