import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/tour/tour_repository.dart';
import 'package:wallet/src/domain/usecase/tour/impl/tour_overview_viewed_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/tour/tour_overview_viewed_usecase.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late TourRepository mockTourRepository;

  late TourOverviewViewedUseCase usecase;

  setUp(() {
    mockTourRepository = MockTourRepository();

    usecase = TourOverviewViewedUseCaseImpl(
      mockTourRepository,
    );
  });

  group('invoke', () {
    test('should call `setShowTourBanner` in repo on invoke', () async {
      await usecase.invoke();
      verify(mockTourRepository.setShowTourBanner(showTourBanner: false)).called(1);
    });
  });
}
