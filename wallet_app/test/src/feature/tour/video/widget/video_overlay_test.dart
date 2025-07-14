import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:video_player/video_player.dart';
import 'package:wallet/src/data/service/semantics_event_service.dart';
import 'package:wallet/src/feature/common/widget/centered_loading_indicator.dart';
import 'package:wallet/src/feature/tour/video/widget/video_caption.dart';
import 'package:wallet/src/feature/tour/video/widget/video_overlay.dart';
import 'package:wallet/src/feature/tour/video/widget/video_time_seek_bar.dart';

import '../../../../../wallet_app_test_widget.dart';
import '../../../../mocks/wallet_mocks.mocks.dart'; // Import the golden test utilities (ensure you have this or a similar utility)
import '../../../../test_util/golden_utils.dart';

// Helper to pump the widget with necessary wrappers
Future<void> pumpVideoOverlay(
  WidgetTester tester, {
  required VideoPlayerController controller,
  VoidCallback? onClosePressed,
  bool autoPlay = false,
  Size screenSize = iphoneXSizeLandscape,
}) async {
  await tester.pumpWidgetWithAppWrapper(
    VideoOverlay(
      videoTitle: 'Video Title',
      controller: controller,
      onClosePressed: onClosePressed ?? () {},
      autoPlay: autoPlay,
    ),
    surfaceSize: screenSize,
    providers: [RepositoryProvider<SemanticsEventService>(create: (_) => MockSemanticsEventService())],
  );
  await tester.pumpAndSettle(); // Ensure layout is complete
}

// Default VideoPlayerValue for tests
VideoPlayerValue initialVideoValue = const VideoPlayerValue(
  duration: Duration(seconds: 60),
  position: Duration.zero,
  buffered: [],
  isPlaying: false,
  isLooping: false,
  volume: 1,
  caption: Caption.none,
  captionOffset: Duration.zero,
  isInitialized: true,
  errorDescription: null,
  size: Size(1920, 1080),
);

void main() {
  late MockVideoPlayerController mockController;
  late StreamController<VideoPlayerValue> videoValueStreamController;

  setUp(() {
    mockController = MockVideoPlayerController();
    videoValueStreamController = StreamController<VideoPlayerValue>.broadcast();

    // Mock default controller behavior
    when(mockController.value).thenReturn(initialVideoValue);
    when(mockController.play()).thenAnswer((_) async {});
    when(mockController.pause()).thenAnswer((_) async {});
    when(mockController.seekTo(any)).thenAnswer((_) async {});
    when(mockController.setVolume(any)).thenAnswer((_) async {});
    when(mockController.addListener(any)).thenAnswer((invocation) {
      final listener = invocation.positionalArguments.first as VoidCallback;
      // Simulate listener calls when the stream emits a new value
      videoValueStreamController.stream.listen((_) => listener());
    });
    when(mockController.removeListener(any)).thenAnswer((_) {});
    when(mockController.dispose()).thenAnswer((_) async {});
  });

  tearDown(() {
    videoValueStreamController.close();
  });

  // Helper to update the mock controller's value and trigger a rebuild
  Future<void> updateVideoValue(WidgetTester tester, VideoPlayerValue value) async {
    when(mockController.value).thenReturn(value);
    videoValueStreamController.add(value); // Notify listeners
    await tester.pump(); // Rebuild with new value
  }

  group('VideoOverlay Tests', () {
    testWidgets('initial UI elements are displayed correctly', (WidgetTester tester) async {
      await pumpVideoOverlay(tester, controller: mockController, autoPlay: false);

      // Top Controls
      expect(find.byIcon(Icons.volume_off), findsOneWidget); // Assuming initial state is volume on
      expect(find.byIcon(Icons.subtitles), findsOneWidget); // Assuming initial state is captions off
      expect(find.byIcon(Icons.close), findsOneWidget);

      // Center Controls
      expect(find.byIcon(Icons.replay_10), findsOneWidget);
      expect(find.byIcon(Icons.play_arrow), findsOneWidget); // No autoplay, so initial state is paused
      expect(find.byIcon(Icons.forward_10), findsOneWidget);

      // Bottom Controls
      expect(find.byType(VideoTimeSeekBar), findsOneWidget);

      // No caption initially
      expect(find.byType(VideoCaption), findsNothing);

      // No loading indicator initially
      expect(find.byType(CenteredLoadingIndicator), findsNothing);
    });

    testWidgets('shows loading indicator when buffering', (WidgetTester tester) async {
      await pumpVideoOverlay(tester, controller: mockController);
      await updateVideoValue(tester, initialVideoValue.copyWith(isBuffering: true));
      await tester.pump(const Duration(milliseconds: 30));
      expect(find.byType(CenteredLoadingIndicator), findsOneWidget);
    });

    testWidgets('close button calls onClosePressed callback', (WidgetTester tester) async {
      bool onCloseCalled = false;
      await pumpVideoOverlay(
        tester,
        controller: mockController,
        onClosePressed: () => onCloseCalled = true,
      );

      await tester.tap(find.byIcon(Icons.close));
      await tester.pump();

      expect(onCloseCalled, isTrue);
    });

    group('Play/Pause Button', () {
      testWidgets('tapping play button calls controller.play() and updates icon to pause', (WidgetTester tester) async {
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: false);

        // Initial state is paused (play_arrow icon)
        expect(find.byIcon(Icons.play_arrow), findsOneWidget);
        expect(find.byIcon(Icons.pause), findsNothing);

        await tester.tap(find.byIcon(Icons.play_arrow));
        await tester.pump(); // Process tap

        verify(mockController.play()).called(1);

        // Simulate controller updating its state to playing
        await updateVideoValue(tester, initialVideoValue.copyWith(isPlaying: true));
        await tester.pumpAndSettle(kAutoHideFullScreenControlsDelay + const Duration(milliseconds: 100));

        // Icon should change to pause, and controls are hidden
        expect(find.byIcon(Icons.pause), findsNothing);

        // Let's tap the screen to ensure controls are visible for the check
        await tester.tapAt(const Offset(10, 10));
        await tester.pumpAndSettle();

        expect(find.byIcon(Icons.pause), findsOneWidget);
        expect(find.byIcon(Icons.play_arrow), findsNothing);
      });

      testWidgets('tapping pause button calls controller.pause() and updates icon to play',
          (WidgetTester tester) async {
        // Start in playing state
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: true));
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);
        await tester.pumpAndSettle(); // Initial build

        // Initial state is playing (pause icon)
        expect(find.byIcon(Icons.pause), findsOneWidget);
        expect(find.byIcon(Icons.play_arrow), findsNothing);

        await tester.tap(find.byIcon(Icons.pause));
        await tester.pump(); // Process tap

        verify(mockController.pause()).called(1);

        // Simulate controller updating its state to paused
        await updateVideoValue(tester, initialVideoValue.copyWith(isPlaying: false));

        expect(find.byIcon(Icons.play_arrow), findsOneWidget);
        expect(find.byIcon(Icons.pause), findsNothing);
      });

      testWidgets('tapping replay button calls controller.play() when video completed', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            isPlaying: false,
            position: initialVideoValue.duration,
            isCompleted: true,
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController);
        await tester.pumpAndSettle(); // for controls to show up as video is completed

        expect(find.byIcon(Icons.replay), findsOneWidget);

        await tester.tap(find.byIcon(Icons.replay));
        await tester.pump();

        verify(mockController.play()).called(1);
      });
    });

    group('Seek Buttons', () {
      testWidgets('tapping forward_10 calls controller.seekTo() with correct duration', (WidgetTester tester) async {
        final currentPosition = const Duration(seconds: 20);
        when(mockController.value).thenReturn(initialVideoValue.copyWith(position: currentPosition));
        await pumpVideoOverlay(tester, controller: mockController);

        await tester.tap(find.byIcon(Icons.forward_10));
        await tester.pump();

        final expectedSeekPosition = currentPosition + kVideoSeekByDuration;
        verify(mockController.seekTo(expectedSeekPosition)).called(1);
      });

      testWidgets('tapping replay_10 calls controller.seekTo() with correct duration', (WidgetTester tester) async {
        final currentPosition = const Duration(seconds: 20);
        when(mockController.value).thenReturn(initialVideoValue.copyWith(position: currentPosition));
        await pumpVideoOverlay(tester, controller: mockController);

        await tester.tap(find.byIcon(Icons.replay_10));
        await tester.pump();

        final expectedSeekPosition = currentPosition - kVideoSeekByDuration;
        verify(mockController.seekTo(expectedSeekPosition)).called(1);
      });
    });

    group('Volume Button', () {
      testWidgets('tapping volume_off calls controller.setVolume(0) and updates icon to volume_up',
          (WidgetTester tester) async {
        // Initial state: volume is on (1.0), icon is volume_off
        when(mockController.value).thenReturn(initialVideoValue.copyWith(volume: 1));
        await pumpVideoOverlay(tester, controller: mockController);

        expect(find.byIcon(Icons.volume_off), findsOneWidget);
        expect(find.byIcon(Icons.volume_up), findsNothing);

        await tester.tap(find.byIcon(Icons.volume_off));
        await tester.pump();

        verify(mockController.setVolume(0)).called(1);

        // Simulate controller updating its volume
        await updateVideoValue(tester, initialVideoValue.copyWith(volume: 0));

        expect(find.byIcon(Icons.volume_up), findsOneWidget);
        expect(find.byIcon(Icons.volume_off), findsNothing);
      });

      testWidgets('tapping volume_up calls controller.setVolume(1) and updates icon to volume_off',
          (WidgetTester tester) async {
        // Initial state: volume is off (0), icon is volume_up
        when(mockController.value).thenReturn(initialVideoValue.copyWith(volume: 0));
        await pumpVideoOverlay(tester, controller: mockController);

        expect(find.byIcon(Icons.volume_up), findsOneWidget);
        expect(find.byIcon(Icons.volume_off), findsNothing);

        await tester.tap(find.byIcon(Icons.volume_up));
        await tester.pump();

        verify(mockController.setVolume(1)).called(1);

        // Simulate controller updating its volume
        await updateVideoValue(tester, initialVideoValue.copyWith(volume: 1));

        expect(find.byIcon(Icons.volume_off), findsOneWidget);
        expect(find.byIcon(Icons.volume_up), findsNothing);
      });
    });

    group('Caption Button', () {
      testWidgets('tapping subtitles enables captions and updates icon to subtitles_off', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            caption: const Caption(number: 0, start: Duration.zero, end: Duration.zero, text: 'Test Caption'),
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController);

        // Initial state: captions off
        expect(find.byIcon(Icons.subtitles), findsOneWidget);
        expect(find.byIcon(Icons.subtitles_off), findsNothing);
        expect(find.byType(VideoCaption), findsNothing);

        await tester.tap(find.byIcon(Icons.subtitles));
        await tester.pumpAndSettle(); // pumpAndSettle for animation and state update

        expect(find.byIcon(Icons.subtitles_off), findsOneWidget);
        expect(find.byIcon(Icons.subtitles), findsNothing);
        expect(find.byType(VideoCaption), findsOneWidget);
        expect(find.text('Test Caption'), findsOneWidget);
      });

      testWidgets('tapping subtitles_off disables captions and updates icon to subtitles', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            caption: const Caption(number: 0, start: Duration.zero, end: Duration.zero, text: 'Test Caption'),
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController);

        // Enable captions first
        await tester.tap(find.byIcon(Icons.subtitles));
        await tester.pumpAndSettle();

        expect(find.byIcon(Icons.subtitles_off), findsOneWidget);
        expect(find.byType(VideoCaption), findsOneWidget);

        // Now tap to disable
        await tester.tap(find.byIcon(Icons.subtitles_off));
        await tester.pumpAndSettle();

        expect(find.byIcon(Icons.subtitles), findsOneWidget);
        expect(find.byIcon(Icons.subtitles_off), findsNothing);
        expect(find.byType(VideoCaption), findsNothing);
      });
    });

    group('Controls Visibility (Fullscreen Controls)', () {
      testWidgets('tapping overlay toggles fullscreen controls visibility', (WidgetTester tester) async {
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: true));
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);
        await tester.pumpAndSettle(); // Ensure initial state

        // Controls should be visible initially
        expect(find.byIcon(Icons.pause), findsOneWidget); // Center control
        expect(find.byType(VideoTimeSeekBar), findsOneWidget); // Bottom control

        // Tap to hide
        await tester.tapAt(const Offset(10, 10)); // Tap top left corner of gesture area
        await tester.pumpAndSettle(kControlAnimationDuration + const Duration(milliseconds: 50)); // Wait for animation

        expect(find.byIcon(Icons.pause), findsNothing);
        expect(find.byType(VideoTimeSeekBar), findsNothing);
        expect(
          find.byIcon(Icons.video_settings),
          findsOneWidget,
        ); // Settings icon should appear when controls are hidden

        // Tap to show again
        await tester.tapAt(const Offset(10, 10)); // Tap top left corner of gesture area
        await tester.pumpAndSettle(kControlAnimationDuration + const Duration(milliseconds: 50));

        expect(find.byIcon(Icons.pause), findsOneWidget);
        expect(find.byType(VideoTimeSeekBar), findsOneWidget);
        expect(find.byIcon(Icons.video_settings), findsNothing);
      });

      testWidgets('controls auto-hide when video is playing', (WidgetTester tester) async {
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: true));
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);
        await tester.pumpAndSettle(); // Initial build, controls visible

        // Controls are visible
        expect(find.byIcon(Icons.pause), findsOneWidget);

        // Wait for auto-hide delay
        await tester.pumpAndSettle(kAutoHideFullScreenControlsDelay + const Duration(milliseconds: 100));

        // Controls should be hidden
        expect(find.byIcon(Icons.pause), findsNothing);
        expect(find.byIcon(Icons.video_settings), findsOneWidget);
      });

      testWidgets('controls do not auto-hide when video is paused', (WidgetTester tester) async {
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: false));
        await pumpVideoOverlay(tester, controller: mockController);
        await tester.pumpAndSettle();

        // Controls are visible
        expect(find.byIcon(Icons.play_arrow), findsOneWidget);

        // Wait for auto-hide delay
        await tester.pumpAndSettle(kAutoHideFullScreenControlsDelay + const Duration(milliseconds: 100));

        // Controls should still be visible
        expect(find.byIcon(Icons.play_arrow), findsOneWidget);
        expect(find.byIcon(Icons.video_settings), findsNothing);
      });

      testWidgets('controls show up when video completes and were hidden', (WidgetTester tester) async {
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);
        await tester.pumpAndSettle();

        // Tap to hide controls initially
        await tester.tapAt(const Offset(10, 10));
        await tester.pumpAndSettle(kControlAnimationDuration + const Duration(milliseconds: 50));
        expect(find.byIcon(Icons.play_arrow), findsNothing); // Verify controls are hidden

        // Simulate video completing
        await updateVideoValue(
          tester,
          initialVideoValue.copyWith(position: initialVideoValue.duration, isPlaying: false, isCompleted: true),
        );
        await tester.pumpAndSettle();

        expect(find.byIcon(Icons.replay), findsOneWidget);
      });
    });

    // Golden Test Group
    group('goldens', () {
      testGoldens('initial paused state', (WidgetTester tester) async {
        // Ensure the mock controller is set to the initial paused state
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: false));

        await pumpVideoOverlay(tester, controller: mockController, autoPlay: false);

        // Let animations and UI settle.
        await tester.pumpAndSettle(const Duration(milliseconds: 500)); // Adjust delay if needed

        // The name here will be used for the golden file, e.g., video_overlay_initial_paused_state.png
        await screenMatchesGolden('initial.paused.controls');
      });

      testGoldens('playing state with controls hidden', (WidgetTester tester) async {
        // Set the controller to a playing state
        when(mockController.value).thenReturn(initialVideoValue.copyWith(isPlaying: true));
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true, screenSize: const Size(420, 250));
        await tester.pumpAndSettle(); // Initial pump, controls visible

        // Wait for controls to auto-hide
        await tester.pumpAndSettle(kAutoHideFullScreenControlsDelay + const Duration(milliseconds: 200));

        await screenMatchesGolden('playing.no_controls');
      });

      testGoldens('video completed state', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            isPlaying: false,
            position: initialVideoValue.duration,
            isCompleted: true,
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController);
        await tester.pumpAndSettle(); // Controls should show up

        await screenMatchesGolden('completed');
      });

      testGoldens('controls and captions visible', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            isPlaying: true, // Let's make it playing for this test
            caption: const Caption(
              number: 0,
              start: Duration.zero,
              end: Duration(seconds: 5),
              text: 'Hello Golden Test! This is a caption.',
            ),
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);

        // Tap to enable captions
        await tester.tap(find.byIcon(Icons.subtitles));
        await tester.pumpAndSettle(); // Settle after tap and state update

        await tester.pumpAndSettle();

        await screenMatchesGolden('playing.controls.captions');
      });

      testGoldens('no controls and captions visible', (WidgetTester tester) async {
        when(mockController.value).thenReturn(
          initialVideoValue.copyWith(
            isPlaying: true, // Let's make it playing for this test
            caption: const Caption(
              number: 0,
              start: Duration.zero,
              end: Duration(seconds: 5),
              text: 'Hello Golden Test! This is a caption.',
            ),
          ),
        );
        await pumpVideoOverlay(tester, controller: mockController, autoPlay: true);

        // Tap to enable captions
        await tester.tap(find.byIcon(Icons.subtitles));
        await tester.pumpAndSettle(); // Settle after tap and state update

        // Tap to instantly hide controls
        await tester.tapAt(const Offset(10, 10));
        await tester.pumpAndSettle();

        await screenMatchesGolden('playing.no_controls.captions');
      });
    });
  });
}
