import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/domain/usecase/auth/get_digid_auth_url_usecase.dart';
import 'package:wallet/src/domain/usecase/auth/impl/get_digid_auth_url_usecase_impl.dart';

import '../../../../mocks/wallet_mocks.dart';

void main() {
  late GetDigidAuthUrlUseCase usecase;
  final mockRepo = Mocks.create<MockDigidAuthRepository>();

  setUp(() {
    usecase = GetDigidAuthUrlUseCaseImpl(mockRepo);
  });

  group('DigiD Auth Url', () {
    test('auth url should be fetched through the Repository', () async {
      final url = await usecase.invoke();
      verify(mockRepo.getAuthUrl());
      expect(url, kMockDigidAuthUrl);
    });
  });
}
