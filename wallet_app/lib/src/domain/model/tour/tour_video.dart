import 'package:equatable/equatable.dart';

import '../localized_asset.dart';
import '../localized_text.dart';

/// Represents a tour video with localized content including title, description,
/// thumbnail, and video/subtitle URLs.
class TourVideo extends Equatable {
  /// The localized title of the tour video
  final LocalizedText title;

  /// The localized bullet points describing the video content
  final LocalizedText bulletPoints;

  /// The localized asset path for the video thumbnail image
  final LocalizedAsset videoThumb;

  /// The localized asset path for the video file
  final LocalizedAsset videoUrl;

  /// The localized asset path for the video subtitle file
  final LocalizedAsset subtitleUrl;

  const TourVideo({
    required this.title,
    required this.bulletPoints,
    required this.videoThumb,
    required this.videoUrl,
    required this.subtitleUrl,
  });

  @override
  List<Object?> get props => [title, bulletPoints, videoThumb, videoUrl, subtitleUrl];
}
