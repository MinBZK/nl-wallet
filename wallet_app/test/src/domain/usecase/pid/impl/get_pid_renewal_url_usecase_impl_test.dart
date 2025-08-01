import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/get_pid_renewal_url_usecase.dart';
import 'package:wallet/src/domain/usecase/pid/impl/get_pid_renewal_url_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

const _kMockUrl = 'https://renewal.org';

void main() {
  late GetPidRenewalUrlUseCase usecase;
  final PidRepository mockRepo = Mocks.create();

  setUp(() {
    usecase = GetPidRenewalUrlUseCaseImpl(mockRepo);
  });

  group('PidRenewal Url', () {
    test('pid renewal url should be fetched through the Repository', () async {
      // Setup: Mock url response
      when(mockRepo.getPidRenewalUrl()).thenAnswer((_) async => _kMockUrl);
      // Act: Invoke the use case to fetch the PID renewal url
      final url = await usecase.invoke();
      // Verify: repo called and expected url returned
      verify(mockRepo.getPidRenewalUrl());
      expect(url.value, _kMockUrl);
    });
  });
}
