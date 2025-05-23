import 'package:flutter/material.dart';

import '../../../../domain/model/app_image_data.dart';
import '../app_image.dart';

const _kBorderRadiusFactor = 10.0;

class OrganizationLogo extends StatelessWidget {
  final AppImageData image;
  final double size;

  /// Optional fixed value used as the [BorderRadius], when you don't want to base it on the logo's size.
  final double? fixedRadius;

  const OrganizationLogo({
    required this.image,
    required this.size,
    this.fixedRadius,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: size,
      height: size,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(fixedRadius ?? (size / _kBorderRadiusFactor)),
        child: AppImage(asset: image, fit: BoxFit.contain),
      ),
    );
  }
}
