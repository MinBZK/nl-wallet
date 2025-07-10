import '../shared_preferences_provider.dart';
import '../tour_store.dart';

const _kShowTourBannerKey = 'show_app_tour_banner';
const _kDefaultShowTourBanner = true;

class TourStoreImpl extends TourStore {
  final PreferenceProvider _preferences;

  TourStoreImpl(this._preferences);

  @override
  Future<bool> getShowTourBanner() async {
    final prefs = await _preferences.call();
    return prefs.getBool(_kShowTourBannerKey) ?? _kDefaultShowTourBanner;
  }

  @override
  Future<void> setShowTourBanner({required bool showTourBanner}) async {
    final prefs = await _preferences.call();
    await prefs.setBool(_kShowTourBannerKey, showTourBanner);
  }
}
