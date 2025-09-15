import 'package:flutter/material.dart';
import 'package:lottie/lottie.dart';

import '../../../../../environment.dart';
import '../../../../theme/wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../button/icon/animation_control_icon_button.dart';

class WalletLottiePlayer extends StatefulWidget {
  final String asset;

  const WalletLottiePlayer({required this.asset, super.key});

  @override
  State<WalletLottiePlayer> createState() => _WalletLottiePlayerState();
}

class _WalletLottiePlayerState extends State<WalletLottiePlayer> {
  bool _playAnimations = true;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Container(
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: context.colorScheme.primaryContainer,
            borderRadius: WalletTheme.kBorderRadius12,
          ),
          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 24),
          child: Lottie.asset(
            widget.asset,
            fit: BoxFit.contain,
            animate: _playAnimations && !Environment.isTest,
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(16),
          child: _buildPlayPauseButton(context),
        ),
      ],
    );
  }

  Widget _buildPlayPauseButton(BuildContext context) {
    return AnimationControlIconButton(
      animationState: _playAnimations ? AnimationState.playing : AnimationState.paused,
      onPressed: () => setState(() => _playAnimations = !_playAnimations),
    );
  }
}
