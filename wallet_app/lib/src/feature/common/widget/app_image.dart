import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../domain/model/app_image_data.dart';

/// Widget that renders any [AppImageData]
class AppImage extends StatelessWidget {
  final AppImageData asset;
  final BoxFit fit;
  final Alignment alignment;
  final String? altText;

  const AppImage({
    super.key,
    required this.asset,
    this.fit = BoxFit.contain,
    this.alignment = Alignment.center,
    this.altText,
  });

  @override
  Widget build(BuildContext context) {
    final object = asset;
    switch (object) {
      case SvgImage():
        return SvgPicture.string(object.data, fit: fit, alignment: alignment, semanticsLabel: altText);
      case AppAssetImage():
        return Image(image: AssetImage(object.name), fit: fit, alignment: alignment, semanticLabel: altText);
      case AppMemoryImage():
        return Image.memory(object.data, fit: fit, alignment: alignment, semanticLabel: altText);
    }
  }
}
