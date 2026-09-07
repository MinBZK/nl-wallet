import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../domain/model/app_image_data.dart';

/// Widget that renders any [AppImageData]
class AppImage extends StatelessWidget {
  final AppImageData asset;
  final BoxFit fit;
  final Alignment alignment;
  final String? altText;

  /// Optional dimensions. When only one of them is provided the other is derived
  /// from the image's aspect ratio, which keeps the widget snug around the image.
  final double? width, height;

  const AppImage({
    super.key,
    required this.asset,
    this.fit = BoxFit.contain,
    this.alignment = Alignment.center,
    this.altText,
    this.width,
    this.height,
  });

  @override
  Widget build(BuildContext context) {
    final object = asset;
    final Widget result;
    switch (object) {
      case SvgImage():
        result = SvgPicture.string(
          object.data,
          fit: fit,
          alignment: alignment,
          semanticsLabel: altText,
          width: width,
          height: height,
        );
      case AppAssetImage():
        result = Image(
          image: AssetImage(object.name),
          fit: fit,
          alignment: alignment,
          semanticLabel: altText,
          width: width,
          height: height,
        );
      case AppMemoryImage():
        result = Image.memory(
          object.data,
          fit: fit,
          alignment: alignment,
          semanticLabel: altText,
          width: width,
          height: height,
        );
    }
    return ExcludeSemantics(
      excluding: altText == null,
      child: result,
    );
  }
}
