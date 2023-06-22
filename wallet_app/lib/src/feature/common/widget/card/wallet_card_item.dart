import 'package:flutter/material.dart';

import '../../../../domain/model/card_front.dart';
import '../../../../theme/dark_wallet_theme.dart';
import '../../../../theme/light_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../svg_or_image.dart';
import '../utility/limit_font_scaling.dart';
import 'card_holograph.dart';
import 'card_logo.dart';
import 'show_details_cta.dart';

const _kMaxCardTextScale = 2.5;
const _kCardAspectRatio = 328.0 / 192.0;
const _kCardBorderRadius = BorderRadius.all(Radius.circular(12));
const _kCardContentPadding = 24.0;
const _kLightBrightnessTextColor = LightWalletTheme.textColor;
const _kDarkBrightnessTextColor = DarkWalletTheme.textColor;

class WalletCardItem extends StatelessWidget {
  /// Defines the default size of the card when rendered
  static const kCardRenderSize = Size(328, 192);

  /// The cards title
  final String title;

  /// The background asset, rendered as the background of the card
  ///
  /// This background is expected to be relatively long (portrait aspect ratio) so
  /// that it can grow in size vertically to accommodate longer and scalable texts.
  final String background;

  /// Specifies the brightness of the card (mostly based on background)
  ///
  /// E.g. when card is said to be [Brightness.dark] the correct contrasting
  /// text colors will be selected (i.e. light text colors).
  final Brightness brightness;

  /// The cards subtitle, rendered below the title
  final String? subtitle1;

  /// The cards secondary subtitle, rendered below the subtitle
  final String? subtitle2;

  /// The logo asset rendered in the top right corner
  final String? logo;

  /// The holograph asset rendered behind the text
  final String? holograph;

  /// Callback that is triggered when the card is clicked
  ///
  /// 'Show Details' CTA will be hidden if [onPressed] is null.
  final VoidCallback? onPressed;

  const WalletCardItem({
    Key? key,
    required this.title,
    this.subtitle1,
    this.subtitle2,
    required this.background,
    this.logo,
    this.holograph,
    required this.brightness,
    this.onPressed,
  }) : super(key: key);

  WalletCardItem.fromCardFront({required CardFront front, this.onPressed, super.key})
      : title = front.title,
        background = front.backgroundImage,
        logo = front.logoImage,
        holograph = front.holoImage,
        subtitle1 = front.subtitle,
        subtitle2 = front.info,
        brightness = front.theme == CardFrontTheme.light ? Brightness.light : Brightness.dark;

  @override
  Widget build(BuildContext context) {
    return Theme(
      data: _resolveTheme(context),
      child: LimitFontScaling(
        maxTextScaleFactor: _kMaxCardTextScale,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final minCardHeight = constraints.maxWidth / _kCardAspectRatio;
            return ConstrainedBox(
              constraints: BoxConstraints(minHeight: minCardHeight),
              child: ClipRRect(
                borderRadius: _kCardBorderRadius,
                child: Stack(
                  children: [
                    _buildBackground(context),
                    _buildHolograph(context, minCardHeight),
                    _buildContent(context),
                    _buildShowDetailsCta(context),
                    _buildRippleAndFocus(context),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }

  Widget _buildBackground(BuildContext context) {
    return Positioned.fill(
      child: SvgOrImage(
        asset: background,
        fit: BoxFit.cover,
        alignment: Alignment.topCenter,
      ),
    );
  }

  Widget _buildHolograph(BuildContext context, double height) {
    if (holograph == null) return const SizedBox.shrink();
    return Positioned(
      top: 0,
      right: 0,
      height: height,
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: CardHolograph(
          holograph: holograph!,
          brightness: brightness,
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(_kCardContentPadding),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(title, style: context.textTheme.displaySmall),
                const SizedBox(height: 4),
                Text(subtitle1 ?? '', style: context.textTheme.bodyLarge),
                const SizedBox(height: 4),
                Text(subtitle2 ?? '', style: context.textTheme.bodyLarge),
                const SizedBox(height: 16),
                const Opacity(
                  /* guarantees correct spacing to 'show details' cta rendered at the bottom of the card */
                  opacity: 0,
                  child: ShowDetailsCta(),
                ),
              ],
            ),
          ),
          if (logo != null) const SizedBox(width: 16),
          if (logo != null) CardLogo(logo: logo!),
        ],
      ),
    );
  }

  Widget _buildRippleAndFocus(BuildContext context) {
    return Positioned.fill(
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: _kCardBorderRadius,
          onTap: onPressed,
        ),
      ),
    );
  }

  Widget _buildShowDetailsCta(BuildContext context) {
    if (onPressed == null) return const SizedBox.shrink();
    return const Positioned(
      bottom: _kCardContentPadding,
      left: _kCardContentPadding,
      right: _kCardContentPadding,
      child: ShowDetailsCta(),
    );
  }

  /// Resolve the [ThemeData] for the selected [brightness], making sure the text contrasts the provided [background]
  ThemeData _resolveTheme(BuildContext context) {
    final textColor = brightness == Brightness.light ? _kLightBrightnessTextColor : _kDarkBrightnessTextColor;
    return context.theme.copyWith(
      textTheme: context.textTheme.apply(bodyColor: textColor, displayColor: textColor),
    );
  }
}
