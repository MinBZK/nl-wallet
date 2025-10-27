import 'dart:math';

import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../../wallet_assets.dart';

const _kLogoMaxHeightPortrait = 88.0;
const _kLogoMinHeightPortrait = 64.0;
const _kLogoHeightLandscape = 64.0;

class GovernmentLogo extends StatelessWidget {
  const GovernmentLogo({super.key});

  @override
  Widget build(BuildContext context) {
    final topPadding = context.mediaQuery.padding.top;
    final double portraitLogoHeight = min(_kLogoMaxHeightPortrait, _kLogoMinHeightPortrait + topPadding);
    return Image.asset(
      WalletAssets.logo_rijksoverheid_label,
      height: context.isLandscape ? _kLogoHeightLandscape : portraitLogoHeight,
      fit: BoxFit.contain,
    );
  }
}
