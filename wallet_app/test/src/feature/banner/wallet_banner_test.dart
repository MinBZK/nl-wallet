import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/domain/model/update/version_state.dart';
import 'package:wallet/src/feature/banner/wallet_banner.dart';

void main() {
  group('WalletBanner Equatable Tests', () {
    group('UpdateAvailableBanner', () {
      test('instances with the same state should be equal', () {
        final banner1 = UpdateAvailableBanner(state: VersionStateOk());
        final banner2 = UpdateAvailableBanner(state: VersionStateOk());
        expect(banner1, equals(banner2));
        expect(banner1.hashCode, equals(banner2.hashCode));
      });

      test('instances with different states should not be equal', () {
        final banner1 = UpdateAvailableBanner(state: VersionStateOk());
        final banner2 = UpdateAvailableBanner(state: VersionStateNotify());
        expect(banner1, isNot(equals(banner2)));
      });

      test('instances with different internal states should not be equal', () {
        final banner1 =
            UpdateAvailableBanner(state: VersionStateWarn(timeUntilBlocked: const Duration(milliseconds: 21)));
        final banner2 = UpdateAvailableBanner(state: VersionStateWarn(timeUntilBlocked: const Duration(seconds: 1337)));
        expect(banner1, isNot(equals(banner2)));
      });

      test('instance should not be equal to a different type of banner', () {
        final banner1 = UpdateAvailableBanner(state: VersionStateNotify());
        const banner2 = TourSuggestionBanner();
        expect(banner1, isNot(equals(banner2)));
      });
    });

    group('TourSuggestionBanner', () {
      test('instances should always be equal to each other', () {
        const banner1 = TourSuggestionBanner();
        const banner2 = TourSuggestionBanner();
        expect(banner1, equals(banner2));
        expect(banner1.hashCode, equals(banner2.hashCode));
      });
    });
  });
}
