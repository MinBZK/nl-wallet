import 'package:flutter/material.dart';

import '../../../domain/model/card_front.dart';
import 'wallet_card_front.dart';

const _kCardOverlap = 56.0;

class StackedWalletCards extends StatelessWidget {
  final List<CardFront> cards;

  const StackedWalletCards({required this.cards, Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    List<Widget> children = List<Widget>.generate(cards.length, (index) {
      return Padding(
        padding: EdgeInsets.fromLTRB(0, index * _kCardOverlap, 0, 0),
        child: WalletCardFront(cardFront: cards[index]),
      );
    });

    return Stack(children: children);
  }
}
