import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:vector_graphics/vector_graphics.dart';

/// Takes an [asset] of either an SVG or a 'normal' image and
/// renders it with the provided [fit] and [alignment].
class SvgOrImage extends StatelessWidget {
  final String asset;
  final BoxFit fit;
  final Alignment alignment;

  const SvgOrImage({
    super.key,
    required this.asset,
    this.fit = BoxFit.contain,
    this.alignment = Alignment.center,
  });

  @override
  Widget build(BuildContext context) {
    switch (asset) {
      case (final String svg) when asset.endsWith('.svg'):
        return SvgPicture.asset(svg, fit: fit, alignment: alignment);
      case (final String vec) when asset.endsWith('.svg.vec'):
        return SvgPicture(
          AssetBytesLoader(vec, assetBundle: DefaultAssetBundle.of(context)),
          fit: fit,
          alignment: alignment,
        );
      case (final String other):
        return Image.asset(other, fit: fit, alignment: alignment);
    }
  }
}
