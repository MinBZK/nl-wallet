import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mocktail/mocktail.dart';
import 'package:wallet/src/domain/model/update/version_state.dart';
import 'package:wallet/src/feature/banner/banner_list.dart';
import 'package:wallet/src/feature/banner/cubit/banner_cubit.dart';
import 'package:wallet/src/feature/banner/wallet_banner.dart';

import '../../../wallet_app_test_widget.dart';
import '../../test_util/golden_utils.dart';

class MockBannerCubit extends MockCubit<List<WalletBanner>> implements BannerCubit {}

const kSingleItemSize = Size(390, 100);
const kMultiItemSize = Size(390, 180);

const tourBanner = TourSuggestionBanner();
final updateNotifyBanner = UpdateAvailableBanner(state: VersionStateNotify());
final updateRecommendBanner = UpdateAvailableBanner(state: VersionStateRecommend());

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
    testGoldens('empty list - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [],
        surfaceSize: const Size(390, 20),
      );
      await screenMatchesGolden('banner_list.empty.light');
    });

    testGoldens('single tour banner - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [tourBanner],
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_tour.light');
    });

    testGoldens('single update banner (notify) - light theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [updateNotifyBanner],
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_update_notify.light');
    });

    testGoldens('multiple banners (tour and update) - light theme', (tester) async {
      await pumpBannerList(tester, initialBanners: [tourBanner, updateRecommendBanner]);
      await screenMatchesGolden('banner_list.multiple_banners.light');
    });

    testGoldens('multiple banners (update and tour) - light theme', (tester) async {
      await pumpBannerList(tester, initialBanners: [updateRecommendBanner, tourBanner]);
      await screenMatchesGolden('banner_list.multiple_banners_alt_order.light');
    });

    testGoldens('single tour banner - dark theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [tourBanner],
        brightness: Brightness.dark,
        surfaceSize: kSingleItemSize,
      );
      await screenMatchesGolden('banner_list.single_tour.dark');
    });

    testGoldens('multiple banners - dark theme', (tester) async {
      await pumpBannerList(
        tester,
        initialBanners: [updateNotifyBanner, tourBanner],
        brightness: Brightness.dark,
      );
      await screenMatchesGolden('banner_list.multiple_banners.dark');
    });

    testGoldens('single tour banner - text scaled', (tester) async {
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
