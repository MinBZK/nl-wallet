import 'package:flutter/foundation.dart';

import '../../../../../l10n/generated/app_localizations.dart';
import '../../../../data/repository/configuration/configuration_repository.dart';
import '../../../../util/extension/list_extension.dart';
import '../../../../wallet_assets.dart';
import '../../../model/localized_asset.dart';
import '../../../model/localized_text.dart';
import '../../../model/result/result.dart';
import '../../../model/tour/tour_video.dart';
import '../fetch_tour_videos_usecase.dart';

/// An implementation of [FetchTourVideosUseCase] that retrieves tour video
/// data, including localized titles, bullet points, video URLs, thumbnail URLs,
/// and subtitle URLs.
class FetchTourVideosUseCaseImpl extends FetchTourVideosUseCase {
  final ConfigurationRepository _configurationRepository;

  FetchTourVideosUseCaseImpl(this._configurationRepository);

  /// Fetches a list of [TourVideo] objects.
  ///
  /// This method retrieves the application configuration to determine the base URL
  /// for static assets. It then iterates through the predefined video slugs
  /// in [WalletAssets.video_slugs] and constructs a [TourVideo] object for each.
  @override
  Future<Result<List<TourVideo>>> invoke() async {
    final config = await _configurationRepository.appConfiguration.first;
    final baseUrl = config.staticAssetsBaseUrl;

    // Generate tour videos with localized content for all supported locales
    final videos = WalletAssets.video_slugs.map((slug) {
      return TourVideo(
        title: createLocalizedText(_titleMap(slug)),
        bulletPoints: createLocalizedText(_bulletPointsMap(slug)),
        videoThumb: createLocalizedAsset(
          (locale) => WalletAssets.video_tour_thumbnail_asset_placeholder
              .replaceAll('{languageCode}', locale.languageCode)
              .replaceAll('{slug}', slug),
        ),
        videoUrl: createLocalizedAsset(
          (locale) =>
              baseUrl +
              WalletAssets.video_tour_video_url_placeholder
                  .replaceAll('{languageCode}', locale.languageCode)
                  .replaceAll('{slug}', slug),
        ),
        subtitleUrl: createLocalizedAsset(
          (locale) =>
              baseUrl +
              WalletAssets.video_tour_subtitle_url_placeholder
                  .replaceAll('{languageCode}', locale.languageCode)
                  .replaceAll('{slug}', slug),
        ),
      );
    });
    return Result.success(videos.toList());
  }

  /// Creates a [LocalizedAsset] map.
  ///
  /// This helper function takes a `urlGenerator` function that produces a URL string
  /// for a given [Locale]. It then generates a map where keys are locales
  /// (derived from [WalletAssets.supported_video_language_codes]) and values are
  /// the corresponding URLs.
  @visibleForTesting
  LocalizedAsset createLocalizedAsset(String Function(Locale) urlGenerator) {
    final supportedVideoLocales = WalletAssets.supported_video_language_codes.map(Locale.new).toList();
    return supportedVideoLocales.toMap((locale) => urlGenerator(locale));
  }

  /// Creates a [LocalizedText] map.
  ///
  /// This helper function takes a [LocalizationGetter] function.
  /// It generates a map where keys are locales (from [AppLocalizations.supportedLocales])
  /// and values are the localized strings obtained by calling the `getter`
  /// with the appropriate [AppLocalizations] instance for each locale.
  @visibleForTesting
  LocalizedText createLocalizedText(LocalizationGetter getter) {
    final supportedL10nLocales = AppLocalizations.supportedLocales;
    return supportedL10nLocales.toMap((locale) => getter(lookupAppLocalizations(locale)));
  }

  /// Returns a [LocalizationGetter] function for the video title corresponding to the given [slug].
  /// Throws an [UnsupportedError] if the [slug] is not recognized.
  LocalizationGetter _titleMap(String slug) {
    return switch (slug) {
      'intro' => (AppLocalizations l10n) => l10n.videoTitle_intro,
      'cards-insight' => (AppLocalizations l10n) => l10n.videoTitle_cards_insight,
      'share-personal-information' => (AppLocalizations l10n) => l10n.videoTitle_share_personal_information,
      'login' => (AppLocalizations l10n) => l10n.videoTitle_login,
      'share-from-another-device' => (AppLocalizations l10n) => l10n.videoTitle_share_from_another_device,
      'insight-activities' => (AppLocalizations l10n) => l10n.videoTitle_insight_activities,
      'dark-mode-and-language' => (AppLocalizations l10n) => l10n.videoTitle_dark_mode_and_language,
      _ => throw UnsupportedError('Unknown slug: $slug'),
    };
  }

  /// Returns a [LocalizationGetter] function for the video bullet points corresponding to the given [slug].
  /// Throws an [UnsupportedError] if the [slug] is not recognized.
  LocalizationGetter _bulletPointsMap(String slug) {
    return switch (slug) {
      'intro' => (AppLocalizations l10n) => l10n.videoBulletPoints_intro,
      'cards-insight' => (AppLocalizations l10n) => l10n.videoBulletPoints_cards_insight,
      'share-personal-information' => (AppLocalizations l10n) => l10n.videoBulletPoints_share_personal_information,
      'login' => (AppLocalizations l10n) => l10n.videoBulletPoints_login,
      'share-from-another-device' => (AppLocalizations l10n) => l10n.videoBulletPoints_share_from_another_device,
      'insight-activities' => (AppLocalizations l10n) => l10n.videoBulletPoints_insight_activities,
      'dark-mode-and-language' => (AppLocalizations l10n) => l10n.videoBulletPoints_dark_mode_and_language,
      _ => throw UnsupportedError('Unknown slug: $slug'),
    };
  }
}

/// A typedef for a function that takes an [AppLocalizations] instance and returns a localized string.
typedef LocalizationGetter = String Function(AppLocalizations);
