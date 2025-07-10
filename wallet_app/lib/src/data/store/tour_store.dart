abstract class TourStore {
  Future<bool> getShowTourBanner();

  Future<void> setShowTourBanner({required bool showTourBanner});
}
