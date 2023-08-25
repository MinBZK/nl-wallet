import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';
import 'card/wallet_card_item.dart';

const _kCardOverlap = 56.0;

class StackedWalletCards extends StatelessWidget {
  final List<WalletCard> cards;
  final Function(WalletCard)? onCardPressed;

  const StackedWalletCards({
    required this.cards,
    this.onCardPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    List<Widget> children = List<Widget>.generate(cards.length, (index) {
      return Padding(
        padding: EdgeInsets.fromLTRB(0, index * _kCardOverlap, 0, 0),
        child: Hero(
          tag: cards[index].id,
          flightShuttleBuilder: (
            BuildContext flightContext,
            Animation<double> animation,
            HeroFlightDirection flightDirection,
            BuildContext fromHeroContext,
            BuildContext toHeroContext,
          ) =>
              _buildShuttleCard(animation, cards[index]),
          child: GestureDetector(
            child: WalletCardItem.fromCardFront(front: cards[index].front),
            onTap: () => onCardPressed?.call(cards[index]),
          ),
        ),
      );
    });
    return Stack(children: children);
  }

  Widget _buildShuttleCard(Animation<double> animation, WalletCard card) {
    final scaleTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.0, end: 1.05).chain(CurveTween(curve: Curves.easeIn)),
          weight: 30.0,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1.05),
          weight: 60.0,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1.0).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 10.0,
        ),
      ],
    );

    final perspectiveTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.0, end: 0.2).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 20.0,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0.2),
          weight: 65.0,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0.0).chain(CurveTween(curve: Curves.decelerate)),
          weight: 15.0,
        ),
      ],
    );

    return AnimatedBuilder(
      animation: animation,
      child: WalletCardItem.fromCardFront(
        front: card.front,
        fadeInCta: true,
        onPressed: () {},
      ),
      builder: (context, child) {
        return Transform(
          alignment: FractionalOffset.center,
          transform: Matrix4.identity()
            ..scale(scaleTween.evaluate(animation))
            ..setEntry(3, 2, 0.001)
            ..rotateX(perspectiveTween.evaluate(animation)),
          child: child,
        );
      },
    );
  }
}
