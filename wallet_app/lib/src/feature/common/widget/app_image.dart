import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../domain/model/app_image_data.dart';

/// Widget that renders any [AppImageData]
class AppImage extends StatelessWidget {
  final AppImageData asset;
  final BoxFit fit;
  final Alignment alignment;

  const AppImage({
    super.key,
    required this.asset,
    this.fit = BoxFit.contain,
    this.alignment = Alignment.center,
  });

  @override
  Widget build(BuildContext context) {
    switch (asset) {
      case SvgImage():
        return SvgPicture.string(asset.data, fit: fit, alignment: alignment);
      case AppAssetImage():
        return Image(image: AssetImage(asset.data), fit: fit, alignment: alignment);
      case Base64Image():
        return Image.memory(const Base64Decoder().convert(asset.data), fit: fit, alignment: alignment);
    }
  }
}
