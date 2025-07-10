import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:video_player/video_player.dart';

import '../../../../data/service/semantics_event_service.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../common/widget/centered_loading_indicator.dart';
import 'video_caption.dart';
import 'video_control_icon_button.dart';
import 'video_control_text_button.dart';
import 'video_time_seek_bar.dart';

// UI related consts
const _overlayBgColor = Color(0x99191C1B);
const double _kPlayButtonSize = 72;
const double _kTopControlsSize = 48;
const kVideoControlDefaultBg = Color(0x991C1E25);
const kVideoControlPressedOrFocusedBg = Color(0xE51C1E25);

// Timing related consts
const kControlAnimationDuration = Duration(milliseconds: 200);
const kAutoHideFullScreenControlsDelay = Duration(seconds: 3);
const kVideoSeekByDuration = Duration(seconds: 10);

class VideoOverlay extends StatefulWidget {
  final VideoPlayerController controller;
  final VoidCallback onClosePressed;
  final String videoTitle;
  final bool autoPlay; // Useful to control play state while testing

  const VideoOverlay({
    required this.controller,
    required this.onClosePressed,
    required this.videoTitle,
    this.autoPlay = true,
    super.key,
  });

  @override
  State<VideoOverlay> createState() => _VideoOverlayState();
}

class _VideoOverlayState extends State<VideoOverlay> {
  bool captionEnabled = false;
  bool fullscreenControlsEnabled = true;

  Timer? _autoHideFullScreenControlsTimer;
  StreamSubscription? _semanticsEventSubscription;

  VideoPlayerValue get videoPlayerState => widget.controller.value;

  @override
  void initState() {
    super.initState();
    if (widget.autoPlay) _playVideo();
    widget.controller.addListener(_onVideoTick);
    _semanticsEventSubscription = context
        .read<SemanticsEventService>()
        .actionEventStream
        .listen((_) => _resetDelayedAutoHideFullScreenControls());
  }

  void _onVideoTick() {
    setState(() {}); // Update UI which relies on [widget.controller].
    // Show (replay) controls when video is done playing
    if (!fullscreenControlsEnabled && videoPlayerState.isCompleted) {
      _showFullscreenControls(autoHide: false);
    }
  }

  @override
  void dispose() {
    widget.controller.removeListener(_onVideoTick);
    _autoHideFullScreenControlsTimer?.cancel();
    _semanticsEventSubscription?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      excludeFromSemantics: true,
      onTap: _toggleFullscreenControls,
      child: Stack(
        children: <Widget>[
          AnimatedContainer(
            duration: kControlAnimationDuration,
            color: fullscreenControlsEnabled ? _overlayBgColor : Colors.transparent,
            child: Column(
              children: [
                const SizedBox(height: 24),
                _buildTopControls(context),
                const Spacer(),
                if (captionEnabled)
                  Opacity(
                    opacity: fullscreenControlsEnabled ? 0.6 : 1,
                    child: VideoCaption(caption: videoPlayerState.caption.text),
                  ),
                SizedBox(height: fullscreenControlsEnabled ? 4 : 24),
                _buildBottomControls(context),
              ],
            ),
          ),
          if (fullscreenControlsEnabled) _buildCenterControls(),
          if (_showBufferIndicator()) _buildBufferIndicator(),
        ],
      ),
    );
  }

  Widget _buildBufferIndicator() => Semantics(
        attributedLabel: context.l10n.videoPlayerBufferingWCAGLabel.toAttributedString(context),
        child: const CenteredLoadingIndicator(),
      );

  bool _showBufferIndicator() {
    if (videoPlayerState.isPlaying) return false;
    if (videoPlayerState.isCompleted) return false;
    return videoPlayerState.isBuffering;
  }

  Widget _buildTopControls(BuildContext context) {
    final audioEnabled = videoPlayerState.volume == 1;

    return SafeArea(
      minimum: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (fullscreenControlsEnabled)
            Expanded(
              child: Wrap(
                spacing: 16,
                runSpacing: 16,
                alignment: context.isLandscape ? WrapAlignment.end : WrapAlignment.start,
                children: [
                  VideoControlTextButton(
                    icon: audioEnabled ? Icons.volume_off : Icons.volume_up,
                    label: audioEnabled
                        ? context.l10n.videoPlayerControlAudioOffToggleButton
                        : context.l10n.videoPlayerControlAudioOnToggleButton,
                    onPressed: () => widget.controller.setVolume(videoPlayerState.volume == 1 ? 0 : 1),
                  ),
                  VideoControlTextButton(
                    icon: captionEnabled ? Icons.subtitles_off : Icons.subtitles,
                    label: captionEnabled
                        ? context.l10n.videoPlayerControlCaptionOffToggleButton
                        : context.l10n.videoPlayerControlCaptionOnToggleButton,
                    onPressed: () => setState(() => captionEnabled = !captionEnabled),
                  ),
                ],
              ),
            ),
          const SizedBox(width: 16),
          if (!fullscreenControlsEnabled) ...[
            const Spacer(),
            SizedBox(
              width: _kTopControlsSize,
              height: _kTopControlsSize,
              child: VideoControlIconButton(
                icon: const Icon(Icons.video_settings),
                onPressed: _toggleFullscreenControls,
                attributedTooltip: context.l10n.videoPlayerControlsTooltip.toAttributedString(context),
              ),
            ),
            const SizedBox(width: 16),
          ],
          SizedBox(
            height: _kTopControlsSize,
            width: _kTopControlsSize,
            child: VideoControlIconButton(
              icon: const Icon(Icons.close),
              attributedTooltip: context.l10n.videoPlayerCloseTooltip.toAttributedString(context),
              onPressed: widget.onClosePressed,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCenterControls() {
    return Center(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        spacing: 40,
        children: [
          if (!videoPlayerState.isCompleted)
            VideoControlIconButton(
              icon: const Icon(Icons.replay_10),
              shape: const CircleBorder(),
              onPressed: _seekBackward,
              attributedTooltip: context.l10n.videoPlayerSeekBackwardTooltip.toAttributedString(context),
            ),
          Transform.scale(
            scale: _kPlayButtonSize / kDefaultVideoControlIconButtonSize,
            child: VideoControlIconButton(
              icon: _resolvePlayButtonIcon(videoPlayerState.isPlaying),
              shape: const CircleBorder(),
              attributedTooltip: _resolvePlayButtonTooltip(videoPlayerState.isPlaying).toAttributedString(context),
              onPressed: _togglePlayback,
            ),
          ),
          if (!videoPlayerState.isCompleted)
            VideoControlIconButton(
              icon: const Icon(Icons.forward_10),
              shape: const CircleBorder(),
              onPressed: _seekForward,
              attributedTooltip: context.l10n.videoPlayerSeekForwardTooltip.toAttributedString(context),
            ),
        ],
      ),
    );
  }

  Widget _resolvePlayButtonIcon(bool isPlaying) {
    if (isPlaying) return const Icon(Icons.pause);
    if (videoPlayerState.isCompleted) return const Icon(Icons.replay);
    return const Icon(Icons.play_arrow);
  }

  String _resolvePlayButtonTooltip(bool isPlaying) {
    if (isPlaying) return context.l10n.videoPlayerPauseTooltip;
    if (videoPlayerState.isCompleted) return context.l10n.videoPlayerReplayTooltip;
    return context.l10n.videoPlayerPlayTooltip(widget.videoTitle);
  }

  Widget _buildBottomControls(BuildContext context) {
    return Visibility(
      visible: fullscreenControlsEnabled,
      child: SafeArea(
        minimum: const EdgeInsets.symmetric(horizontal: 16).copyWith(bottom: 32),
        child: VideoTimeSeekBar(
          position: videoPlayerState.position,
          duration: videoPlayerState.duration,
          onPositionChanged: widget.controller.seekTo,
        ),
      ),
    );
  }

  void _resetDelayedAutoHideFullScreenControls() {
    _autoHideFullScreenControlsTimer?.cancel();
    if (!fullscreenControlsEnabled) return; // No controls to hide.
    // Schedule auto hide timer
    _autoHideFullScreenControlsTimer = Timer(kAutoHideFullScreenControlsDelay, () {
      if (videoPlayerState.isPlaying) _hideFullscreenControls();
    });
  }

  void _togglePlayback() => videoPlayerState.isPlaying ? widget.controller.pause() : _playVideo();

  void _playVideo() {
    widget.controller.play();
    _resetDelayedAutoHideFullScreenControls();
  }

  void _seekForward() => widget.controller.seekTo(widget.controller.value.position + kVideoSeekByDuration);

  void _seekBackward() => widget.controller.seekTo(widget.controller.value.position - kVideoSeekByDuration);

  void _toggleFullscreenControls() {
    if (fullscreenControlsEnabled) {
      _hideFullscreenControls();
    } else {
      _showFullscreenControls();
    }
  }

  void _hideFullscreenControls() => setState(() => fullscreenControlsEnabled = false);

  void _showFullscreenControls({bool autoHide = true}) {
    _autoHideFullScreenControlsTimer?.cancel();
    setState(() => fullscreenControlsEnabled = true);
    if (autoHide) _resetDelayedAutoHideFullScreenControls();
  }
}
