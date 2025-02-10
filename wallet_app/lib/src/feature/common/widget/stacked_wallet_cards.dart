import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';
import '../../../util/extension/build_context_extension.dart';
import 'card/wallet_card_item.dart';

const _kCardOverlap = 56.0;

class StackedWalletCards extends StatelessWidget {
  final List<WalletCard> cards;
  final Function(WalletCard)? onCardPressed;

  const StackedWalletCards({
    required this.cards,
    this.onCardPressed,
    super.key,
  });

  @override
  Widget build(BuildContext context) {
    // Take the textScaling into account when offsetting the stacked cards (to account for larger titles)
    final cardOffsetY = context.mediaQuery.textScaler.scale(_kCardOverlap);
    final List<Widget> children = List<Widget>.generate(cards.length, (index) {
      return Padding(
        padding: EdgeInsets.fromLTRB(0, index * cardOffsetY, 0, 0),
        child: Hero(
          tag: cards[index].id,
          flightShuttleBuilder: (
            BuildContext flightContext,
            Animation<double> animation,
            HeroFlightDirection flightDirection,
            BuildContext fromHeroContext,
            BuildContext toHeroContext,
          ) =>
              WalletCardItem.buildShuttleCard(animation, cards[index].front, ctaAnimation: CtaAnimation.fadeIn),
          child: Semantics(
            button: onCardPressed != null,
            child: GestureDetector(
              child: WalletCardItem.fromCardFront(context: context, front: cards[index].front),
              onTap: () => onCardPressed?.call(cards[index]),
            ),
          ),
        ),
      );
    });
    return Stack(children: children);
  }
}
