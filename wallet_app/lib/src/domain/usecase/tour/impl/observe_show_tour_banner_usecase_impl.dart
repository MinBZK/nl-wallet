import '../../../../data/repository/tour/tour_repository.dart';
import '../observe_show_tour_banner_usecase.dart';

class ObserveShowTourBannerUseCaseImpl extends ObserveShowTourBannerUseCase {
  final TourRepository _tourRepository;

  ObserveShowTourBannerUseCaseImpl(this._tourRepository);

  @override
  Stream<bool> invoke() {
    return _tourRepository.showTourBanner;
  }
}
