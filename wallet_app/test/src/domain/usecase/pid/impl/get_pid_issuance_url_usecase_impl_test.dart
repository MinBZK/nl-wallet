import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/get_pid_issuance_url_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/get_pid_issuance_url_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late GetPidIssuanceUrlUseCase usecase;
  final PidRepository mockRepo = Mocks.create();

  setUp(() {
    usecase = GetPidIssuanceUrlUseCaseImpl(mockRepo);
  });

  group('PidIssuance Url', () {
    test('pid issuance url should be fetched through the Repository', () async {
      final url = await usecase.invoke();
      verify(mockRepo.getPidIssuanceUrl());
      expect(url.value, kMockPidIssuanceUrl);
    });
  });
}
