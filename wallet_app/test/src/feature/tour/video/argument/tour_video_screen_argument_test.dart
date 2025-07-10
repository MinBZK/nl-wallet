import 'package:flutter_test/flutter_test.dart';
import 'package:wallet/src/feature/tour/video/argument/tour_video_screen_argument.dart';

void main() {
  group('TourVideoScreenArgument', () {
    const subtitleUrl1 = 'http://example.com/subs1.srt';
    const videoUrl1 = 'http://example.com/video1.mp4';
    const videoTitle1 = 'Introduction Video';
    const subtitleUrl2 = 'http://example.com/subs2.vtt';
    const videoUrl2 = 'http://example.com/video2.mp4';
    const videoTitle2 = 'Advanced Features';

    const argument1 = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl1,
      videoUrl: videoUrl1,
      videoTitle: videoTitle1,
    );

    const argument1Copy = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl1, // Same as argument1
      videoUrl: videoUrl1, // Same as argument1
      videoTitle: videoTitle1, // Same as argument1
    );

    const argument2 = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl2,
      videoUrl: videoUrl2,
      videoTitle: videoTitle2,
    );

    const argumentDifferentSubtitle = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl2, // Different from argument1
      videoUrl: videoUrl1, // Same as argument1
      videoTitle: videoTitle1, // Same as argument1
    );

    const argumentDifferentVideo = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl1, // Same as argument1
      videoUrl: videoUrl2, // Different from argument1
      videoTitle: videoTitle1, // Same as argument1
    );

    const argumentDifferentTitle = TourVideoScreenArgument(
      subtitleUrl: subtitleUrl1, // Same as argument1
      videoUrl: videoUrl1, // Same as argument1
      videoTitle: videoTitle2, // Different from argument1
    );

    test('constructor assigns values correctly', () {
      expect(argument1.subtitleUrl, equals(subtitleUrl1));
      expect(argument1.videoUrl, equals(videoUrl1));
      expect(argument1.videoTitle, equals(videoTitle1));
    });

    group('Equatable (== and hashCode)', () {
      test('instances with same values should be equal', () {
        expect(argument1, equals(argument1Copy));
      });

      test('instances with different values should not be equal', () {
        expect(argument1, isNot(equals(argument2)));
        expect(argument1, isNot(equals(argumentDifferentSubtitle)));
        expect(argument1, isNot(equals(argumentDifferentVideo)));
        expect(argument1, isNot(equals(argumentDifferentTitle)));
      });

      test('hashCode should be consistent for equal objects', () {
        expect(argument1.hashCode, equals(argument1Copy.hashCode));
      });

      test('hashCode should ideally be different for unequal objects (though not guaranteed)', () {
        // This isn't a strict requirement of hashCode but good for well-behaved implementations.
        expect(argument1.hashCode, isNot(equals(argument2.hashCode)));
        expect(argument1.hashCode, isNot(equals(argumentDifferentSubtitle.hashCode)));
        expect(argument1.hashCode, isNot(equals(argumentDifferentVideo.hashCode)));
        expect(argument1.hashCode, isNot(equals(argumentDifferentTitle.hashCode)));
      });

      test('instances should not be equal to null', () {
        expect(argument1, isNot(equals(null)));
      });
    });

    group('toMap and fromMap', () {
      test('toMap produces correct map', () {
        final expectedMap = {
          'subtitleUrl': subtitleUrl1,
          'videoUrl': videoUrl1,
          'videoTitle': videoTitle1,
        };
        expect(argument1.toMap(), equals(expectedMap));
      });

      test('fromMap creates correct instance', () {
        final map = {
          'subtitleUrl': subtitleUrl1,
          'videoUrl': videoUrl1,
          'videoTitle': videoTitle1,
        };
        final fromMapInstance = TourVideoScreenArgument.fromMap(map);
        expect(fromMapInstance, equals(argument1));
      });

      test('fromMap handles map with extra keys gracefully', () {
        final mapWithExtra = {
          'subtitleUrl': subtitleUrl1,
          'videoUrl': videoUrl1,
          'videoTitle': videoTitle1,
          'extraKey': 'extraValue',
        };
        final fromMapInstance = TourVideoScreenArgument.fromMap(mapWithExtra);
        // Should still be equal to argument1 as extra keys are ignored by fromMap
        expect(fromMapInstance, equals(argument1));
      });
    });
  });
}
