import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:video_player/video_player.dart';
import 'package:wallet/src/feature/error/error_page.dart';
import 'package:wallet/src/feature/tour/video/tour_video_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/wallet_mocks.dart';
import '../../../test_util/golden_utils.dart';
import '../../../test_util/test_utils.dart';

void main() {
  late MockVideoPlayerController mockVideoController;
  late MockInternetConnectionChecker mockInternetChecker;

  setUp(() {
    mockVideoController = MockVideoPlayerController();
    mockInternetChecker = MockInternetConnectionChecker();

    // Setup default mock behaviors
    when(mockVideoController.value).thenReturn(
      const VideoPlayerValue(
        isInitialized: true,
        size: Size(1920, 1080),
        duration: Duration(minutes: 5),
      ),
    );
    when(mockVideoController.initialize()).thenAnswer((_) => Future.value());
    when(mockVideoController.dispose()).thenAnswer((_) => Future.value());
  });

  group('widgets', () {
    testWidgets('creates widget successfully', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const TourVideoScreen(
          videoTitle: 'Video Title',
          videoUrl: 'https://example.com/video.mp4',
          subtitleUrl: 'https://example.com/subtitles.srt',
        ),
      );

      // Widget should be created successfully
      expect(find.byType(TourVideoScreen), findsOneWidget);
      expect(find.byType(Scaffold), findsAtLeastNWidgets(1));
    });

    testWidgets('sets system UI mode to immersive on init', (tester) async {
      // Track system chrome calls
      final methodCalls = <MethodCall>[];
      tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        (MethodCall methodCall) async {
          methodCalls.add(methodCall);
          return null;
        },
      );

      await tester.pumpWidgetWithAppWrapper(
        const TourVideoScreen(
          videoTitle: 'Video Title',
          videoUrl: 'https://example.com/video.mp4',
          subtitleUrl: 'https://example.com/subtitles.srt',
        ),
      );

      // Verify immersive mode was set
      expect(
        methodCalls,
        contains(
          isA<MethodCall>()
              .having((call) => call.method, 'method', 'SystemChrome.setEnabledSystemUIMode')
              .having((call) => call.arguments, 'arguments', contains('SystemUiMode.immersive')),
        ),
      );
    });

    testWidgets('handles initialization failure gracefully', (tester) async {
      when(mockInternetChecker.hasConnection).thenAnswer((_) async => true);
      await tester.pumpWidgetWithAppWrapper(
        TourVideoScreen(
          videoTitle: 'Video Title',
          videoUrl: 'invalid-url',
          subtitleUrl: 'invalid-subtitle-url',
          internetConnectionChecker: mockInternetChecker,
        ),
      );

      // Wait for initialization to complete
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      // Should not crash and should show generic error UI
      expect(find.byType(ErrorPage), findsAtLeastNWidgets(1));
      expect(find.text(l10n.errorScreenGenericHeadline), findsOneWidget);
    });

    testWidgets('handles initialization failure gracefully - no internet', (tester) async {
      when(mockInternetChecker.hasConnection).thenAnswer((_) async => false);
      await tester.pumpWidgetWithAppWrapper(
        TourVideoScreen(
          videoTitle: 'Video Title',
          videoUrl: 'invalid-url',
          subtitleUrl: 'invalid-subtitle-url',
          internetConnectionChecker: mockInternetChecker,
        ),
      );

      // Wait for initialization to complete
      await tester.pumpAndSettle();

      final l10n = await TestUtils.englishLocalizations;
      // Should not crash and should show no internet error UI
      expect(find.byType(ErrorPage), findsAtLeastNWidgets(1));
      expect(find.text(l10n.errorScreenNoInternetHeadline), findsOneWidget);
    });
  });

  group('goldens', () {
    testGoldens('no internet error', (tester) async {
      when(mockInternetChecker.hasConnection).thenAnswer((_) async => false);
      await tester.pumpWidgetWithAppWrapper(
        TourVideoScreen(
          videoTitle: 'Video Title',
          videoUrl: 'invalid-url',
          subtitleUrl: 'invalid-subtitle-url',
          internetConnectionChecker: mockInternetChecker,
        ),
      );
      await tester.pumpAndSettle();

      await screenMatchesGolden('error.no_internet');
    });
  });

  group('TourVideoScreenArgument', () {
    test('getArgument parses valid arguments correctly', () {
      final routeSettings = const RouteSettings(
        arguments: {
          'videoTitle': 'video_title',
          'videoUrl': 'https://example.com/video.mp4',
          'subtitleUrl': 'https://example.com/subtitles.srt',
        },
      );

      final argument = TourVideoScreen.getArgument(routeSettings);

      expect(argument, isNotNull);
      expect(argument.videoTitle, equals('video_title'));
      expect(argument.videoUrl, equals('https://example.com/video.mp4'));
      expect(argument.subtitleUrl, equals('https://example.com/subtitles.srt'));
    });

    test('getArgument throws UnsupportedError for invalid arguments', () {
      final routeSettings = const RouteSettings(arguments: 'invalid arguments');

      expect(
        () => TourVideoScreen.getArgument(routeSettings),
        throwsA(isA<UnsupportedError>()),
      );
    });

    test('getArgument throws UnsupportedError for null arguments', () {
      final routeSettings = const RouteSettings(arguments: null);

      expect(
        () => TourVideoScreen.getArgument(routeSettings),
        throwsA(isA<UnsupportedError>()),
      );
    });
  });

  group('VideoPlayerInitState enum', () {
    test('has expected values', () {
      expect(VideoPlayerInitState.values, hasLength(3));
      expect(VideoPlayerInitState.values, contains(VideoPlayerInitState.initializing));
      expect(VideoPlayerInitState.values, contains(VideoPlayerInitState.ok));
      expect(VideoPlayerInitState.values, contains(VideoPlayerInitState.error));
    });
  });
}
