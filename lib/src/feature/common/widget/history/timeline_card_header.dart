import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import '../wallet_card_front.dart';

const double _kCardFrontScaleFactor = 0.35;

class TimelineCardHeader extends StatelessWidget {
  final CardFront cardFront;

  const TimelineCardHeader({required this.cardFront, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    const walletCardOriginalHeight = kWalletCardHeight;
    const walletCardScaledWidth = kWalletCardWidth * _kCardFrontScaleFactor;
    const walletCardScaledHeight = kWalletCardHeight * _kCardFrontScaleFactor;

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
          child: Column(
            children: [
              SizedBox(
                width: walletCardScaledWidth,
                height: walletCardScaledHeight,
                child: FittedBox(
                  alignment: Alignment.center,
                  child: SizedBox(
                    height: walletCardOriginalHeight,
                    child: WalletCardFront(cardFront: cardFront, onPressed: null),
                  ),
                ),
              ),
              const SizedBox(height: 24),
              Text(
                cardFront.title,
                style: Theme.of(context).textTheme.headline2,
              ),
            ],
          ),
        ),
        const Divider(height: 1),
      ],
    );
  }
}
