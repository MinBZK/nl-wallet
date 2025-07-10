import 'package:flutter/foundation.dart';

@immutable
class TourVideoScreenArgument {
  static const _kSubtitleUrlKey = 'subtitleUrl';
  static const _kVideoUrlKey = 'videoUrl';
  static const _kVideoTitleKey = 'videoTitle';

  final String subtitleUrl;
  final String videoUrl;
  final String videoTitle;

  const TourVideoScreenArgument({
    required this.subtitleUrl,
    required this.videoUrl,
    required this.videoTitle,
  });

  Map<String, dynamic> toMap() {
    return {
      _kSubtitleUrlKey: subtitleUrl,
      _kVideoUrlKey: videoUrl,
      _kVideoTitleKey: videoTitle,
    };
  }

  TourVideoScreenArgument.fromMap(Map<String, dynamic> map)
      : subtitleUrl = map[_kSubtitleUrlKey],
        videoUrl = map[_kVideoUrlKey],
        videoTitle = map[_kVideoTitleKey];

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TourVideoScreenArgument &&
          runtimeType == other.runtimeType &&
          subtitleUrl == other.subtitleUrl &&
          videoUrl == other.videoUrl &&
          videoTitle == other.videoTitle;

  @override
  int get hashCode => Object.hash(
        runtimeType,
        subtitleUrl,
        videoUrl,
        videoTitle,
      );
}
