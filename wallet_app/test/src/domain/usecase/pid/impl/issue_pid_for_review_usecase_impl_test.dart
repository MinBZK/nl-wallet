import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/pid/pid_repository.dart';
import 'package:wallet/src/domain/usecase/pid/impl/issue_pid_for_review_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/pid/issue_pid_for_review_usecase.dart';

import '../../../../mocks/wallet_mock_data.dart';
import '../../../../mocks/wallet_mocks.dart';

void main() {
  late IssuePidForReviewUseCase usecase;
  late MockPidRepository mockRepo;

  setUp(() {
    mockRepo = Mocks.create<PidRepository>() as MockPidRepository;
    usecase = IssuePidForReviewUseCaseImpl(mockRepo);
  });

  test('the attributes of the issued pid are returned', () async {
    when(mockRepo.issuePidForReview('123123')).thenAnswer((_) async => [WalletMockData.textDataAttribute]);

    final result = await usecase.invoke('123123');

    expect(result.hasError, isFalse);
    expect(result.value, [WalletMockData.textDataAttribute]);
  });

  test('errors from the repository are returned as error result', () async {
    when(mockRepo.issuePidForReview('123123')).thenAnswer((_) async => throw Exception('error'));

    final result = await usecase.invoke('123123');

    expect(result.hasError, isTrue);
  });
}
