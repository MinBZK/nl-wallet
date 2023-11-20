import 'package:flutter/material.dart';

import '../../../../domain/model/app_image_data.dart';
import '../app_image.dart';

const _kBorderRadiusFactor = 10.0;

class OrganizationLogo extends StatelessWidget {
  final AppImageData image;
  final double size;

  const OrganizationLogo({
    required this.image,
    required this.size,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: size,
      height: size,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(size / _kBorderRadiusFactor),
        child: AppImage(asset: image, fit: BoxFit.cover),
      ),
    );
  }
}
