import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:wallet/l10n/generated/app_localizations.dart';
import 'package:wallet/src/data/repository/configuration/configuration_repository.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/domain/model/configuration/flutter_app_configuration.dart';
import 'package:wallet/src/domain/model/result/result.dart';
import 'package:wallet/src/domain/model/tour/tour_video.dart';
import 'package:wallet/src/domain/usecase/tour/impl/fetch_tour_videos_usecase_impl.dart';
import 'package:wallet/src/wallet_assets.dart';

import '../../../../mocks/wallet_mocks.mocks.dart';

void main() {
  late ConfigurationRepository mockConfigurationRepository;
  late FetchTourVideosUseCaseImpl useCase;

  const testBaseUrl = 'https://example.com/';
  final appConfig = const FlutterAppConfiguration(
    idleLockTimeout: Duration.zero,
    idleWarningTimeout: Duration.zero,
    backgroundLockTimeout: Duration.zero,
    staticAssetsBaseUrl: testBaseUrl,
    version: 0,
  );

  setUp(() {
    mockConfigurationRepository = MockConfigurationRepository();
    useCase = FetchTourVideosUseCaseImpl(mockConfigurationRepository);

    when(mockConfigurationRepository.appConfiguration).thenAnswer((_) => Stream.value(appConfig));
  });

  test('invoke returns success with a list of tour videos matching the amount of video_slugs', () async {
    final result = await useCase.invoke();

    expect(result, isA<Success<List<TourVideo>>>());
    final videos = result.value!;
    expect(videos.length, WalletAssets.video_slugs.length);
  });

  test('invoke constructs correct URLs for videos, thumbnails, and subtitles', () async {
    final result = await useCase.invoke();
    final videos = result.value!;

    for (var i = 0; i < videos.length; i++) {
      final video = videos[i];
      final slug = WalletAssets.video_slugs[i];

      // Check video URLs
      final expectedVideoUrlEn =
          '$testBaseUrl${WalletAssets.video_tour_video_url_placeholder.replaceAll('{languageCode}', 'en').replaceAll('{slug}', slug)}';
      final expectedVideoUrlNl =
          '$testBaseUrl${WalletAssets.video_tour_video_url_placeholder.replaceAll('{languageCode}', 'nl').replaceAll('{slug}', slug)}';
      expect(video.videoUrl[const Locale('en')], expectedVideoUrlEn);
      expect(video.videoUrl[const Locale('nl')], expectedVideoUrlNl);

      // Check thumbnail URLs
      final expectedThumbUrlEn = WalletAssets.video_tour_thumbnail_asset_placeholder
          .replaceAll('{languageCode}', 'en')
          .replaceAll('{slug}', slug);
      final expectedThumbUrlNl = WalletAssets.video_tour_thumbnail_asset_placeholder
          .replaceAll('{languageCode}', 'nl')
          .replaceAll('{slug}', slug);
      expect(video.videoThumb[const Locale('en')], expectedThumbUrlEn);
      expect(video.videoThumb[const Locale('nl')], expectedThumbUrlNl);

      // Check subtitle URLs
      final expectedSubtitleUrlEn =
          '$testBaseUrl${WalletAssets.video_tour_subtitle_url_placeholder.replaceAll('{languageCode}', 'en').replaceAll('{slug}', slug)}';
      final expectedSubtitleUrlNl =
          '$testBaseUrl${WalletAssets.video_tour_subtitle_url_placeholder.replaceAll('{languageCode}', 'nl').replaceAll('{slug}', slug)}';
      expect(video.subtitleUrl[const Locale('en')], expectedSubtitleUrlEn);
      expect(video.subtitleUrl[const Locale('nl')], expectedSubtitleUrlNl);
    }
  });

  test('invoke provides localized titles and bullet points for all supported locales', () async {
    final result = await useCase.invoke();
    final videos = result.value!;

    for (final video in videos) {
      for (final locale in AppLocalizations.supportedLocales) {
        expect(video.title[locale], isNotNull);
        expect(video.title[locale], isNotEmpty);
        expect(video.bulletPoints[locale], isNotNull);
        expect(video.bulletPoints[locale], isNotEmpty);
      }
    }
  });

  test('createLocalizedAsset creates correct map for supported video locales', () {
    final result = useCase.createLocalizedAsset((locale) => 'test_url_${locale.languageCode}');

    expect(result.length, WalletAssets.supported_video_language_codes.length);
    for (final langCode in WalletAssets.supported_video_language_codes) {
      final locale = Locale(langCode);
      expect(result[locale], 'test_url_$langCode');
    }
  });

  test('createLocalizedText creates correct map for supported l10n locales', () {
    final result = useCase.createLocalizedText((l10n) => l10n.videoTitle_intro);

    expect(result.length, AppLocalizations.supportedLocales.length);
    for (final locale in AppLocalizations.supportedLocales) {
      final localizations = lookupAppLocalizations(locale);
      expect(result[locale], localizations.videoTitle_intro);
    }
  });
}
