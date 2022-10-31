import 'package:flutter/material.dart';

import '../../../domain/model/wallet_card.dart';

const _kCardAspectRatio = 328.0 / 192.0;
const _kCardBorderRadius = 12.0;
const _kLogoBorderRadius = 4.0;

const _kTitleMaxLines = 2;
const _kSubtitleMaxLines = 2;
const _kInfoMaxLines = 2;

class WalletCardFront extends StatelessWidget {
  final WalletCard walletCard;
  final Function(String cardId) onPressed;

  const WalletCardFront({
    required this.walletCard,
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () => onPressed(walletCard.id),
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
                image: AssetImage(walletCard.backgroundImage ?? ''),
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
                                walletCard.title,
                                maxLines: _kTitleMaxLines,
                                style: Theme.of(context).textTheme.subtitle1!,
                                overflow: TextOverflow.ellipsis,
                              ),
                              Text(
                                walletCard.subtitle ?? '',
                                maxLines: _kSubtitleMaxLines,
                                style: Theme.of(context).textTheme.bodyText2!,
                                overflow: TextOverflow.ellipsis,
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(width: 12.0),
                        ClipRRect(
                          borderRadius: const BorderRadius.all(Radius.circular(_kLogoBorderRadius)),
                          child: Image(
                            image: AssetImage(walletCard.logoImage ?? ''),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 16.0),
                    Text(
                      walletCard.info ?? '',
                      maxLines: _kInfoMaxLines,
                      style: Theme.of(context).textTheme.bodyText2!,
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
  }
}
