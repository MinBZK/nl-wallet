import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/src/data/repository/tour/tour_repository.dart';
import 'package:wallet/src/domain/usecase/tour/impl/observe_show_tour_banner_usecase_impl.dart';
import 'package:wallet/src/domain/usecase/tour/observe_show_tour_banner_usecase.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late TourRepository mockTourRepository;

  late ObserveShowTourBannerUseCase usecase;

  setUp(() {
    mockTourRepository = MockTourRepository();

    usecase = ObserveShowTourBannerUseCaseImpl(
      mockTourRepository,
    );
  });

  group('invoke', () {
    test('should return `showTourBanner` stream on invoke', () async {
      when(mockTourRepository.showTourBanner).thenAnswer(
        (_) => (() async* {
          yield true;
          await Future.delayed(const Duration(milliseconds: 200));
          yield false;
        })(),
      );

      await expectLater(usecase.invoke(), emitsInOrder([true, false]));
      verify(mockTourRepository.showTourBanner).called(1);
    });
  });
}
