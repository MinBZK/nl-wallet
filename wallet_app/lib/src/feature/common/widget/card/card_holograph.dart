import 'package:flutter/material.dart';
import 'package:foil/foil.dart';

import '../svg_or_image.dart';

class CardHolograph extends StatelessWidget {
  final String holograph;

  const CardHolograph({
    required this.holograph,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Foil(
      blendMode: BlendMode.srcATop,
      gradient: Foils.oilslick,
      child: SvgOrImage(
        asset: holograph,
        fit: BoxFit.scaleDown,
        alignment: Alignment.centerRight,
      ),
    );
  }
}
