import 'shared_preferences_provider.dart';

const _kWrprcStrictModeKey = 'close_proximity_wrprc_strict_mode';

/// Temporary interoperability preference for Close Proximity disclosure.
class CloseProximitySettingsStore {
  final PreferenceProvider _preferences;

  CloseProximitySettingsStore(this._preferences);

  Future<bool> getWrprcStrictMode() async {
    final preferences = await _preferences();
    // SharedPreferences updates its cache before a platform write succeeds. Read persisted state so a failed
    // attempt to disable strict mode cannot accidentally relax the next disclosure session.
    await preferences.reload();
    return preferences.getBool(_kWrprcStrictModeKey) ?? true;
  }

  Future<void> setWrprcStrictMode({required bool enabled}) async {
    final preferences = await _preferences();
    if (!await preferences.setBool(_kWrprcStrictModeKey, enabled)) {
      throw StateError('Could not save Close Proximity WRPRC strict mode');
    }
  }
}
