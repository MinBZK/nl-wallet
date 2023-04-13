import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import '../card/wallet_card_item.dart';

const _kCardRenderSize = Size(328, 192);
const _kCardDisplaySize = Size(115, 67);

class TimelineCardHeader extends StatelessWidget {
  final CardFront cardFront;

  const TimelineCardHeader({required this.cardFront, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Column(
            children: [
              SizedBox.fromSize(
                size: _kCardDisplaySize,
                child: FittedBox(
                  alignment: Alignment.center,
                  child: SizedBox.fromSize(
                    size: _kCardRenderSize,
                    child: WalletCardItem.fromCardFront(front: cardFront),
                  ),
                ),
              ),
              const SizedBox(height: 24),
              Text(
                cardFront.title,
                style: Theme.of(context).textTheme.displayMedium,
              ),
            ],
          ),
        ),
        const Divider(height: 1),
      ],
    );
  }
}
