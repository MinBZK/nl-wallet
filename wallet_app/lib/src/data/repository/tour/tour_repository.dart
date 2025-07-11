abstract class TourRepository {
  Future<void> setShowTourBanner({required bool showTourBanner});

  Stream<bool> get showTourBanner;
}
