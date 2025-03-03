import 'package:flutter/material.dart';

const kLogoBorderRadius = 4.0;
const kLogoHeight = 40.0;

class CardNetworkLogo extends StatelessWidget {
  final String uri;
  final String? altText;

  const CardNetworkLogo({
    required this.uri,
    this.altText,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(kLogoBorderRadius),
      child: Image.network(
        uri,
        height: kLogoHeight,
        semanticLabel: altText,
      ),
    );
  }
}
