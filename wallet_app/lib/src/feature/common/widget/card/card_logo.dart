import 'package:flutter/material.dart';

const kLogoBorderRadius = 4.0;
const kLogoHeight = 40.0;

class CardLogo extends StatelessWidget {
  final String logo;
  final String? altText;

  const CardLogo({
    required this.logo,
    this.altText,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(kLogoBorderRadius),
      child: Image.asset(
        logo,
        height: kLogoHeight,
        semanticLabel: altText,
      ),
    );
  }
}
