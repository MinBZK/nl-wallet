import 'package:rxdart/rxdart.dart';

import '../../../store/tour_store.dart';
import '../tour_repository.dart';

class TourRepositoryImpl extends TourRepository {
  final TourStore _tourStore;

  final BehaviorSubject<bool> _showTourBannerStream = BehaviorSubject();

  TourRepositoryImpl(this._tourStore) {
    _tourStore.getShowTourBanner().then(_showTourBannerStream.add);
  }

  @override
  Future<void> setShowTourBanner({required bool showTourBanner}) async {
    await _tourStore.setShowTourBanner(showTourBanner: showTourBanner);
    _showTourBannerStream.add(showTourBanner);
  }

  @override
  Stream<bool> get showTourBanner => _showTourBannerStream.distinct();
}
