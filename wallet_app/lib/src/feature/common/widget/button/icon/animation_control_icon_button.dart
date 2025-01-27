import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

import '../../../../../util/extension/build_context_extension.dart';
import '../../../../../util/extension/string_extension.dart';

class AnimationControlIconButton extends StatelessWidget {
  final AnimationState animationState;
  final VoidCallback onPressed;

  const AnimationControlIconButton({
    required this.animationState,
    required this.onPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      attributedLabel: _resolveAttributedLabel(context),
      button: true,
      excludeSemantics: true,
      onTap: onPressed,
      child: IconButton(
        onPressed: onPressed,
        style: context.theme.iconButtonTheme.style?.copyWith(
          backgroundColor: WidgetStateColor.resolveWith(
            (states) => context.colorScheme.surface,
          ),
          shape: WidgetStatePropertyAll(
            RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
          ),
        ),
        icon: _resolveIcon(),
      ),
    );
  }

  AttributedString _resolveAttributedLabel(BuildContext context) {
    switch (animationState) {
      case AnimationState.playing:
        return context.l10n.introductionWCAGPauseButtonLabel.toAttributedString(context);
      case AnimationState.paused:
        return context.l10n.introductionWCAGPlayButtonLabel.toAttributedString(context);
    }
  }

  Icon _resolveIcon() {
    switch (animationState) {
      case AnimationState.playing:
        return const Icon(Icons.pause_outlined);
      case AnimationState.paused:
        return const Icon(Icons.play_arrow_rounded);
    }
  }
}

enum AnimationState { playing, paused }
