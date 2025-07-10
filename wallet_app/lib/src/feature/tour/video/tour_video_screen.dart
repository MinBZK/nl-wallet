import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:fimber/fimber.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:internet_connection_checker/internet_connection_checker.dart';
import 'package:video_player/video_player.dart';

import '../../common/widget/centered_loading_indicator.dart';
import '../../common/widget/utility/do_on_init.dart';
import '../../error/error_page.dart';
import '../../lock/auto_lock_provider.dart';
import 'argument/tour_video_screen_argument.dart';
import 'widget/video_overlay.dart';

class TourVideoScreen extends StatefulWidget {
  static TourVideoScreenArgument getArgument(RouteSettings settings) {
    final args = settings.arguments;
    try {
      return TourVideoScreenArgument.fromMap(args! as Map<String, dynamic>);
    } catch (exception, stacktrace) {
      Fimber.e('Failed to decode $args', ex: exception, stacktrace: stacktrace);
      throw UnsupportedError('Make sure to pass in [TourVideoScreenArgument].toMap() when opening the TourVideoScreen');
    }
  }

  final String videoUrl;
  final String subtitleUrl;
  final String videoTitle;
  final InternetConnectionChecker? internetConnectionChecker;

  const TourVideoScreen({
    required this.videoUrl,
    required this.subtitleUrl,
    required this.videoTitle,
    this.internetConnectionChecker, // Optional [InternetConnectionChecker], falls back to [InternetConnectionChecker.instance] when not provided
    super.key,
  });

  @override
  State<TourVideoScreen> createState() => _TourVideoScreenState();
}

class _TourVideoScreenState extends State<TourVideoScreen> {
  late VideoPlayerController _controller;
  VideoPlayerInitState _playerInitState = VideoPlayerInitState.initializing;

  @override
  void initState() {
    super.initState();
    // Force fullscreen mode
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersive);

    _initVideoController();
  }

  Future<void> _initVideoController() async {
    Fimber.d('About to start streaming ${widget.videoUrl}. With subtitles: ${widget.subtitleUrl}');
    try {
      // Create video player controller with the provided video URL and captions
      _controller = VideoPlayerController.networkUrl(
        Uri.parse(widget.videoUrl),
        closedCaptionFile: _loadCaptions(widget.subtitleUrl),
      );
      await _controller.initialize();
      setState(() => _playerInitState = VideoPlayerInitState.ok);
      _controller.addListener(_onTick);
    } catch (ex) {
      Fimber.e('Error playing video', ex: ex);
      setState(() => _playerInitState = VideoPlayerInitState.error);
    }
  }

  /// Avoids showing [IdleWarningDialog] while video is playing
  void _onTick() => AutoLockProvider.of(context)?.resetIdleTimeout();

  @override
  void dispose() {
    // Revert fullscreen mode on dispose
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);

    // Dispose of the video player controller
    _controller.removeListener(_onTick);
    _controller.dispose();

    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: switch (_playerInitState) {
        VideoPlayerInitState.initializing => const CenteredLoadingIndicator(),
        VideoPlayerInitState.ok => _buildOk(context),
        VideoPlayerInitState.error => _buildError(context),
      },
    );
  }

  Widget _buildOk(BuildContext context) {
    return Stack(
      fit: StackFit.expand,
      children: <Widget>[
        _buildVideoPlayer(context),
        VideoOverlay(
          controller: _controller,
          onClosePressed: () => Navigator.pop(context),
          videoTitle: widget.videoTitle,
        ),
      ],
    );
  }

  Widget _buildVideoPlayer(BuildContext context) {
    return FittedBox(
      fit: BoxFit.contain,
      child: SizedBox(
        width: _controller.value.size.width,
        height: _controller.value.size.height,
        child: VideoPlayer(_controller),
      ),
    );
  }

  Widget _buildError(BuildContext context) {
    return FutureBuilder<bool>(
      future: (widget.internetConnectionChecker ?? InternetConnectionChecker.instance).hasConnection,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const CenteredLoadingIndicator();
        }
        late Widget errorPage;
        final hasInternet = snapshot.data ?? false;
        if (hasInternet) {
          errorPage = ErrorPage.generic(
            context,
            onPrimaryActionPressed: () => Navigator.pop(context),
            style: ErrorCtaStyle.close,
          );
        } else {
          errorPage = ErrorPage.noInternet(
            context,
            onPrimaryActionPressed: () => Navigator.pop(context),
            style: ErrorCtaStyle.close,
          );
        }
        return DoOnInit(
          // Disable fullScreen mode.
          onInit: (_) => SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge),
          child: errorPage,
        );
      },
    );
  }

  Future<ClosedCaptionFile> _loadCaptions(String subtitleUrl) async {
    try {
      final response = await HttpClient().getUrl(Uri.parse(subtitleUrl));
      final httpResponse = await response.close();
      final fileContents = await httpResponse.transform(utf8.decoder).join();
      return SubRipCaptionFile(fileContents);
    } catch (ex) {
      Fimber.e('Failed to load subtitles.', ex: ex);
      // Provide empty subtitles, so that the video is played normally.
      return SubRipCaptionFile('');
    }
  }
}

enum VideoPlayerInitState { initializing, ok, error }
