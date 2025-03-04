import 'package:flutter/material.dart';

import '../../../../domain/model/card/card_front.dart';
import '../svg_or_image.dart';
import 'card_holograph.dart';

class MockCardBackground extends StatelessWidget {
  final CardFront front;

  const MockCardBackground({required this.front, super.key});

  @override
  Widget build(BuildContext context) {
    final bgImage = SvgOrImage(asset: front.backgroundImage, fit: BoxFit.cover, alignment: Alignment.topCenter);
    if (front.holoImage == null) return bgImage;
    return Stack(
      children: [
        bgImage,
        Positioned(
          height: 192,
          right: 0,
          top: 0,
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: CardHolograph(
              holograph: front.holoImage ?? '',
              brightness: front.theme == CardFrontTheme.light ? Brightness.light : Brightness.dark,
            ),
          ),
        ),
      ],
    );
  }
}
