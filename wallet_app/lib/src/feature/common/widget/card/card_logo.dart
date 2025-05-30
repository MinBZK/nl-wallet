import 'package:flutter/material.dart';

import '../../../../domain/model/app_image_data.dart';
import '../app_image.dart';

const kLogoBorderRadius = 4.0;
const kLogoHeight = 40.0;

class CardLogo extends StatelessWidget {
  final AppImageData logo;
  final String? altText;

  const CardLogo({
    required this.logo,
    this.altText,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ConstrainedBox(
      constraints: const BoxConstraints(
        minHeight: kLogoHeight,
        maxHeight: kLogoHeight,
        maxWidth: kLogoHeight * 2 /* add sane width restriction */,
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(kLogoBorderRadius),
        child: AppImage(
          asset: logo,
          altText: altText,
        ),
      ),
    );
  }
}
