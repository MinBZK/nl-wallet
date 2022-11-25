import 'package:flutter/material.dart';

import '../../../domain/model/card_front.dart';

const kWalletCardWidth = 328.0;
const kWalletCardHeight = 192.0;

const _kCardAspectRatio = kWalletCardWidth / kWalletCardHeight;
const _kTitleMaxLines = 2;
const _kSubtitleMaxLines = 2;
const _kInfoMaxLines = 2;

const _kDefaultPadding = 16.0;
const _kDefaultCardBorderRadius = 12.0;
const _kDefaultLogoBorderRadius = 4.0;
const _kDefaultLogoHeight = 40.0;

class WalletCardFront extends StatelessWidget {
  final CardFront cardFront;
  final VoidCallback? onPressed;

  const WalletCardFront({
    required this.cardFront,
    this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: _createTextTheme(context),
      child: Builder(builder: (context) {
        return InkWell(
          onTap: onPressed,
          radius: _kDefaultCardBorderRadius,
          borderRadius: BorderRadius.circular(_kDefaultCardBorderRadius),
          child: Center(
            child: AspectRatio(
              aspectRatio: _kCardAspectRatio,
              child: Card(
                elevation: 0.0,
                margin: EdgeInsets.zero,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_kDefaultCardBorderRadius)),
                semanticContainer: true,
                clipBehavior: Clip.antiAliasWithSaveLayer,
                child: Stack(
                  fit: StackFit.expand,
                  children: [
                    Image(
                      image: AssetImage(cardFront.backgroundImage ?? ''),
                      fit: BoxFit.fill,
                    ),
                    Padding(
                      padding: const EdgeInsets.all(_kDefaultPadding),
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          Row(
                            mainAxisSize: MainAxisSize.max,
                            mainAxisAlignment: MainAxisAlignment.start,
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Expanded(
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Text(
                                      cardFront.title,
                                      maxLines: _kTitleMaxLines,
                                      style: Theme.of(context).textTheme.subtitle1,
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                    Text(
                                      cardFront.subtitle ?? '',
                                      maxLines: _kSubtitleMaxLines,
                                      style: Theme.of(context).textTheme.bodyText2,
                                      overflow: TextOverflow.ellipsis,
                                    ),
                                  ],
                                ),
                              ),
                              const SizedBox(width: _kDefaultPadding),
                              ClipRRect(
                                borderRadius: const BorderRadius.all(Radius.circular(_kDefaultLogoBorderRadius)),
                                child: Image(
                                  height: _kDefaultLogoHeight,
                                  image: AssetImage(cardFront.logoImage ?? ''),
                                ),
                              ),
                            ],
                          ),
                          const SizedBox(height: _kDefaultPadding),
                          Text(
                            cardFront.info ?? '',
                            maxLines: _kInfoMaxLines,
                            style: Theme.of(context).textTheme.bodyText2,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      }),
    );
  }

  ThemeData _createTextTheme(BuildContext context) {
    final ColorScheme scheme = Theme.of(context).colorScheme;
    final Color textColor = cardFront.theme == CardFrontTheme.light ? scheme.onBackground : scheme.onPrimary;
    return Theme.of(context).copyWith(
      textTheme: Theme.of(context).textTheme.apply(bodyColor: textColor, displayColor: textColor),
    );
  }
}
