import 'package:flutter/material.dart';

import '../../../domain/model/tour/tour_video.dart';
import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

final class TourVideoData {
  static List<TourVideo> videos(BuildContext context) {
    return [
      TourVideo(
        title: context.l10n.tourOverviewVideo1Title,
        bulletPoints: context.l10n.tourOverviewVideo2BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_1_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_1_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo2Title,
        bulletPoints: context.l10n.tourOverviewVideo2BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_2_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_2_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo3Title,
        bulletPoints: context.l10n.tourOverviewVideo3BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_3_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_3_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo4Title,
        bulletPoints: context.l10n.tourOverviewVideo4BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_4_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_4_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo5Title,
        bulletPoints: context.l10n.tourOverviewVideo5BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_5_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_5_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo6Title,
        bulletPoints: context.l10n.tourOverviewVideo6BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_6_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_6_en,
        },
      ),
      TourVideo(
        title: context.l10n.tourOverviewVideo7Title,
        bulletPoints: context.l10n.tourOverviewVideo7BulletPoints,
        videoThumb: {
          Locale('nl'): WalletAssets.image_tour_video_thumb_7_nl,
          Locale('en'): WalletAssets.image_tour_video_thumb_7_en,
        },
      ),
    ];
  }
}
