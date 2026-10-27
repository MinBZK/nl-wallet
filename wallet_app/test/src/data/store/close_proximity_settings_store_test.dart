import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wallet/src/data/store/close_proximity_settings_store.dart';
import 'package:wallet/src/domain/usecase/wallet/impl/reset_wallet_usecase_impl.dart';

import '../../mocks/wallet_mocks.dart';

void main() {
  test('strict mode defaults on, persists changes, and resets with the wallet', () async {
    SharedPreferences.setMockInitialValues({});
    final store = CloseProximitySettingsStore(SharedPreferences.getInstance);
    expect(await store.getWrprcStrictMode(), isTrue);
    await store.setWrprcStrictMode(enabled: false);
    final restored = CloseProximitySettingsStore(SharedPreferences.getInstance);
    expect(await restored.getWrprcStrictMode(), isFalse);
    await ResetWalletUseCaseImpl(MockWalletRepository(), SharedPreferences.getInstance).invoke();
    expect(await store.getWrprcStrictMode(), isTrue);
  });
}
