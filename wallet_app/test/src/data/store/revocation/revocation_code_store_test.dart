import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wallet/src/data/store/impl/revocation_code_store_impl.dart';

void main() {
  late SharedPreferences sharedPreferences;
  late RevocationCodeStoreImpl store;

  setUp(() async {
    SharedPreferences.setMockInitialValues({});
    sharedPreferences = await SharedPreferences.getInstance();
    store = RevocationCodeStoreImpl(SharedPreferences.getInstance);
  });

  tearDown(SharedPreferences.resetStatic);

  group('RevocationCodeStore', () {
    test('getRevocationCodeSavedFlag returns value from preferences', () async {
      await sharedPreferences.setBool(kRevocationCodeSavedKey, true);

      final result = await store.getRevocationCodeSavedFlag();

      expect(result, true);
    });

    test('getRevocationCodeSavedFlag returns false if not present', () async {
      final result = await store.getRevocationCodeSavedFlag();

      expect(result, false);
    });

    test('setRevocationCodeSavedFlag sets value in preferences', () async {
      await store.setRevocationCodeSavedFlag(saved: true);

      final result = sharedPreferences.getBool(kRevocationCodeSavedKey);
      expect(result, true);
    });
  });
}
