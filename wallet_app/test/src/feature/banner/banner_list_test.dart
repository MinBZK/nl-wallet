import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:wallet/src/domain/model/update/version_state.dart';
import 'package:wallet/src/feature/banner/banner_list.dart';
import 'package:wallet/src/feature/banner/cubit/banner_cubit.dart';
import 'package:wallet/src/feature/banner/wallet_banner.dart';

import '../../../wallet_app_test_widget.dart';
import '../../mocks/wallet_mock_data.dart';
import '../../test_util/golden_utils.dart';

class MockBannerCubit extends MockCubit<List<WalletBanner>> implements BannerCubit {}

const kSingleItemSize = Size(390, 100);
const kMultiItemSize = Size(390, 180);

const tourBanner = TourSuggestionBanner();
final updateNotifyBanner = UpdateAvailableBanner(state: VersionStateNotify());
final updateRecommendBanner = UpdateAvailableBanner(state: VersionStateRecommend());
final cardExpiryWarningBanner = CardExpiresSoonBanner(
  card: WalletMockData.altCard,
  expiresAt: DateTime.now().add(const Duration(days: 4, minutes: 1)),
);
final cardExpiredBanner = CardExpiredBanner(card: WalletMockData.card);
final cardRevokedBanner = CardRevokedBanner(card: WalletMockData.card);

void main() {
  // Common setup for widget pumping
  Future<void> pumpBannerList(
    WidgetTester tester, {
    required List<WalletBanner> initialBanners,
    Brightness brightness = Brightness.light,
    Size surfaceSize = kMultiItemSize,
    double textScaleSize = 1.0,
  }) async {
    final mockBannerCubit = MockBannerCubit();
    when(() => mockBannerCubit.state).thenAnswer((_) => initialBanners);

    await tester.pumpWidgetWithAppWrapper(
      const BannerList().withState<BannerCubit, List<WalletBanner>>(mockBannerCubit, initialBanners),
      surfaceSize: surfaceSize,
      brightness: brightness,
      textScaleSize: textScaleSize,
    );

    /// Pump the animated list
    await tester.pumpAndSettle(const Duration(milliseconds: 500));
  }

  group('BannerList Goldens', () {
    testGoldens('ltc14 ltc42 empty list - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [],
        surfaceSize: const Size(390, 20),
      );
      await screenMatchesGolden('banner_list.empty.light');
    });

    testGoldens('ltc14 ltc42 single tour banner - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [tourBanner],
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_tour.light');
    });

    testGoldens('ltc14 ltc42 single update banner (notify) - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [updateNotifyBanner],
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_update_notify.light');
    });

    testGoldens('ltc14 ltc42 multiple banners (tour and update) - light theme', (tester) async {
      await pumpBannerList(tester, initialBanners: [tourBanner, updateRecommendBanner]);
      await screenMatchesGolden('banner_list.multiple_banners.light');
    });

    testGoldens('ltc14 ltc42 multiple banners (update and tour) - light theme', (tester) async {
      await pumpBannerList(tester, initialBanners: [updateRecommendBanner, tourBanner]);
      await screenMatchesGolden('banner_list.multiple_banners_alt_order.light');
    });

    testGoldens('ltc71 multiple banners (expiry, expired, revoked) - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [
          cardExpiredBanner,
          cardExpiryWarningBanner,
          cardRevokedBanner,
        ],
        surfaceSize: const Size(390, 310),
      );
      await screenMatchesGolden('banner_list.multiple_expiry_banners.light');
    });

    testGoldens('ltc71 multiple banners (expiry, expired, revoked) - dark theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [
          cardExpiryWarningBanner,
          cardExpiredBanner,
          cardRevokedBanner,
        ],
        brightness: Brightness.dark,
        surfaceSize: const Size(390, 310),
      );
      await screenMatchesGolden('banner_list.multiple_expiry_banners.dark');
    });

    testGoldens('ltc14 ltc42 single tour banner - dark theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [tourBanner],
        brightness: Brightness.dark,
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_tour.dark');
    });

    testGoldens('ltc14 ltc42 multiple banners - dark theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [updateNotifyBanner, tourBanner],
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('banner_list.multiple_banners.dark');
    });

    testGoldens('ltc14 ltc42 single tour banner - text scaled', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [tourBanner],
        textScaleSize: 1.5,
        surfaceSize: const Size(390, 130),
      );
      await screenMatchesGolden('banner_list.single_tour.scaled.light');
    });
  });
}
