import '../../../../data/repository/tour/tour_repository.dart';
import '../../../model/result/result.dart';
import '../tour_overview_viewed_usecase.dart';

class TourOverviewViewedUseCaseImpl extends TourOverviewViewedUseCase {
  final TourRepository _tourRepository;

  TourOverviewViewedUseCaseImpl(this._tourRepository);

  @override
  Future<Result<void>> invoke() => tryCatch(
        () async => _tourRepository.setShowTourBanner(showTourBanner: false),
        'Failed to persist tour banner state',
      );
}
