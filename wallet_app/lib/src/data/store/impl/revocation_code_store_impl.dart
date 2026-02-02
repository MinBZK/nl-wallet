import 'package:flutter/cupertino.dart';

import '../revocation_code_store.dart';
import '../shared_preferences_provider.dart';

@visibleForTesting
const kRevocationCodeSavedKey = 'revocation_code_saved';
const _kDefaultRevocationCodeSaved = false;

class RevocationCodeStoreImpl extends RevocationCodeStore {
  final PreferenceProvider _preferences;

  RevocationCodeStoreImpl(this._preferences);

  @override
  Future<bool> getRevocationCodeSavedFlag() async {
    final prefs = await _preferences.call();
    return prefs.getBool(kRevocationCodeSavedKey) ?? _kDefaultRevocationCodeSaved;
  }

  @override
  Future<void> setRevocationCodeSavedFlag({required bool saved}) async {
    final prefs = await _preferences.call();
    await prefs.setBool(kRevocationCodeSavedKey, saved);
  }
}
