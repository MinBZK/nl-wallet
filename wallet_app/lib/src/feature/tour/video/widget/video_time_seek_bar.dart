import 'package:flutter/material.dart';

import '../../../../theme/base_wallet_theme.dart';
import '../../../../theme/dark_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';

class VideoTimeSeekBar extends StatelessWidget {
  final Duration position;
  final Duration duration;
  final Function(Duration seekToPosition) onPositionChanged;

  const VideoTimeSeekBar({
    required this.position,
    required this.duration,
    required this.onPositionChanged,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildTime(context),
        const SizedBox(height: 8),
        _buildSeekBar(context),
      ],
    );
  }

  Widget _buildSeekBar(BuildContext context) {
    final value = _calculateSliderValue(position, duration);
    return Slider(
      semanticFormatterCallback: (value) => context.l10n.videoPlayerPercentageIndicatorWCAGLabel((value * 100).toInt()),
      activeColor: Colors.white,
      inactiveColor: const Color(0xFFABABAB),
      padding: const EdgeInsets.symmetric(horizontal: 8),
      value: value,
      min: 0,
      max: 1,
      onChanged: _isEnabled ? _handleSliderChanged : null,
    );
  }

  bool get _isEnabled => duration != Duration.zero;

  void _handleSliderChanged(double value) {
    final seekToPosition = Duration(
      seconds: (duration.inSeconds * value).toInt(),
    );
    onPositionChanged(seekToPosition);
  }

  Widget _buildTime(BuildContext context) {
    final positionFormatted = _formatDuration(position);
    final durationFormatted = _formatDuration(duration);
    final timeDefaultTextStyle = DarkWalletTheme.textTheme.bodySmall;
    final timeBoldTextStyle = DarkWalletTheme.textTheme.bodyMedium?.copyWith(
      fontVariations: [BaseWalletTheme.fontVariationBold],
    );
    return Semantics(
      label: context.l10n.videoPlayerTimeIndicatorWCAGLabel(position.inSeconds, duration.inSeconds),
      excludeSemantics: true,
      child: Text.rich(
        TextSpan(
          children: [
            TextSpan(text: positionFormatted, style: timeBoldTextStyle),
            TextSpan(text: ' / $durationFormatted'),
          ],
          style: timeDefaultTextStyle,
        ),
      ),
    );
  }

  double _calculateSliderValue(Duration position, Duration duration) {
    if (duration.inSeconds > 0) {
      return position.inSeconds / duration.inSeconds;
    } else {
      return 0;
    }
  }

  String _formatDuration(Duration duration) {
    assert(
      duration.inMinutes <= 59,
      'If you are seeing this assertion, it is likely time to rewrite this method due to long form videos.',
    );
    String twoDigits(int n) => n.toString().padLeft(2, '0');
    final minutes = twoDigits(duration.inMinutes.remainder(60));
    final seconds = twoDigits(duration.inSeconds.remainder(60));
    return '$minutes:$seconds';
  }
}
