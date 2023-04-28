import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import '../card/sized_card_front.dart';

const _kCardDisplayWidth = 115.0;

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
              SizedCardFront(
                cardFront: cardFront,
                displayWidth: _kCardDisplayWidth,
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
