import 'package:flutter/material.dart';

import '../../../domain/model/card_front.dart';

const _kCardAspectRatio = 328.0 / 192.0;
const _kCardBorderRadius = 12.0;
const _kLogoBorderRadius = 4.0;
const _kLogoHeight = 40.0;

const _kTitleMaxLines = 2;
const _kSubtitleMaxLines = 2;
const _kInfoMaxLines = 2;

class WalletCardFront extends StatelessWidget {
  final CardFront cardFront;
  final VoidCallback? onPressed;

  const WalletCardFront({
    required this.cardFront,
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: _createTextTheme(context),
      child: Builder(builder: (context) {
        return InkWell(
          onTap: onPressed,
          radius: _kCardBorderRadius,
          borderRadius: BorderRadius.circular(_kCardBorderRadius),
          child: AspectRatio(
            aspectRatio: _kCardAspectRatio,
            child: Card(
              elevation: 0.0,
              margin: EdgeInsets.zero,
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(_kCardBorderRadius)),
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
                    padding: const EdgeInsets.all(16.0),
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
                            const SizedBox(width: 12.0),
                            ClipRRect(
                              borderRadius: const BorderRadius.all(Radius.circular(_kLogoBorderRadius)),
                              child: Image(
                                height: _kLogoHeight,
                                image: AssetImage(cardFront.logoImage ?? ''),
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 16.0),
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
        );
      }),
    );
  }

  ThemeData _createTextTheme(BuildContext context) {
    final ColorScheme scheme = Theme.of(context).colorScheme;
    final Color textColor = cardFront.theme == CardFrontTheme.light ? scheme.onPrimary : scheme.onBackground;
    return Theme.of(context).copyWith(
      textTheme: Theme.of(context).textTheme.apply(
            bodyColor: textColor,
            displayColor: textColor,
          ),
    );
  }
}
