import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import 'wallet_card_item.dart';

class SizedCardFront extends StatelessWidget {
  final CardFront cardFront;
  final double displayWidth;

  const SizedCardFront({
    required this.cardFront,
    required this.displayWidth,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SizedBox.fromSize(
      size: _calculateSize(),
      child: FittedBox(
        alignment: Alignment.center,
        child: SizedBox.fromSize(
          size: WalletCardItem.kCardRenderSize,
          child: WalletCardItem.fromCardFront(front: cardFront),
        ),
      ),
    );
  }

  Size _calculateSize() {
    const Size renderSize = WalletCardItem.kCardRenderSize;

    final double aspectRatio = displayWidth / WalletCardItem.kCardRenderSize.width;
    final double width = renderSize.width * aspectRatio;
    final double height = renderSize.height * aspectRatio;

    return Size(width, height);
  }
}
