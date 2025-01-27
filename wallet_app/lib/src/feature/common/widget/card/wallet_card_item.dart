import 'package:flutter/material.dart';

import '../../../../domain/model/attribute/attribute.dart';
import '../../../../domain/model/card_front.dart';
import '../../../../theme/dark_wallet_theme.dart';
import '../../../../theme/light_wallet_theme.dart';
import '../../../../util/extension/build_context_extension.dart';
import '../../../../util/extension/object_extension.dart';
import '../../../../util/extension/string_extension.dart';
import '../../../../util/extension/text_style_extension.dart';
import '../animated_fade_in.dart';
import '../animated_fade_out.dart';
import '../svg_or_image.dart';
import 'card_holograph.dart';
import 'card_logo.dart';
import 'show_details_cta.dart';

const _kCardRenderSize = Size(328, 192);
const _kCardBorderRadius = BorderRadius.all(Radius.circular(12));
const _kCardContentPadding = 24.0;
const _kLightBrightnessTextColor = LightWalletTheme.textColor;
const _kDarkBrightnessTextColor = DarkWalletTheme.textColor;

class WalletCardItem extends StatefulWidget {
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

  /// Specify how to animate the 'show details' cta on the initial build
  final CtaAnimation? ctaAnimation;

  /// Callback that is triggered when the card is clicked
  ///
  /// 'Show Details' CTA will be hidden if [onPressed] is null.
  final VoidCallback? onPressed;

  /// If the text should be scaled based on the device's textScaleFactor
  /// Worth disabling if widget is used as a thumbnail.
  final bool scaleText;

  /// Show the title & subtitle, defaults to true.
  final bool showText;

  const WalletCardItem({
    super.key,
    required this.title,
    this.subtitle1,
    this.subtitle2,
    required this.background,
    this.logo,
    this.holograph,
    required this.brightness,
    this.onPressed,
    this.ctaAnimation,
    this.scaleText = true,
    this.showText = true,
  });

  WalletCardItem.fromCardFront({
    required BuildContext context,
    required CardFront front,
    this.onPressed,
    this.ctaAnimation,
    this.scaleText = true,
    this.showText = true,
    super.key,
  })  : title = front.title.l10nValue(context),
        background = front.backgroundImage,
        logo = front.logoImage,
        holograph = front.holoImage,
        subtitle1 = front.subtitle?.l10nValue(context),
        subtitle2 = front.info?.l10nValue(context),
        brightness = front.theme == CardFrontTheme.light ? Brightness.light : Brightness.dark;

  @override
  State<WalletCardItem> createState() => _WalletCardItemState();

  static Widget buildShuttleCard(
    Animation<double> animation,
    CardFront front, {
    CtaAnimation ctaAnimation = CtaAnimation.visible,
  }) {
    final scaleTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1, end: 1.05).chain(CurveTween(curve: Curves.easeIn)),
          weight: 30,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1.05),
          weight: 60,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 1.05, end: 1).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 10,
        ),
      ],
    );

    final perspectiveTween = TweenSequence<double>(
      <TweenSequenceItem<double>>[
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0, end: 0.2).chain(CurveTween(curve: Curves.easeInCubic)),
          weight: 20,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0.2),
          weight: 65,
        ),
        TweenSequenceItem<double>(
          tween: Tween<double>(begin: 0.2, end: 0).chain(CurveTween(curve: Curves.decelerate)),
          weight: 15,
        ),
      ],
    );

    final VoidCallback? onPressed = switch (ctaAnimation) {
      CtaAnimation.fadeIn => () {},
      CtaAnimation.fadeOut => () {},
      CtaAnimation.visible => () {},
      CtaAnimation.invisible => null,
    };

    return AnimatedBuilder(
      animation: animation,
      child: Builder(
        builder: (context) {
          return WalletCardItem.fromCardFront(
            context: context,
            front: front,
            ctaAnimation: ctaAnimation,
            onPressed: onPressed,
          );
        },
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

class _WalletCardItemState extends State<WalletCardItem> {
  late WidgetStatesController _statesController;

  @override
  void initState() {
    super.initState();
    _statesController = WidgetStatesController();
    WidgetsBinding.instance.addPostFrameCallback((_) => _statesController.addListener(() => setState(() {})));
  }

  @override
  void dispose() {
    _statesController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MediaQuery(
      data: context.mediaQuery.copyWith(textScaler: widget.scaleText ? null : TextScaler.noScaling),
      child: Theme(
        data: _resolveTheme(context),
        child: Builder(
          builder: (context) {
            return FittedBox(
              child: Container(
                constraints: BoxConstraints(
                  maxWidth: _kCardRenderSize.width,
                  minHeight: _kCardRenderSize.height,
                ),
                child: ClipRRect(
                  borderRadius: _kCardBorderRadius,
                  child: MergeSemantics(
                    child: Stack(
                      children: [
                        _buildBackground(context),
                        _buildHolograph(context, _kCardRenderSize.height),
                        _buildContent(context),
                        _buildPositionedShowDetailsCta(context),
                        _buildRippleAndFocus(context),
                      ],
                    ),
                  ),
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
        asset: widget.background,
        fit: BoxFit.cover,
        alignment: Alignment.topCenter,
      ),
    );
  }

  Widget _buildHolograph(BuildContext context, double height) {
    if (widget.holograph == null) return const SizedBox.shrink();
    return Positioned(
      top: 0,
      right: 0,
      height: height,
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: CardHolograph(
          holograph: widget.holograph!,
          brightness: widget.brightness,
        ),
      ),
    );
  }

  Widget _buildContent(BuildContext context) {
    return Focus(
      // Prevents the card content from being focused
      canRequestFocus: false,
      descendantsAreFocusable: false,
      child: Padding(
        padding: const EdgeInsets.all(_kCardContentPadding),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Semantics(
                    attributedLabel: context.l10n.cardTitleSemanticsLabel.toAttributedString(context),
                    child: DefaultTextStyle(
                      style: context.textTheme.displaySmall!.underlineWhenPressedOrFocused(_statesController.value),
                      child: Text.rich((widget.title.takeIf((_) => widget.showText) ?? '').toTextSpan(context)),
                    ),
                  ),
                  const SizedBox(height: 4),
                  DefaultTextStyle(
                    style: context.textTheme.bodyLarge!.underlineWhenPressedOrFocused(_statesController.value),
                    child: Text.rich((widget.subtitle1.takeIf((_) => widget.showText) ?? '').toTextSpan(context)),
                  ),
                  const SizedBox(height: 4),
                  DefaultTextStyle(
                    style: context.textTheme.bodyLarge!.underlineWhenPressedOrFocused(_statesController.value),
                    child: Text.rich((widget.subtitle2.takeIf((_) => widget.showText) ?? '').toTextSpan(context)),
                  ),
                  const SizedBox(height: 16),
                  Opacity(
                    /* guarantees correct spacing to 'show details' cta rendered at the bottom of the card */
                    opacity: 0,
                    child: _buildShowDetailsCta(context),
                  ),
                  const SizedBox(height: 16),
                ],
              ),
            ),
            if (widget.logo != null) const SizedBox(width: 16),
            if (widget.logo != null) CardLogo(logo: widget.logo!),
          ],
        ),
      ),
    );
  }

  Widget _buildRippleAndFocus(BuildContext context) {
    return Positioned.fill(
      child: TextButton(
        style: context.theme.textButtonTheme.style?.copyWith(
          backgroundColor: const WidgetStatePropertyAll(Colors.transparent),
          padding: const WidgetStatePropertyAll(EdgeInsets.zero),
        ),
        statesController: _statesController,
        onPressed: widget.onPressed,
        child: const SizedBox.shrink(),
      ),
    );
  }

  Widget _buildPositionedShowDetailsCta(BuildContext context) {
    if (!_showDetailsCta) return const SizedBox.shrink();
    return Positioned(
      bottom: _kCardContentPadding,
      left: _kCardContentPadding,
      right: _kCardContentPadding,
      child: switch (widget.ctaAnimation) {
        null => widget.onPressed == null ? const SizedBox.shrink() : _buildShowDetailsCta(context),
        CtaAnimation.fadeIn => AnimatedFadeIn(child: _buildShowDetailsCta(context)),
        CtaAnimation.fadeOut => AnimatedFadeOut(child: _buildShowDetailsCta(context)),
        CtaAnimation.visible => _buildShowDetailsCta(context),
        CtaAnimation.invisible => const SizedBox.shrink(),
      },
    );
  }

  Widget _buildShowDetailsCta(BuildContext context) {
    return Focus(
      // Prevents the card content from being focused
      canRequestFocus: false,
      descendantsAreFocusable: false,
      child: Row(
        children: [
          IntrinsicWidth(
            child: ShowDetailsCta(
              brightness: widget.brightness,
              onPressed: () => {},
              text: Text.rich(
                context.l10n.showDetailsCta.toTextSpan(context),
              ),
            ),
          ),
        ],
      ),
    );
  }

  /// Resolve the [ThemeData] for the selected [widget.brightness], making sure the text contrasts the provided [widget.background]
  ThemeData _resolveTheme(BuildContext context) {
    final textColor = widget.brightness == Brightness.light ? _kLightBrightnessTextColor : _kDarkBrightnessTextColor;
    return context.theme.copyWith(
      textTheme: context.textTheme.apply(
        bodyColor: textColor,
        displayColor: textColor,
      ),
    );
  }

  bool get _showDetailsCta => widget.onPressed != null;
}

enum CtaAnimation { fadeIn, fadeOut, visible, invisible }
