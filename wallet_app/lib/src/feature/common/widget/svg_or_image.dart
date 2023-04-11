import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

const kSvgFileExtensions = ['svg', 'svg.vec'];

/// Takes an [asset] of either an SVG or a 'normal' image and
/// renders it with the provided [fit] and [alignment].
class SvgOrImage extends StatelessWidget {
  final String asset;
  final BoxFit fit;
  final Alignment alignment;

  const SvgOrImage({
    Key? key,
    required this.asset,
    this.fit = BoxFit.contain,
    this.alignment = Alignment.center,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final isSvg = kSvgFileExtensions.any((extension) => asset.endsWith(extension));
    if (isSvg) {
      return SvgPicture.asset(
        asset,
        fit: fit,
        alignment: alignment,
      );
    } else {
      return Image.asset(
        asset,
        fit: fit,
        alignment: alignment,
      );
    }
  }
}
