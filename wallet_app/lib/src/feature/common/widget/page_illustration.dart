import 'package:flutter/material.dart';

import '../../../theme/wallet_theme.dart';
import '../../../util/extension/build_context_extension.dart';
import 'svg_or_image.dart';

const _kLandscapeHeight = 160.0;
const _kPortraitHeight = 214.0;
const _kContainerColor = Color(0xFFF5F4F9);

class PageIllustration extends StatelessWidget {
  final String asset;
  final EdgeInsets padding;

  const PageIllustration({
    required this.asset,
    this.padding = const EdgeInsets.symmetric(horizontal: 16),
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: padding,
      decoration: const BoxDecoration(
        color: _kContainerColor,
        borderRadius: WalletTheme.kBorderRadius12,
      ),
      height: context.isLandscape ? _kLandscapeHeight : _kPortraitHeight,
      width: double.infinity,
      child: SvgOrImage(
        asset: asset,
        fit: context.isLandscape ? BoxFit.contain : BoxFit.scaleDown,
      ),
    );
  }
}
